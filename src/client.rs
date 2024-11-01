use crate::server::{PortForwarding, Peer};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
pub struct HfClient {
    remote_peers: Vec<Peer>,
    forward_ports: Vec<PortForwarding>,
}

impl HfClient {
    pub fn new() -> Self {
        HfClient {
            forward_ports: Vec::new(),
            remote_peers: Vec::new(),
        }
    }

    pub async fn connect(mut self, remote_addr: String, remote_port: u16) {
        self.remote_peers.push(Peer {
            id: self.remote_peers.len() as u32,
            addr: remote_addr.clone(),
            port: remote_port.clone(),
            ports: Vec::new(),
        });

        let stream = tokio::net::TcpStream::connect(format!("{}:{}", remote_addr, remote_port)).await.unwrap();

        self.handle_connection(stream).await;
    }

    async fn handle_connection(self, mut socket: tokio::net::TcpStream) {
        tokio::spawn(async move {
            println!("Connected to {}", socket.peer_addr().unwrap());

            socket.write(b"Hello, world!").await.unwrap();

            loop {
                let mut buffer: [u8; 1024] = [0; 1024];

                let buffer_size = socket.read(&mut buffer).await.unwrap();

                if buffer_size == 0 {
                    break;
                }

                println!("Received: {}", String::from_utf8_lossy(&buffer));
            }
        });
    }
}