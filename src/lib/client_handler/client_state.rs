use tokio::net::TcpStream;
use crate::msg::TcpMessage;

pub enum ClientStateAction {
    Disconnect,
    KeepAlive,
}

pub trait ClientHandler : Send + Sync + 'static {
    type Message: TcpMessage;
    fn handle(tcp_stream: &mut TcpStream, message: Self::Message) -> impl std::future::Future<Output = ClientStateAction>;
}
