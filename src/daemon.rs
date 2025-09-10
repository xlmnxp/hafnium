use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;

use crate::handshake::{HandshakeServer, send_encrypted_message, receive_encrypted_message};
use crate::config_manager;
use p256::CompressedPoint;

#[derive(Debug, Clone)]
pub struct PortForwarding {
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String, // "tcp" or "udp"
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: String,
    pub addr: SocketAddr,
    pub public_key: String,
    pub shared_secret: [u8; 32],
    pub port_forwardings: Vec<PortForwarding>,
    pub last_seen: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub listen_addr: SocketAddr,
    pub unix_socket_path: String,
    pub pid_file: String,
    pub log_file: String,
    pub working_dir: String,
    pub user: String,
    pub group: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 37842),
            unix_socket_path: "/var/run/hafnium.sock".to_string(),
            pid_file: "/var/run/hafnium.pid".to_string(),
            log_file: "/var/log/hafnium.log".to_string(),
            working_dir: "/var/lib/hafnium".to_string(),
            user: "hafnium".to_string(),
            group: "hafnium".to_string(),
        }
    }
}

pub struct HafniumDaemon {
    config: DaemonConfig,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    port_forwarders: Arc<RwLock<HashMap<u16, TcpListener>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl HafniumDaemon {
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            port_forwarders: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Hafnium daemon...");

        // Create working directory if it doesn't exist
        fs::create_dir_all(&self.config.working_dir)
            .await
            .context("Failed to create working directory")?;

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start Unix socket listener for CLI communication
        self.start_unix_socket_listener().await?;

        // Start TCP listener for peer connections
        self.start_tcp_listener().await?;

        // Start cleanup task
        let peers = self.peers.clone();
        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                Self::cleanup_stale_peers(&peers).await;
            }
        });

        info!("Daemon started successfully");
        info!("Listening on TCP: {}", self.config.listen_addr);
        info!("Unix socket: {}", self.config.unix_socket_path);

        // Wait for shutdown signal
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Received shutdown signal");
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down");
            }
        }

        // Cleanup
        cleanup_handle.abort();

        info!("Daemon stopped");
        Ok(())
    }

    async fn start_unix_socket_listener(&self) -> Result<()> {
        // Remove existing socket file if it exists
        if Path::new(&self.config.unix_socket_path).exists() {
            fs::remove_file(&self.config.unix_socket_path).await?;
        }

        let listener = UnixListener::bind(&self.config.unix_socket_path)
            .context("Failed to bind Unix socket")?;

        // Set socket permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.config.unix_socket_path).await?.permissions();
            perms.set_mode(0o660);
            fs::set_permissions(&self.config.unix_socket_path, perms).await?;
        }

        let peers = self.peers.clone();
        let port_forwarders = self.port_forwarders.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let peers = peers.clone();
                        let port_forwarders = port_forwarders.clone();
                        let config = config.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_unix_connection(stream, peers, port_forwarders, config).await {
                                error!("Error handling Unix connection: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept Unix connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_tcp_listener(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.listen_addr)
            .await
            .context("Failed to bind TCP listener")?;

        let peers = self.peers.clone();
        let port_forwarders = self.port_forwarders.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New peer connection from: {}", addr);
                        
                        let peers = peers.clone();
                        let port_forwarders = port_forwarders.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_peer_connection(stream, addr, peers, port_forwarders).await {
                                error!("Error handling peer connection from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept peer connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_unix_connection(
        mut stream: UnixStream,
        peers: Arc<RwLock<HashMap<String, Peer>>>,
        port_forwarders: Arc<RwLock<HashMap<u16, TcpListener>>>,
        _config: DaemonConfig,
    ) -> Result<()> {
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        
        reader.read_line(&mut line).await?;
        let command = line.trim();

        debug!("Received command: {}", command);

        let response = match command.split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["LISTEN", port] => {
                let port: u16 = port.parse()?;
                Self::handle_listen_command(port, port_forwarders).await
            }
            ["CONNECT", addr, port] => {
                let addr = addr.to_string();
                let port: u16 = port.parse()?;
                Self::handle_connect_command(addr, port).await
            }
            ["LIST", "peers"] => {
                Self::handle_list_peers_command(peers).await
            }
            ["LIST", "ports"] => {
                Self::handle_list_ports_command(port_forwarders).await
            }
            ["STATUS"] => {
                Self::handle_status_command(peers, port_forwarders).await
            }
            _ => {
                format!("ERROR: Unknown command: {}\n", command)
            }
        };

        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }

    async fn handle_listen_command(
        port: u16,
        port_forwarders: Arc<RwLock<HashMap<u16, TcpListener>>>,
    ) -> String {
        let mut forwarders = port_forwarders.write().await;
        
        if forwarders.contains_key(&port) {
            return format!("ERROR: Port {} is already being forwarded\n", port);
        }

        match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
            Ok(listener) => {
                forwarders.insert(port, listener);
                info!("Started listening on port {}", port);
                format!("OK: Listening on port {}\n", port)
            }
            Err(e) => {
                error!("Failed to bind to port {}: {}", port, e);
                format!("ERROR: Failed to bind to port {}: {}\n", port, e)
            }
        }
    }

    async fn handle_connect_command(addr: String, port: u16) -> String {
        info!("Connect command received: {}:{}", addr, port);
        // TODO: Implement actual connection logic
        format!("OK: Connection initiated to {}:{}\n", addr, port)
    }

    async fn handle_list_peers_command(peers: Arc<RwLock<HashMap<String, Peer>>>) -> String {
        let peers_guard = peers.read().await;
        if peers_guard.is_empty() {
            return "No connected peers\n".to_string();
        }

        let mut response = String::new();
        for (id, peer) in peers_guard.iter() {
            response.push_str(&format!(
                "Peer {}: {} (last seen: {}s ago)\n",
                id,
                peer.addr,
                peer.last_seen.elapsed().as_secs()
            ));
        }
        response
    }

    async fn handle_list_ports_command(
        port_forwarders: Arc<RwLock<HashMap<u16, TcpListener>>>,
    ) -> String {
        let forwarders = port_forwarders.read().await;
        if forwarders.is_empty() {
            return "No ports being forwarded\n".to_string();
        }

        let mut response = String::new();
        for port in forwarders.keys() {
            response.push_str(&format!("Port {}: TCP forwarding active\n", port));
        }
        response
    }

    async fn handle_status_command(
        peers: Arc<RwLock<HashMap<String, Peer>>>,
        port_forwarders: Arc<RwLock<HashMap<u16, TcpListener>>>,
    ) -> String {
        let peers_guard = peers.read().await;
        let forwarders = port_forwarders.read().await;
        
        format!(
            "Status: Running\nConnected peers: {}\nActive port forwarders: {}\n",
            peers_guard.len(),
            forwarders.len()
        )
    }

    async fn handle_peer_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        peers: Arc<RwLock<HashMap<String, Peer>>>,
        _port_forwarders: Arc<RwLock<HashMap<u16, TcpListener>>>,
    ) -> Result<()> {
        let peer_id = format!("peer_{}", addr.port());
        info!("Peer {} connected from {}", peer_id, addr);

        // Perform handshake
        let handshake_server = HandshakeServer::new();
        let handshake_result = match timeout(Duration::from_secs(30), handshake_server.perform_handshake(&mut stream)).await {
            Ok(Ok(result)) => {
                info!("Handshake successful for peer {}", peer_id);
                result
            }
            Ok(Err(e)) => {
                error!("Handshake failed for peer {}: {}", peer_id, e);
                return Ok(());
            }
            Err(_) => {
                error!("Handshake timeout for peer {}", peer_id);
                return Ok(());
            }
        };

        // Verify peer is in trusted list
        let peer_public_key = base58::ToBase58::to_base58(CompressedPoint::from(handshake_result.peer_public_key).as_slice()).to_string();
        let trusted_peers = config_manager::get_trusted_peers();
        
        if !trusted_peers.contains(&peer_public_key) {
            error!("Peer {} not in trusted list, rejecting connection", peer_public_key);
            return Ok(());
        }

        info!("Peer {} is trusted, allowing connection", peer_public_key);

        // Create peer with handshake result
        let peer = Peer {
            id: peer_id.clone(),
            addr,
            public_key: peer_public_key,
            shared_secret: handshake_result.shared_secret,
            port_forwardings: vec![],
            last_seen: std::time::Instant::now(),
        };

        {
            let mut peers_guard = peers.write().await;
            peers_guard.insert(peer_id.clone(), peer);
        }

        info!("Peer {} authenticated and added to peer list", peer_id);

        // Handle encrypted peer communication
        loop {
            match timeout(Duration::from_secs(30), receive_encrypted_message(&mut stream, &handshake_result.peer_public_key)).await {
                Ok(Ok(data)) => {
                    debug!("Received {} bytes of encrypted data from peer {}", data.len(), peer_id);
                    
                    // Process the decrypted data
                    if let Ok(message) = String::from_utf8(data) {
                        info!("Peer {} sent: {}", peer_id, message);
                        
                        // Echo back the message (for testing)
                        if let Err(e) = send_encrypted_message(&mut stream, message.as_bytes(), &handshake_result.peer_public_key).await {
                            error!("Failed to send response to peer {}: {}", peer_id, e);
                            break;
                        }
                    }
                    
                    // Update last seen
                    {
                        let mut peers_guard = peers.write().await;
                        if let Some(peer) = peers_guard.get_mut(&peer_id) {
                            peer.last_seen = std::time::Instant::now();
                        }
                    }
                }
                Ok(Err(e)) => {
                    // if the error is EOF or early eof, the peer disconnected
                    if e.to_string().contains("EOF") || e.to_string().contains("early eof") {
                        info!("Peer {} disconnected", peer_id);
                        break;
                    }

                    error!("Error reading encrypted data from peer {}: {}", peer_id, e);
                    break;
                }
                Err(_) => {
                    warn!("Timeout reading from peer {}", peer_id);
                    break;
                }
            }
        }

        // Remove peer from list
        {
            let mut peers_guard = peers.write().await;
            peers_guard.remove(&peer_id);
        }

        info!("Peer {} disconnected", peer_id);
        Ok(())
    }

    async fn cleanup_stale_peers(peers: &Arc<RwLock<HashMap<String, Peer>>>) {
        let mut peers_guard = peers.write().await;
        let now = std::time::Instant::now();
        let timeout = Duration::from_secs(300); // 5 minutes

        peers_guard.retain(|id, peer| {
            if now.duration_since(peer.last_seen) > timeout {
                info!("Removing stale peer: {}", id);
                false
            } else {
                true
            }
        });
    }

    pub fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.try_send(());
        }
    }
}

