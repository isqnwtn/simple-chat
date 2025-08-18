use tokio::io::AsyncWriteExt;

use tokio::net::TcpStream;

use crate::client_handler::client_state::{ClientHandler, ClientStateAction};

pub struct MsgHandler {}

impl MsgHandler {
    pub fn new() -> Self {
        MsgHandler {}
    }
}

impl ClientHandler for MsgHandler {
    type Message = String;
    async fn handle(stream: &mut TcpStream, _message: Self::Message) -> ClientStateAction {
        let response = "Server received your message!";
        if let Err(e) = stream.write_all(response.as_bytes()).await {
            eprintln!("An error occurred while writing: {}", e);
            return ClientStateAction::Disconnect;
        }
        match _message.as_str() {
            "disconnect" => {
                ClientStateAction::Disconnect
            }
            _ => ClientStateAction::KeepAlive,
        }
    }
}
