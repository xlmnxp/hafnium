use std::sync::{Arc, Mutex};
use tokio::{io::AsyncReadExt, net::TcpListener};

#[derive(Clone)]
pub struct PortForwarding {
    local_port: u16,
    remote_port: u16,
}

#[derive(Clone)]
pub struct Peer {
    pub addr: String,
    pub port: u16,
    pub ports: Arc<Mutex<Vec<PortForwarding>>>,
}

#[derive(Clone)]
pub struct HfServer {
    peers: Arc<Mutex<Vec<Peer>>>,
}

impl HfServer {
    pub fn new() -> Self {
        HfServer {
            peers: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn listener(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("0.0.0.0:37842").await.unwrap();
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            handle_connection(self, socket).await?;
        }
    }
}

async fn handle_connection(server: &mut HfServer, mut socket: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    server.peers.lock().expect("Cannot lock peers list").push(Peer {
        addr: socket.peer_addr()?.ip().to_string(),
        port: socket.peer_addr()?.port(),
        ports: Arc::new(Mutex::new(vec![])),
    });

    tokio::spawn(async move {
        println!("Received connection from {}", socket.peer_addr().expect("Cannot get peer address"));

        loop {
            let mut buffer: [u8; 1024] = [0; 1024];

            let buffer_size = socket.read(&mut buffer).await.expect("Cannot read from socket");

            if buffer_size == 0 {
                break;
            }

            println!("Received: {}", String::from_utf8_lossy(&buffer));
        }
    });

    Ok(())
}
