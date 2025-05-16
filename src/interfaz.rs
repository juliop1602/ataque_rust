use eframe::{egui, Frame, NativeOptions};
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use std::time::Instant;
use rand::Rng;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use egui::{Align, Layout, TopBottomPanel};

//Ataques
use crate::cpu_spike::CpuSpike;
use crate::memoria::MemoryLeak;


//Comunicacion
use crate::comunicacion::enviar_metricas;

//Monitoreo
use crate::monitor_cpu::CpuMonitor;
use crate::monitor_memoria::MemoriaInfo;

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
    //Botones
    activar_ddos: bool,
    activar_cpu_spike: bool,
    activar_fuga_memoria: bool,
    //Intensidad
    ddos_solicitudes_por_segundo: String,
    cpu_spike_porcentaje: f32,
    fuga_memoria_porcentaje_maximo: f32,
    log_actividades: String,
    ataques_activos: bool,
    //Tiempos de cada ataque
    tiempo_inicio_ddos: Option<Instant>,
    tiempo_inicio_cpu: Option<Instant>,
    tiempo_inicio_memoria: Option<Instant>,
    //Manipular ataques
    cpu_spike_handle: Option<CpuSpike>,
    memory_handle: Option<MemoryLeak>,
    //Monitorei del cpu
    cpu_monitor: CpuMonitor,
    cpu_usage: f32,
    ultima_actualizacion_cpu: Instant,
    //Monitoreo de la memoria
    memoria_monitor: MemoriaInfo,
    memoria_usage: f32,
    //Manejo de datos 
    generando_datos: bool,
    detener_datos_flag: Arc<AtomicBool>,
    handle_datos: Option<JoinHandle<()>>,
     pub modo_oscuro: bool,
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
            //Cpu
            cpu_monitor: CpuMonitor::new(),
            cpu_usage: 0.0,
            ultima_actualizacion_cpu: Instant::now(),
            //Memoria
            memoria_monitor: MemoriaInfo::new(),
            memoria_usage: 0.0,
            //Manejo de datos sinteticos
            generando_datos: false,
            detener_datos_flag: Arc::new(AtomicBool::new(false)),
            handle_datos: None,
            modo_oscuro: true,
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
        
       if self.activar_ddos {
            match self.ddos_solicitudes_por_segundo.parse::<usize>() {
                Ok(pps) => {
                    
                    self.tiempo_inicio_ddos = Some(Instant::now());
                    self.log_actividades.push_str(&format!(
                        "- DDoS UDP: {} solicitudes/seg iniciado\n",
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
                //Deteniedo con la funcion del modulo

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
                //Deteniedo con la funcion del modulo
                if let Some(memoria) = self.memory_handle.take() {memoria.detener();}

                let duracion = self.tiempo_inicio_memoria.unwrap().elapsed().as_secs();
                registrar_en_csv("Fuga de Memoria", duracion);
                self.tiempo_inicio_memoria = None;
            }

        } else {
            self.log_actividades.push_str("[INFO] No hay ataques activos.\n");
        }
    }
    fn iniciar_datos_sinteticos(&mut self) {
        if self.generando_datos {
            self.log_actividades.push_str("[INFO] Ya se están generando datos sintéticos.\n");
            return;
        }

        self.log_actividades.push_str("[INFO] Iniciando generación de datos sintéticos...\n");

        let detener_flag = Arc::clone(&self.detener_datos_flag);
        detener_flag.store(false, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            while !detener_flag.load(Ordering::SeqCst) {
                let (cpu_sintetico, memoria_sintetica, red_sintetica) = generar_datos_sinteticos();
                enviar_metricas(cpu_sintetico, memoria_sintetica, red_sintetica, 5000);
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
        });

        self.generando_datos = true;
        self.handle_datos = Some(handle);
    }

    fn detener_datos_sinteticos(&mut self) {
    if !self.generando_datos {
        self.log_actividades.push_str("[INFO] No hay datos sintéticos en ejecución.\n");
        return;
    }

    self.log_actividades.push_str("[INFO] Deteniendo generación de datos sintéticos...\n");
    self.detener_datos_flag.store(true, Ordering::SeqCst);

    if let Some(handle) = self.handle_datos.take() {
        let _ = handle.join(); // Espera a que el hilo termine
    }

    self.generando_datos = false;
}

    fn ui(&mut self, ui: &mut egui::Ui) {
        TopBottomPanel::top("top_bar").show(ui.ctx(), |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = if self.modo_oscuro { "Oscuro" } else { "Claro" };
                if ui.selectable_label(self.modo_oscuro, label).clicked() {
                    self.modo_oscuro = !self.modo_oscuro;
                }
    });
        });
        
        ui.heading("Panel de Control de Ataques");
        

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
            if ui.button("Generar Datos Sintéticos").clicked() {
                self.iniciar_datos_sinteticos();
            }
            if ui.button("Detener Datos Sintéticos").clicked() {
                self.detener_datos_sinteticos();
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


    }
}

fn generar_datos_sinteticos() -> (Option<f32>, Option<f32>, Option<String>) {
    let mut rng = rand::thread_rng();

    let cpu: Option<f32> = if rng.gen_bool(0.8) {
        Some(rng.gen_range(5.0..=100.0)as f32)
    } else {
        None
    };

    let memoria: Option<f32> = if rng.gen_bool(0.7) {
        Some(rng.gen_range(0.0..=90.0)as f32)
    } else {
        None
    };

    let red = if rng.gen_bool(0.6) {
    let solicitudes_por_segundo = rng.gen_range(5000..=20000);
    Some(format!("{} req/s", solicitudes_por_segundo))
    } else {
        None
    };


    (cpu, memoria, red)
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

        //Estilo
        if self.modo_oscuro {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        // Actualizacion de el monitoreo
        if self.ultima_actualizacion_cpu.elapsed().as_secs_f32() >= 0.5 {
            self.cpu_usage = self.cpu_monitor.get_cpu_usage();
            self.ultima_actualizacion_cpu = Instant::now();
        }

        self.memoria_monitor.actualizar();
        self.memoria_usage = self.memoria_monitor.porcentaje_uso_memoria();    


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