// Daemon management functions
pub async fn start_daemon() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = DaemonConfig::default();
    let mut daemon = HafniumDaemon::new(config);
    
    // Write PID file
    let pid = std::process::id();
    fs::write(&daemon.config.pid_file, pid.to_string()).await?;
    
    daemon.start().await?;
    
    // Cleanup PID file
    let _ = fs::remove_file(&daemon.config.pid_file).await;
    
    Ok(())
}

pub async fn stop_daemon() -> Result<()> {
    let pid_file = "/var/run/hafnium.pid";

    if !Path::new(pid_file).exists() {
        println!("PID file does not exist: {}", pid_file);
        return Ok(());
    }

    let pid_str = fs::read_to_string(pid_file).await?;
    let pid: i32 = pid_str.trim().parse()?;

    match nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(pid),
        nix::sys::signal::Signal::SIGTERM,
    ) {
        Ok(_) => {
            println!("Daemon stopped successfully.");
            fs::remove_file(pid_file).await?;
        }
        Err(e) => {
            eprintln!("Failed to stop daemon: {}", e);
        }
    }

    Ok(())
}

pub async fn restart_daemon() -> Result<()> {
    stop_daemon().await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    start_daemon().await
}

pub async fn daemon_status() -> Result<()> {
    let pid_file = "/var/run/hafnium.pid";

    if !Path::new(pid_file).exists() {
        println!("Daemon is not running (no PID file found)");
        return Ok(());
    }

    let pid_str = fs::read_to_string(pid_file).await?;
    let pid: i32 = pid_str.trim().parse()?;

    match nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(pid),
        nix::sys::signal::Signal::SIGUSR1, // Use SIGUSR1 for status check
    ) {
        Ok(_) => {
            println!("Daemon is running (PID: {})", pid);
        }
        Err(_) => {
            println!("Daemon is not running (process not found)");
            let _ = fs::remove_file(pid_file).await;
        }
    }

    Ok(())
}