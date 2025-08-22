/*
 *  An implementation that can be used as a server
 */

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::LazyLock;

use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tokio::task::JoinHandle;

use crate::actor::server_actor::ServerActorHandler;
use crate::actor::tcp_handler::TcpActorHandle;

pub struct CentralController {}

// impl ActorTrait for CentralController {
//     type InitParams = ();
//     type State = ServerState;
//     type ActorMessage = ConnectionMessage;
//     type PoisonPill = ();

//     fn startup(_params: Self::InitParams) -> Self::State {
//         ServerState::new()
//     }

//     async fn handle(_state: &mut Self::State, _msg: ConnectionMessage) -> () {
//         println!("Received {:?}", _msg);
//         match _msg {
//             ConnectionMessage::UserMessage { _addr, _message } => {
//                 for (_addr, handle) in _state._connections.iter() {
//                     if _addr != _addr {
//                         handle.send(ClientMessage::Message(_message.clone())).await;
//                     }
//                 }
//             }
//             ConnectionMessage::UserCreationRequest { _addr, _name } => {}
//         }
//         ()
//     }

//     fn cleanup(_state: &mut Self::State, _signal: Self::PoisonPill) -> () {
//         ()
//     }
// }

// impl ServerActorTrait for CentralController {
//     fn handle_connection(_state: &mut Self::State, stream: TcpStream, addr: SocketAddr) -> () {
//         let this_handle = Arc::new(CENTRAL_CONTROLLER_HANDLE.get().unwrap());
//         println!("Connection request from : {:?}", addr);
//         let this_connection: TcpActorHandle<SingleConnectionHandler> =
//             TcpActorHandle::new(1024, stream, SingleConnectionState::new(this_handle, addr));
//         _state._connections.insert(addr, this_connection);
//     }
// }

#[derive(Debug)]
pub enum ConnectionMessage {
    UserMessage { addr: SocketAddr, message: String },
    UserCreationRequest { _addr: SocketAddr, _name: String },
}

pub struct ServerState {
    pub _connections: HashMap<SocketAddr, TcpActorHandle>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            _connections: HashMap::new(),
        }
    }
}

pub static CENTRAL_CONTROLLER_HANDLE: LazyLock<OnceCell<ServerActorHandler>> =
    LazyLock::new(|| OnceCell::new());

pub async fn init_central_controller(size: usize, listener: TcpListener) -> JoinHandle<()> {
    let (this_handle, join_handle): (ServerActorHandler, JoinHandle<()>) =
        ServerActorHandler::new(size, listener);
    CENTRAL_CONTROLLER_HANDLE
        .set(this_handle)
        .map_err(|_| "Failed to initialize central actor")
        .unwrap();
    join_handle
}
