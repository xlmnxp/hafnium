use crate::server::{Peer, PortForwarding};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub struct HfClient {
    remote_peers: Arc<Mutex<Vec<Peer>>>,
    forward_ports: Arc<Mutex<Vec<PortForwarding>>>,
}

impl HfClient {
    pub fn new() -> Self {
        HfClient {
            forward_ports: Arc::new(Mutex::new(vec![])),
            remote_peers: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn connect(
        &mut self,
        remote_addr: String,
        remote_port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.remote_peers.lock().expect("Cannot lock peers list").push(Peer {
            addr: remote_addr.clone(),
            port: remote_port.clone(),
            ports: Arc::new(Mutex::new(vec![])),
        });

        let stream = tokio::net::TcpStream::connect(format!("{}:{}", remote_addr, remote_port))
            .await
            .unwrap();

        handle_connection(self, stream).await?;

        Ok(())
    }
}

async fn handle_connection(
    client: &mut HfClient,
    mut socket: tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
