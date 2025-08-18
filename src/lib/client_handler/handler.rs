use std::marker::PhantomData;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::client_handler::client_state::{ClientHandler, ClientStateAction};
use crate::msg::TcpMessage;

pub struct BaseClientHandler<S: ClientHandler> {
    stream: TcpStream,
    _state: PhantomData<S>,
}

impl<S: ClientHandler> BaseClientHandler<S> {
    pub fn new(stream: TcpStream) -> Self {
        println!("Client connected from: {}", stream.peer_addr().unwrap());
        BaseClientHandler {
            stream,
            _state: PhantomData,
        }
    }
    pub async fn handle(&mut self) {
        let mut buffer = [0; 512];
        // Loop to continuously read from the client stream asynchronously.
        loop {
            // `read_size` is the number of bytes read.
            // `await` pauses the execution of this task until data is available.
            let read_size = match self.stream.read(&mut buffer).await {
                Ok(size) if size == 0 => {
                    // If the size is 0, the client has disconnected.
                    println!(
                        "Client disconnected from: {}",
                        self.stream.peer_addr().unwrap()
                    );
                    return;
                }
                Ok(size) => size,
                Err(e) => {
                    // Handle potential read errors.
                    eprintln!("An error occurred while reading: {}", e);
                    return;
                }
            };

            // Convert the received bytes to Message and process it.
            let message = <S as ClientHandler>::Message::from_bytes(&buffer[..read_size]);
            println!("Received from client: {:?}", message);
            match <S as ClientHandler>::handle(&mut self.stream, message).await {
                ClientStateAction::Disconnect => {
                    println!(
                        "Client requested disconnect from: {}",
                        self.stream.peer_addr().unwrap()
                    );
                    return;
                }
                ClientStateAction::KeepAlive => {}
            };
        }
    }
}
