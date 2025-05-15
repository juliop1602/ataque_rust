use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};

pub struct CpuSpike {
    stop_flag: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl CpuSpike {
    /// Crea una nueva instancia de CpuSpike
    pub fn new(porcentaje: f32) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();

        // Determina cuántos hilos lanzar según el porcentaje
        let num_threads = ((num_cpus::get() as f32) * (porcentaje / 100.0)).ceil() as usize;

        for _ in 0..num_threads {
            let flag = Arc::clone(&stop_flag);
            let handle = thread::spawn(move || {
                while !flag.load(Ordering::Relaxed) {
                    // Cálculo inútil para mantener ocupado el hilo
                    let mut x = 0;
                    for i in 0..10_000 {
                        x += i;
                    }
                    std::hint::black_box(x); // Evita que el compilador optimice
                }
            });
            handles.push(handle);
        }

        CpuSpike { stop_flag, handles }
    }

    /// Detiene todos los hilos
    pub fn detener(self) {
        self.stop_flag.store(true, Ordering::Relaxed);

        for handle in self.handles {
            if let Err(e) = handle.join() {
                eprintln!("Error al unir hilo CPU Spike: {:?}", e);
            }
        }
    }
}

