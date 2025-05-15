use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering}
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use sysinfo::{System, RefreshKind};

pub struct MemoryLeak {
    stop_flag: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl MemoryLeak {
    pub fn new(porcentaje: f32, progresivo: bool) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&stop_flag);

        let handle = thread::spawn(move || {
            let mut data: Vec<Vec<u8>> = Vec::new();

            let mut sys = System::new_with_specifics(
                RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::new())
            );

            sys.refresh_memory();

            let total_ram = sys.total_memory(); // en KB
            let objetivo_total = (total_ram as f32 * (porcentaje / 100.0)) as u64;

            let bloque_tamano_kb = 10 * 1024; // 10 MB en KB
            let bloque_tamano = bloque_tamano_kb * 1024; // en bytes

            while !flag_clone.load(Ordering::Relaxed) {
                sys.refresh_memory();
                let usada_actual = sys.used_memory(); // en KB

                if usada_actual >= objetivo_total {
                    println!("Se alcanzó el objetivo de uso de memoria: {} KB", usada_actual);
                    break;
                }

                let mut bloque = Vec::new();

                if bloque.try_reserve_exact(bloque_tamano as usize).is_ok() {
                    bloque.resize(bloque_tamano as usize, 0);
                    data.push(bloque);

                    println!(
                        "Memoria usada: {} KB / Objetivo: {} KB",
                        usada_actual, objetivo_total
                    );

                    if progresivo {
                        thread::sleep(Duration::from_millis(200));
                    }
                } else {
                    println!("No se pudo reservar más memoria. Deteniendo para evitar crash.");
                    break;
                }
            }

            // Mantener ocupada la memoria hasta que se detenga el ataque
            while !flag_clone.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
            }
        });

        Self {
            stop_flag,
            handle: Some(handle),
        }
    }

    pub fn detener(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.join() {
                eprintln!("Error al unir hilo MemoryLeak: {:?}", e);
            }
        }
    }
}
