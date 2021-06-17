use std::io::Write;
use std::io::Read;
use std::net::{TcpListener, TcpStream};

fn forward_stream(server: &TcpStream, mut client: TcpStream) {
    let mut server = server.try_clone().unwrap();
    std::thread::spawn(move || {
        loop {
            let mut buffer: [u8; 2048] = [0; 2048];
            client.read(&mut buffer).unwrap();
            &server.write(&mut buffer).unwrap();
            println!("{:?} {}", client, String::from_utf8_lossy(&buffer));    
        }
    });
}

fn main() -> std::io::Result<()> {
    let server_listener = TcpListener::bind("0.0.0.0:8000")?;
    let client_listener = TcpListener::bind("0.0.0.0:80")?;

    for server in server_listener.incoming() {
        let server = &server.unwrap();
        for client in client_listener.incoming() {
            forward_stream(server, client.unwrap());
        }
    }
    Ok(())
}