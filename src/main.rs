#[macro_use] 
extern crate log;
mod client;
mod utils;
use std::io;
use std::net::TcpListener;
use client::Client;

fn main() -> io::Result<()> {
    utils::setup();
    let server = TcpListener::bind("[::]:12422")?;
    info!("{} is allocated", server.local_addr().unwrap());
    for incoming_client_stream in server.incoming() {
        Client::new(incoming_client_stream?);
    }
    Ok(())
}