use eframe::{egui, Frame, NativeOptions};
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use std::time::Instant;

//Ataques
use crate::cpu_spike::CpuSpike;
use crate::ddos::DDoS;
use crate::memoria::MemoryLeak;

//Comunicacion
use crate::comunicacion::enviar_metricas;

//Monitoreo
use crate::monitor_cpu::CpuMonitor;
use crate::monitor_memoria::MemoriaInfo;
use crate::monitor_trafico::{MonitorRed, TraficoRed};

enum TipoAtaque {
    Ddos,
    CpuSpike,
    FugaMemoria,
}

impl fmt::Display for TipoAtaque {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TipoAtaque::Ddos => write!(f, "DDoS"),
            TipoAtaque::CpuSpike => write!(f, "CPU Spike"),
            TipoAtaque::FugaMemoria => write!(f, "Fuga de Memoria"),
        }
    }
}

struct AppState {
    activar_ddos: bool,
    activar_cpu_spike: bool,
    activar_fuga_memoria: bool,
    ddos_solicitudes_por_segundo: String,
    cpu_spike_porcentaje: f32,
    fuga_memoria_porcentaje_maximo: f32,
    log_actividades: String,
    ataques_activos: bool,
    tiempo_inicio_ddos: Option<Instant>,
    tiempo_inicio_cpu: Option<Instant>,
    tiempo_inicio_memoria: Option<Instant>,
    cpu_spike_handle: Option<CpuSpike>,
    memory_handle: Option<MemoryLeak>,
    ddos_handle: Option<DDoS>,
    //Monitorei del cpu
    cpu_monitor: CpuMonitor,
    cpu_usage: f32,
    ultima_actualizacion_cpu: Instant,
    //Monitoreo de la memoria
    memoria_monitor: MemoriaInfo,
    memoria_usage: f32,
    //Monitoreo tarfico
    monitor: MonitorRed,
    trafico_actual: TraficoRed,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            activar_ddos: false,
            activar_cpu_spike: false,
            activar_fuga_memoria: false,
            ddos_solicitudes_por_segundo: "10000".to_string(),
            cpu_spike_porcentaje: 100.0,
            fuga_memoria_porcentaje_maximo: 90.0,
            log_actividades: String::new(),
            ataques_activos: false,
            tiempo_inicio_ddos: None,
            tiempo_inicio_cpu: None,
            tiempo_inicio_memoria: None,
            cpu_spike_handle: None,
            memory_handle: None,
            ddos_handle: None,
            //Cpu
            cpu_monitor: CpuMonitor::new(),
            cpu_usage: 0.0,
            ultima_actualizacion_cpu: Instant::now(),
            //Memoria
            memoria_monitor: MemoriaInfo::new(),
            memoria_usage: 0.0,
            //Trafico
            monitor: MonitorRed::new(),
            trafico_actual: TraficoRed {
                recibido: 0,
                enviado: 0,
            },
        }
    }
}

impl AppState {
    fn iniciar_todo(&mut self) {
        if !self.activar_ddos && !self.activar_cpu_spike && !self.activar_fuga_memoria {
            self.log_actividades.push_str("[WARNING] No se seleccionó ningún ataque.\n");
            return;
        }

        self.ataques_activos = true;
        self.log_actividades.push_str("[INFO] Iniciando ataques seleccionados...\n");
        
        let cpu = if self.activar_cpu_spike {
            Some(self.cpu_spike_porcentaje)
        } else {
            None
        };

        let memoria = if self.activar_fuga_memoria {
            Some(self.fuga_memoria_porcentaje_maximo)
        } else {
            None
        };

        let red = if self.activar_ddos {
            Some(format!("{} solicitudes/s", self.ddos_solicitudes_por_segundo))
        } else {
            None
        };

        enviar_metricas(cpu, memoria, red, 5000);
        
       if self.activar_ddos {
            match self.ddos_solicitudes_por_segundo.parse::<usize>() {
                Ok(pps) => {
                    self.ddos_handle = Some(DDoS::new(pps));
                    self.tiempo_inicio_ddos = Some(Instant::now());
                    self.log_actividades.push_str(&format!(
                        "- DDoS: {} solicitudes/seg iniciado\n",
                        pps
                    ));
                }
                Err(_) => {
                    self.log_actividades.push_str("[ERROR] Valor inválido para solicitudes por segundo\n");
                }
            }
        }
        
        if self.activar_cpu_spike {
            self.cpu_spike_handle = Some(CpuSpike::new(self.cpu_spike_porcentaje));
            
            self.tiempo_inicio_cpu = Some(Instant::now());
            self.log_actividades.push_str(&format!(
                "- CPU Spike: {}%\n",
                self.cpu_spike_porcentaje
            ));

        }
        if self.activar_fuga_memoria {
            self.memory_handle = Some(MemoryLeak::new(self.fuga_memoria_porcentaje_maximo, true));

            self.tiempo_inicio_memoria = Some(Instant::now());
            self.log_actividades.push_str(&format!(
                "- Fuga de Memoria: {}%\n",
                self.fuga_memoria_porcentaje_maximo
            ));

        }
    }

