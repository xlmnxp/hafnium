extern crate simplelog;
use simplelog::*;
use std::fs::File;
use std::io;
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;


pub fn setup() {
    let logger_file = File::create("hafnium.log").unwrap();
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Warn, Config::default(), logger_file.try_clone().unwrap()),
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), logger_file),
        ]
    ).unwrap();
}

// forward full-duplex
pub fn forward_duplex_stream(server: TcpStream, client: TcpStream) -> io::Result<()> {
    let server = Arc::new(server);
    let client = Arc::new(client);
    let mut server_rw = server.try_clone()?;
    let mut client_rw = client.try_clone()?;

    let forward_threads = vec![
        thread::spawn(move || {
            io::copy(&mut server.as_ref(), &mut client.as_ref()).unwrap()
        }),
        thread::spawn(move || {
            io::copy(&mut client_rw, &mut server_rw).unwrap()
        }),
    ];

    for thread in forward_threads {
        thread.join().unwrap();
    }

    Ok(())
}