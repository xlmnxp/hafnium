use config_manager::get_public_key;

mod config_manager;
mod encryption;
mod server;
mod client;

use {
    server::HfServer,
    client::HfClient,
};
use clap::{Args, Parser, Subcommand};

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

    /// Start the server
    Start,

    /// Connect to a remote server
    Connect(ConnectArgs)
}

#[derive(Args)]
struct ConnectArgs {
    remote_address: Option<String>,
    remote_port: Option<u16>,
    remote_public_key: Option<String>
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
        Commands::Start => {
            async {
                let mut server = HfServer::new();
    
                server.listener().await;
            }.await;
        },
        Commands::Connect(args) => {
            async {
                let client = HfClient::new();

                client.connect(args.remote_address.clone().expect("missing remote address"), args.remote_port.clone().expect("missing remote port")).await;
            }.await;
        }
    }
}
