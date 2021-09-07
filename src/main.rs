#[macro_use] 
extern crate log;
mod utils;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> io::Result<()> {
    utils::setup();
    // let server_listener = TcpListener::bind("0.0.0.0:2822")?;
    // let client_listener = TcpListener::bind("0.0.0.0:1411")?;
    // info!("hafnium is ready, client can connect client to {}", server_listener.local_addr()?);
    // for incoming_server_stream in server_listener.incoming() {
    //     let incoming_server_stream = incoming_server_stream?;
    //     let client_listener = client_listener.try_clone()?;
    //     info!("server {} connected", incoming_server_stream.peer_addr()?);
    //     thread::spawn(move || -> io::Result<()> {
    //         for incoming_client_steam in client_listener.incoming() {
    //             let incoming_server_stream = incoming_server_stream.try_clone()?;
    //             thread::spawn(move || {
    //                 let incoming_client_steam = incoming_client_steam?;
    //                 info!("client {} connected to server {}", incoming_client_steam.peer_addr()?, incoming_server_stream.peer_addr()?);
    //                 utils::forward_duplex_stream(incoming_server_stream, incoming_client_steam)
    //             });
    //         }
    //         Ok(())
    //     });
    // }
    let server = TcpListener::bind("[::]:25565")?;
    info!("{} is allocated", server.local_addr().unwrap());
    for incoming_client_stream in server.incoming() {
        let incoming_client_stream = incoming_client_stream.unwrap();
        info!("{} is connected", incoming_client_stream.local_addr().unwrap());
        thread::spawn(move || utils::forward_duplex_stream(incoming_client_stream, TcpStream::connect("10.122.205.194:25565")?) );
    }
    Ok(())
}