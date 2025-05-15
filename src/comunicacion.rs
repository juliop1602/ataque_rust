use std::{fs, net::UdpSocket, path::Path};
use serde_json::{json, Value};

pub fn enviar_metricas(cpu: Option<f32>, memoria: Option<f32>, red: Option<String>, puerto: u16) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("No se pudo crear el socket");
    let destino = format!("127.0.0.1:{}", puerto);

    let mut data = serde_json::Map::new();
    if let Some(cpu_val) = cpu {
        data.insert("cpu".to_string(), json!(cpu_val));
    }
    if let Some(mem_val) = memoria {
        data.insert("memory".to_string(), json!(mem_val));
    }
    if let Some(net_val) = red {
        data.insert("network".to_string(), json!(net_val));
    }

    // Agrega marca de tiempo
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    data.insert("timestamp".to_string(), json!(timestamp));

    let json_data = Value::Object(data.clone());

    // Enviar por UDP
    let _ = socket.send_to(json_data.to_string().as_bytes(), &destino);

    // Guardar en archivo historial
    let path = Path::new("historial_metricas.json");
    let mut historial = if path.exists() {
        let contenido = fs::read_to_string(path).unwrap_or("[]".to_string());
        serde_json::from_str::<Vec<Value>>(&contenido).unwrap_or_default()
    } else {
        Vec::new()
    };

    historial.push(json_data);

    let nuevo_contenido = serde_json::to_string_pretty(&historial).unwrap();
    fs::write(path, nuevo_contenido).expect("No se pudo escribir el historial");
}
