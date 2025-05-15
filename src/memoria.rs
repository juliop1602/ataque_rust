use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
            let total_ram = sysinfo::System::new_all().total_memory(); // en KB
            let objetivo = (total_ram as f32 * (porcentaje / 100.0)) as usize;

            let mut acumulado = 0;
            let bloque_tamano = 100 * 1024 * 1024; // 100 MB

            while !flag_clone.load(Ordering::Relaxed) && acumulado < objetivo {
                let mut bloque = Vec::new();

                // Intentamos reservar sin hacer panic
                if bloque.try_reserve_exact(bloque_tamano).is_ok() {
                    bloque.resize(bloque_tamano, 0);
                    acumulado += bloque.len() / 1024; // KB
                    data.push(bloque);

                    println!("Memoria acumulada: {} KB / {} KB", acumulado, objetivo);

                    if progresivo {
                        thread::sleep(Duration::from_millis(200));
                    }
                } else {
                    println!("No se pudo reservar más memoria. Deteniendo fuga para evitar crash.");
                    break;
                }
            }

            // Mantener la memoria ocupada hasta que se detenga
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
