use config_manager::get_public_key;

mod client;
mod config_manager;
mod daemon;
mod encryption;
mod handshake;
mod server;

use clap::{Args, Parser, Subcommand};
use client::HfClient;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new key pair
    GenKey,

    /// Get the public key
    GetSelf,

    /// Start the daemon
    #[command(name = "daemon", alias = "d", subcommand)]
    Daemon(DaemonCommands),

    /// Connect to a remote server
    Connect(ConnectArgs),

    /// List connected peers
    ListPeers,

    /// List active port forwardings
    ListPorts,

    /// Show daemon status
    Status,

    /// Manage trusted peers
    #[command(name = "peer", subcommand)]
    Peer(PeerCommands),
}

#[derive(Args)]
struct ConnectArgs {
    remote_address: Option<String>,
    remote_port: Option<u16>,
    server_public_key: Option<String>,
}

#[derive(Subcommand)]
enum DaemonCommands {
    /// Start the daemon
    Start,

    /// Stop the daemon
    Stop,

    /// Restart the daemon
    Restart,

    /// Show daemon status
    Status,
}

#[derive(Subcommand)]
enum PeerCommands {
    /// Add a trusted peer
    Add(AddPeerArgs),

    /// Remove a trusted peer
    Remove(RemovePeerArgs),

    /// List all trusted peers
    List,

    /// Show peer information
    Show(ShowPeerArgs),
}

#[derive(Args)]
struct AddPeerArgs {
    /// Public key of the peer to add
    public_key: String,
    
    /// Optional name/description for the peer
    name: Option<String>,
}

#[derive(Args)]
struct RemovePeerArgs {
    /// Public key of the peer to remove
    public_key: String,
}

#[derive(Args)]
struct ShowPeerArgs {
    /// Public key of the peer to show
    public_key: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GenKey => {
            config_manager::generate_key_pair_and_save();
        }
        Commands::GetSelf => {
            println!("Public Key: {}", get_public_key());
        }
        Commands::Daemon(DaemonCommands::Start) => {
            if let Err(e) = daemon::start_daemon().await {
                eprintln!("Failed to start daemon: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Daemon(DaemonCommands::Stop) => {
            if let Err(e) = daemon::stop_daemon().await {
                eprintln!("Failed to stop daemon: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Daemon(DaemonCommands::Restart) => {
            if let Err(e) = daemon::restart_daemon().await {
                eprintln!("Failed to restart daemon: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Daemon(DaemonCommands::Status) => {
            if let Err(e) = daemon::daemon_status().await {
                eprintln!("Failed to get daemon status: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Connect(args) => {
            let mut client = HfClient::new();

            client
                .connect(
                    args.remote_address.clone().expect("missing remote address"),
                    args.remote_port.clone().expect("missing remote port"),
                    args.server_public_key.clone().expect("missing server public key"),
                )
                .await
                .expect("Cannot connect to remote server");
        }
        Commands::ListPeers => {
            if let Err(e) = send_command_to_daemon("LIST peers").await {
                eprintln!("Failed to list peers: {}", e);
                std::process::exit(1);
            }
        }
        Commands::ListPorts => {
            if let Err(e) = send_command_to_daemon("LIST ports").await {
                eprintln!("Failed to list ports: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Status => {
            if let Err(e) = send_command_to_daemon("STATUS").await {
                eprintln!("Failed to get status: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Peer(PeerCommands::Add(args)) => {
            config_manager::insert_trusted_peer(&args.public_key);
            println!("Added trusted peer: {}", args.public_key);
        }
        Commands::Peer(PeerCommands::Remove(args)) => {
            config_manager::remove_trusted_peer(&args.public_key);
            println!("Removed trusted peer: {}", args.public_key);
        }
        Commands::Peer(PeerCommands::List) => {
            let peers = config_manager::get_trusted_peers();
            if peers.is_empty() {
                println!("No trusted peers configured");
            } else {
                println!("Trusted peers:");
                for (i, peer) in peers.iter().enumerate() {
                    println!("  {}. {}", i + 1, peer);
                }
            }
        }
        Commands::Peer(PeerCommands::Show(args)) => {
            let peers = config_manager::get_trusted_peers();
            if peers.contains(&args.public_key) {
                println!("Peer {} is trusted", args.public_key);
            } else {
                println!("Peer {} is not in trusted list", args.public_key);
            }
        }
    }
}

async fn send_command_to_daemon(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let mut stream = UnixStream::connect("/var/run/hafnium.sock").await?;
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    print!("{}", response);

    Ok(())
}
