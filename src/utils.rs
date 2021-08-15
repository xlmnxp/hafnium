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
pub fn forward_duplex_stream(first_stream: TcpStream, second_stream: TcpStream) -> io::Result<()> {
    let first_stream_reader = Arc::new(first_stream);
    let second_stream_reader = Arc::new(second_stream);
    let mut first_stream_writer = first_stream_reader.try_clone()?;
    let mut second_stream_writer = second_stream_reader.try_clone()?;

    let forward_threads = vec![
        thread::spawn(move || match io::copy(&mut first_stream_reader.as_ref(), &mut second_stream_writer) {
            _ => return
        }),
        thread::spawn(move || match io::copy(&mut second_stream_reader.as_ref(), &mut first_stream_writer) {
            _ => return
        }),
    ];

    for thread in forward_threads {
        thread.join().unwrap();
    }
    Ok(())
}