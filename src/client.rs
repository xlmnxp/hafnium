use std::{thread, io::{Result, copy}, sync::Arc, net::{TcpStream, Shutdown}};

pub struct Client {
    socket: TcpStream
}

impl Client {
    pub fn new(socket: TcpStream) -> Self {
        let client = Client {
            socket
        };
        // let mut buf = String::new();
        // // client.socket.read_to_string(&mut buf);
        // // println!("{}", buf);
        client.forward(TcpStream::connect("play.sy.sa:25565").unwrap()).unwrap()
    }

    pub fn forward(self, to: TcpStream) -> Result<Self> {
        let first_stream_reader = Arc::new(self.socket.try_clone()?);
        let second_stream_reader = Arc::new(to);
        let mut first_stream_writer = first_stream_reader.try_clone()?;
        let mut second_stream_writer = second_stream_reader.try_clone()?;
        
        info!("socket connect {:#?}", self.socket.peer_addr().unwrap());
        let forward_threads = vec![
            thread::spawn(move || match copy(&mut first_stream_reader.as_ref(), &mut second_stream_writer) {
                _ => {
                    // close the connection and I don't care if all closed or not
                    first_stream_reader.shutdown(Shutdown::Both).unwrap_or(());
                    second_stream_writer.shutdown(Shutdown::Both).unwrap_or(());
                }
            }),
            thread::spawn(move || match copy(&mut second_stream_reader.as_ref(), &mut first_stream_writer) {
                _ => {
                    // close the connection and I don't care if all closed or not
                    first_stream_writer.shutdown(Shutdown::Both).unwrap_or(());
                    second_stream_reader.shutdown(Shutdown::Both).unwrap_or(());
                }
            }),
        ];
    
        for thread in forward_threads {
            thread.join().unwrap();
        }

        Ok(self)
    }

    fn close(self) -> Result<()> {
        self.socket.shutdown(Shutdown::Both)
    }
}