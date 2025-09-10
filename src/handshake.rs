use anyhow::{Context, Result};
use log::{debug, info};
use p256::PublicKey;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::encryption::{self, EphemeralSecret, encrypt, decrypt};

#[derive(Debug, Serialize, Deserialize)]
pub enum HandshakeMessage {
    Helo {
        public_key: Vec<u8>,
        ephemeral_public_key: Vec<u8>,
    },
    Ehlo {
        public_key: Vec<u8>,
        ephemeral_public_key: Vec<u8>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug)]
pub struct HandshakeResult {
    pub peer_public_key: PublicKey,
    pub shared_secret: [u8; 32],
    pub is_authenticated: bool,
}

pub struct HandshakeClient {
    private_key: EphemeralSecret,
    ephemeral_secret: EphemeralSecret,
}

impl HandshakeClient {
    pub fn new() -> Self {
        Self {
            private_key: encryption::get_private_key(),
            ephemeral_secret: encryption::generate_key_pair(),
        }
    }

    pub async fn perform_handshake(&self, mut stream: &mut TcpStream) -> Result<HandshakeResult> {
        info!("Starting client handshake");

        // Send HELO message
        let helo_msg = HandshakeMessage::Helo {
            public_key: self.private_key.public_key().to_sec1_bytes().to_vec(),
            ephemeral_public_key: self.ephemeral_secret.public_key().to_sec1_bytes().to_vec(),
        };

        self.send_message(&mut stream, &helo_msg).await?;
        info!("Sent HELO message");

        // Wait for EHLO response
        let ehlo_msg = self.receive_message(&mut stream).await?;
        info!("Received EHLO response");

        let (server_public_key, _server_ephemeral_public_key) = match ehlo_msg {
            HandshakeMessage::Ehlo {
                public_key,
                ephemeral_public_key,
            } => {
                let server_pubkey = PublicKey::from_sec1_bytes(&public_key)
                    .context("Invalid server public key")?;
                let server_ephemeral_pubkey = PublicKey::from_sec1_bytes(&ephemeral_public_key)
                    .context("Invalid server ephemeral public key")?;
                (server_pubkey, server_ephemeral_pubkey)
            }
            HandshakeMessage::Error { message } => {
                return Err(anyhow::anyhow!("Server error: {}", message));
            }
            _ => {
                return Err(anyhow::anyhow!("Unexpected message type"));
            }
        };

        // Compute shared secret using the static key
        let shared_secret = encryption::get_shared_secret(&server_public_key);

        info!("Handshake completed successfully");
        Ok(HandshakeResult {
            peer_public_key: server_public_key,
            shared_secret,
            is_authenticated: true,
        })
    }

    async fn send_message(&self, stream: &mut TcpStream, message: &HandshakeMessage) -> Result<()> {
        let serialized = bincode::serialize(message)
            .context("Failed to serialize handshake message")?;
        
        let length = serialized.len() as u32;
        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&serialized).await?;
        stream.flush().await?;
        
        debug!("Sent handshake message: {:?}", message);
        Ok(())
    }

    async fn receive_message(&self, stream: &mut TcpStream) -> Result<HandshakeMessage> {
        // Read message length
        let mut length_bytes = [0u8; 4];
        stream.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Read message data
        let mut data = vec![0u8; length];
        stream.read_exact(&mut data).await?;

        let message = bincode::deserialize(&data)
            .context("Failed to deserialize handshake message")?;
        
        debug!("Received handshake message: {:?}", message);
        Ok(message)
    }
}

pub struct HandshakeServer {
    private_key: EphemeralSecret,
    ephemeral_secret: EphemeralSecret,
}

impl HandshakeServer {
    pub fn new() -> Self {
        Self {
            private_key: encryption::get_private_key(),
            ephemeral_secret: encryption::generate_key_pair(),
        }
    }

    pub async fn perform_handshake(&self, mut stream: &mut TcpStream) -> Result<HandshakeResult> {
        info!("Starting server handshake");

        // Wait for HELO message
        let helo_msg = self.receive_message(&mut stream).await?;
        info!("Received HELO message");

        let (client_public_key, _client_ephemeral_public_key) = match helo_msg {
            HandshakeMessage::Helo {
                public_key,
                ephemeral_public_key,
            } => {
                let client_pubkey = PublicKey::from_sec1_bytes(&public_key)
                    .context("Invalid client public key")?;
                let client_ephemeral_pubkey = PublicKey::from_sec1_bytes(&ephemeral_public_key)
                    .context("Invalid client ephemeral public key")?;
                (client_pubkey, client_ephemeral_pubkey)
            }
            _ => {
                let error_msg = HandshakeMessage::Error {
                    message: "Expected HELO message".to_string(),
                };
                self.send_message(&mut stream, &error_msg).await?;
                return Err(anyhow::anyhow!("Expected HELO message"));
            }
        };

        // Send EHLO response
        let ehlo_msg = HandshakeMessage::Ehlo {
            public_key: self.private_key.public_key().to_sec1_bytes().to_vec(),
            ephemeral_public_key: self.ephemeral_secret.public_key().to_sec1_bytes().to_vec(),
        };

        self.send_message(&mut stream, &ehlo_msg).await?;
        info!("Sent EHLO response");

        // Compute shared secret using the static key
        let shared_secret = encryption::get_shared_secret(&client_public_key);

        info!("Handshake completed successfully");
        Ok(HandshakeResult {
            peer_public_key: client_public_key,
            shared_secret,
            is_authenticated: true,
        })
    }

    async fn send_message(&self, stream: &mut TcpStream, message: &HandshakeMessage) -> Result<()> {
        let serialized = bincode::serialize(message)
            .context("Failed to serialize handshake message")?;
        
        let length = serialized.len() as u32;
        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&serialized).await?;
        stream.flush().await?;
        
        debug!("Sent handshake message: {:?}", message);
        Ok(())
    }

    async fn receive_message(&self, stream: &mut TcpStream) -> Result<HandshakeMessage> {
        // Read message length
        let mut length_bytes = [0u8; 4];
        stream.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Read message data
        let mut data = vec![0u8; length];
        stream.read_exact(&mut data).await?;

        let message = bincode::deserialize(&data)
            .context("Failed to deserialize handshake message")?;
        
        debug!("Received handshake message: {:?}", message);
        Ok(message)
    }
}

// Helper functions for encrypted communication after handshake
pub async fn send_encrypted_message(
    stream: &mut TcpStream,
    message: &[u8],
    peer_public_key: &PublicKey,
) -> Result<()> {
    let shared_secret = encryption::get_shared_secret(peer_public_key);
    let encrypted = encrypt(message, &shared_secret).await?;
    let length = encrypted.len() as u32;
    
    stream.write_all(&length.to_be_bytes()).await?;
    stream.write_all(&encrypted).await?;
    stream.flush().await?;
    
    debug!("Sent encrypted message ({} bytes)", encrypted.len());
    Ok(())
}

pub async fn receive_encrypted_message(
    stream: &mut TcpStream,
    peer_public_key: &PublicKey,
) -> Result<Vec<u8>> {
    // Read message length
    let mut length_bytes = [0u8; 4];
    stream.read_exact(&mut length_bytes).await?;
    let length = u32::from_be_bytes(length_bytes) as usize;

    // Read encrypted data
    let mut encrypted_data = vec![0u8; length];
    stream.read_exact(&mut encrypted_data).await?;

    let shared_secret = encryption::get_shared_secret(peer_public_key);
    let decrypted = decrypt(&encrypted_data, &shared_secret).await?;
    debug!("Received encrypted message ({} bytes)", decrypted.len());
    Ok(decrypted)
}
