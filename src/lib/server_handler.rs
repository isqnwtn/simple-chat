use tokio::io;
use tokio::net::TcpListener;

use crate::client_handler::handler::BaseClientHandler;
use crate::client_handler::msg_handler::MsgHandler;

pub struct ServerHandler {
    listener: TcpListener,
}

impl ServerHandler {
    pub async fn new() -> io::Result<Self> {
        // Bind the server to a local IP address and port asynchronously.
        let listener = TcpListener::bind("127.0.0.1:7878").await?;

        println!("Server listening on port 7878...");
        Ok(ServerHandler { listener })
    }
    pub async fn main_loop(&self) -> io::Result<()> {
        // Iterate over incoming connections asynchronously.
        // The `accept()` method blocks the task (not the thread) until a client connects.
        loop {
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    // When a connection is successfully established, spawn a new task.
                    // This allows the server to handle multiple clients at once without creating new OS threads.
                    // The `move` keyword ensures ownership of the stream is transferred to the new task.
                    tokio::spawn(async move {
                        let mut client_handler = BaseClientHandler::<MsgHandler>::new(stream);
                        client_handler.handle().await;
                    });
                }
                Err(e) => {
                    // Handle potential connection errors.
                    eprintln!("Failed to accept a connection: {}", e);
                }
            }
        }
    }
}
