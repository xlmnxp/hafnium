use crate::server::{Peer, PortForwarding};
use crate::handshake::{HandshakeClient, send_encrypted_message, receive_encrypted_message};
use p256::CompressedPoint;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
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
        expected_server_public_key: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Connect directly to the remote server
        let server_addr = format!("{}:{}", remote_addr, remote_port);
        let mut stream = TcpStream::connect(&server_addr).await?;
        
        println!("Connected to server at {}", server_addr);

        // Perform handshake
        let handshake_client = HandshakeClient::new();
        let handshake_result = handshake_client.perform_handshake(&mut stream).await?;
        
        // Verify server public key
        let received_server_key = base58::ToBase58::to_base58(CompressedPoint::from(handshake_result.peer_public_key).as_slice());
        if received_server_key != expected_server_public_key {
            return Err(format!("Server public key mismatch! Expected: {}, Received: {}", 
                              expected_server_public_key, received_server_key).into());
        }
        
        println!("Handshake successful! Server public key verified: {}", received_server_key);

        // Send a test message
        let test_message = "Hello from Hafnium client!";
        send_encrypted_message(&mut stream, test_message.as_bytes(), &handshake_result.peer_public_key).await?;
        println!("Sent encrypted message: {}", test_message);

        // Wait for response
        let response = receive_encrypted_message(&mut stream, &handshake_result.peer_public_key).await?;
        if let Ok(response_str) = String::from_utf8(response) {
            println!("Received encrypted response: {}", response_str);
        }

        Ok(())
    }
}

async fn handle_connection(
    _client: &mut HfClient,
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