    fn detener_todo(&mut self) {
        if self.ataques_activos {
            self.log_actividades.push_str("[INFO] Deteniendo todos los ataques...\n");
            self.ataques_activos = false;
            
            if self.tiempo_inicio_ddos.is_some() {
                if let Some(ddos)=self.ddos_handle.take() {ddos.detener();}

                let duracion = self.tiempo_inicio_ddos.unwrap().elapsed().as_secs();
                registrar_en_csv("DDoS", duracion);
                self.tiempo_inicio_ddos = None;
            }
            
            if self.tiempo_inicio_cpu.is_some() {
                //Deteniedo con la funcion del modulo
                if let Some(cpu_spike) = self.cpu_spike_handle.take() {cpu_spike.detener();}
                
                let duracion = self.tiempo_inicio_cpu.unwrap().elapsed().as_secs();
                registrar_en_csv("CPU Spike", duracion);
                self.tiempo_inicio_cpu = None;
            }
            if self.tiempo_inicio_memoria.is_some() {

                if let Some(memoria) = self.memory_handle.take() {memoria.detener();}

                let duracion = self.tiempo_inicio_memoria.unwrap().elapsed().as_secs();
                registrar_en_csv("Fuga de Memoria", duracion);
                self.tiempo_inicio_memoria = None;
            }

        } else {
            self.log_actividades.push_str("[INFO] No hay ataques activos.\n");
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Panel de Control de Ataques");
        ui.separator();

        ui.checkbox(&mut self.activar_ddos, "DDoS");
        ui.checkbox(&mut self.activar_cpu_spike, "CPU Spike");
        ui.checkbox(&mut self.activar_fuga_memoria, "Fuga de Memoria");

        ui.separator();
        ui.label("Configuración de Intensidad:");

        if self.activar_ddos {
            ui.horizontal(|ui| {
                ui.label("Solicitudes/seg:");
                ui.text_edit_singleline(&mut self.ddos_solicitudes_por_segundo);
            });
        }

        if self.activar_cpu_spike {
            ui.horizontal(|ui| {
                ui.label("CPU %:");
                ui.add(egui::Slider::new(&mut self.cpu_spike_porcentaje, 1.0..=100.0).suffix("%"));
            });
        }

        if self.activar_fuga_memoria {
            ui.horizontal(|ui| {
                ui.label("RAM %:");
                ui.add(egui::Slider::new(&mut self.fuga_memoria_porcentaje_maximo,1.0..=100.0,).suffix("%"));
            });
        }
   
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Iniciar Todo").clicked() {
                self.iniciar_todo();
            }
            if ui.button("Detener Todo").clicked() {
                self.detener_todo();
            }
            if ui.button("Generar datos sinteticos").clicked() {
            }
            if ui.button("Limpiar Log").clicked() {
                self.log_actividades.clear();
            }
        });

        ui.separator();
        ui.heading("Log de Actividades");
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            ui.label(&self.log_actividades);
        });

        ui.heading("Uso de CPU en Tiempo Real");
        ui.add(egui::ProgressBar::new(self.cpu_usage / 100.0)
            .text(format!("{:.1}%", self.cpu_usage)));
        ui.separator();

        ui.heading("Uso de Memoria en Tiempo Real");
        ui.add(egui::ProgressBar::new(self.memoria_usage / 100.0)
            .text(format!("{:.1}%", self.memoria_usage)));
        ui.separator();

        ui.heading("Monitoreo de Tráfico de Red");
            ui.label(format!(
                "Total recibido: {:.2} MB",
                self.trafico_actual.recibido as f64 / 1_048_576.0
            ));
            ui.label(format!(
                "Total enviado: {:.2} MB",
                self.trafico_actual.enviado as f64 / 1_048_576.0
            ));

        ui.separator();

    }
}

fn registrar_en_csv(tipo: &str, duracion: u64) {
    let fecha = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let linea = format!("{},{},{}s\n", fecha, tipo, duracion);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log_ataques.csv")
        .expect("No se pudo abrir el archivo CSV");

    file.write_all(linea.as_bytes())
        .expect("No se pudo escribir en el archivo CSV");
}


impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        
        // Actualizacion de el monitoreo
        if self.ultima_actualizacion_cpu.elapsed().as_secs_f32() >= 0.5 {
            self.cpu_usage = self.cpu_monitor.get_cpu_usage();
            self.ultima_actualizacion_cpu = Instant::now();
        }


        self.memoria_monitor.actualizar();
        self.memoria_usage = self.memoria_monitor.porcentaje_uso_memoria();    

        self.trafico_actual = self.monitor.actualizar();

        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });

        ctx.request_repaint();
    }
}

pub fn iniciar_gui() {
    let native_options = NativeOptions::default();
    if let Err(e) = eframe::run_native(
        "Simulador de Ataques",
        native_options,
        Box::new(|_cc| Box::new(AppState::default())),
    ) {
        eprintln!("Error al iniciar la GUI: {}", e);
    }
}
