use sysinfo::Networks;

pub struct TraficoRed {
    pub recibido: u64,
    pub enviado: u64,
}

pub struct MonitorRed {
    redes: Networks,
}

impl MonitorRed {
    pub fn new() -> Self {
        let mut redes = Networks::new_with_refreshed_list();
        redes.refresh();
        Self { redes }
    }

    pub fn actualizar(&mut self) -> TraficoRed {
        self.redes.refresh();

        let mut total_recibido = 0;
        let mut total_enviado = 0;

        for (_nombre, datos) in self.redes.iter() {
            total_recibido += datos.total_received();
            total_enviado += datos.total_transmitted();
        }

        TraficoRed {
            recibido: total_recibido,
            enviado: total_enviado,
        }
    }
}
