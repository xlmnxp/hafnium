use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn forward_stream(server: &TcpStream, mut client: TcpStream) {
    let mut server = server.try_clone().unwrap();
    // full-duplex
    std::thread::spawn(move || {
        let mut buffer: [u8; 2048] = [0; 2048];
        let mut _server = server.try_clone().unwrap();
        let mut _client = client.try_clone().unwrap();
        std::thread::spawn(move || loop {
            if &_server.read(&mut buffer).unwrap() == &0 {
                break;
            }
            _client.write(&mut buffer).unwrap();
        });
        loop {
            buffer = [0; 2048];
            if client.read(&mut buffer).unwrap() == 0 {
                break
            }
            &server.write(&mut buffer);
        }
    });
}

fn main() -> std::io::Result<()> {
    let server_listener = TcpListener::bind("0.0.0.0:8000")?;
    let client_listener = TcpListener::bind("0.0.0.0:80")?;

    // FIXME:   when server disconnect and connected again,
    //          it cannot receive or send message to client
    for server in server_listener.incoming() {
        let server = server.unwrap();
        let client_listener = client_listener.try_clone().unwrap();
        std::thread::spawn(move || for client in client_listener.incoming() {
            forward_stream(&server, client.unwrap());
        });  
    }
    Ok(())
}
