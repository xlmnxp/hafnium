use config_manager::get_public_key;

mod config_manager;
mod encryption;
mod server;
mod client;
mod daemon;

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

    /// Start the daemon
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
            daemon::start().await;
        },
        Commands::Connect(args) => {
            async {
                let mut client = HfClient::new();

                client.connect(args.remote_address.clone().expect("missing remote address"), args.remote_port.clone().expect("missing remote port")).await.expect("Cannot connect to remote server");
            }.await;
        }
    }
}
