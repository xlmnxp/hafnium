use daemonize::Daemonize;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

struct PortForwarder {
    listen_addr: String,
    target_addr: String,
}

impl PortForwarder {
    fn new(listen_addr: &str, target_addr: &str) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            target_addr: target_addr.to_string(),
        }
    }

    fn start(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.listen_addr)?;
        println!("Listening on {}", self.listen_addr);

        for stream in listener.incoming() {
            match stream {
                Ok(inbound) => {
                    let target_addr = self.target_addr.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(inbound, target_addr) {
                            eprintln!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
        Ok(())
    }
}

fn handle_connection(mut inbound: TcpStream, target_addr: String) -> std::io::Result<()> {
    let mut outbound = TcpStream::connect(target_addr)?;
    
    let mut inbound_clone = inbound.try_clone()?;
    let mut outbound_clone = outbound.try_clone()?;

    // Handle client -> target
    thread::spawn(move || {
        forward_data(&mut inbound, &mut outbound)
    });

    // Handle target -> client
    forward_data(&mut outbound_clone, &mut inbound_clone)
}

fn forward_data(from: &mut TcpStream, to: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = from.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(());
        }
        to.write_all(&buffer[..bytes_read])?;
    }
}

pub fn start_daemon() -> std::io::Result<()> {
    let daemonize = Daemonize::new()
        .pid_file("/var/run/hafnium.pid")
        .chown_pid_file(true)
        .working_directory("/var/lib/hafnium")
        .user("hafnium")
        .group("hafnium");

    match daemonize.start() {
        Ok(_) => {
            // Configure your port forwarder here
            let forwarder = PortForwarder::new("127.0.0.1:8080", "127.0.0.1:80");
            forwarder.start()?;
        }
        Err(e) => eprintln!("Error starting daemon: {}", e),
    }

    Ok(())
}

pub fn stop_daemon() -> std::io::Result<()> {
    let pid_file = "/var/run/hafnium.pid";

    if !std::path::Path::new(pid_file).exists() {
        eprintln!("PID file does not exist: {}", pid_file);
        return Ok(());
    }

    // read the PID from the file
    let pid = std::fs::read_to_string(pid_file)?.trim().parse::<i32>().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;

    // send a termination signal to the daemon process
    match nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), nix::sys::signal::Signal::SIGTERM) {
        Ok(_) => {
            println!("Daemon stopped successfully.");
            std::fs::remove_file(pid_file)?
        },
        Err(e) => eprintln!("Failed to stop daemon: {}", e),
    }

    Ok(())
}

pub fn restart_daemon() -> std::io::Result<()> {
    stop_daemon()?;
    start_daemon()
}