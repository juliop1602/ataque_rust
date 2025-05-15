use sysinfo::System;

pub struct MemoriaInfo {
    system: System,
}

impl MemoriaInfo {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_memory();
        Self { system }
    }

    pub fn actualizar(&mut self) {
        self.system.refresh_memory();
    }

    pub fn porcentaje_uso_memoria(&self) -> f32 {
        let usada = self.system.used_memory() as f32;
        let total = self.system.total_memory() as f32;
        if total > 0.0 {
            (usada / total) * 100.0
        } else {
            0.0
        }
    }
}
