use sysinfo::System;

pub struct CpuMonitor {
    system: System,
}

impl CpuMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_cpu();
        CpuMonitor { system }
    }

    pub fn get_cpu_usage(&mut self) -> f32 {
        self.system.refresh_cpu();
        let cpus = self.system.cpus();
        let total: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        total / cpus.len() as f32
    }
}
