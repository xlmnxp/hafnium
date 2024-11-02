use tokio::{io::AsyncReadExt, net::TcpListener};

use crate::encryption;

pub struct PortForwarding {
    local_port: u16,
    remote_port: u16,
}

pub struct Peer {
    pub id: u32,
    pub addr: String,
    pub port: u16,
    pub ports: Vec<PortForwarding>,
}

pub struct HfServer {
    peers: Vec<Peer>,
}

impl HfServer {
    pub fn new() -> Self {
        HfServer {
            peers: Vec::new(),
        }
    }

    pub async fn listener(&mut self) {
        let listener = TcpListener::bind("0.0.0.0:37842").await.unwrap();
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            self.handle_connection(socket).await;
        }
    }

    async fn handle_connection(&mut self, mut socket: tokio::net::TcpStream) {
        self.peers.push(Peer {
            id: self.peers.len() as u32,
            addr: socket.peer_addr().unwrap().ip().to_string(),
            port: socket.peer_addr().unwrap().port(),
            ports: Vec::new(),
        });

        tokio::spawn(async move {
            println!("Received connection from {}", socket.peer_addr().unwrap());

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