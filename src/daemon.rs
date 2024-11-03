use tokio::net::UnixListener;
use tokio::io::AsyncReadExt;
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = std::path::Path::new("./hafnium.sock");
    let listener = UnixListener::bind(socket_path).expect("Cannot bind to socket");

    if socket_path.exists() {
        std::fs::remove_file(socket_path).expect("Cannot remove socket file");
    }
    
    loop {
        let (socket, _) = listener.accept().await.expect("Cannot accept connection");
        handle_connection(socket).await.expect("Cannot handle connection");
    }
}

async fn handle_connection(mut socket: tokio::net::UnixStream) -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        println!("Received connection from {:?}", socket.peer_addr().expect("Cannot get peer address"));

        loop {
            let mut buffer: [u8; 1024] = [0; 1024];

            let buffer_size = socket.read(&mut buffer).await.expect("Cannot read from socket");

            if buffer_size == 0 {
                break;
            }

            println!("Received: {}", String::from_utf8_lossy(&buffer));
        }
    });

    Ok(())
}