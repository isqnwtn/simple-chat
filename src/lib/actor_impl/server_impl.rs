/*
 *  An implementation that can be used as a server
 */

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::OnceCell;
use tokio::task::JoinHandle;

use crate::actor::tcp_handler::TcpActorHandle;
use crate::actor::{
    server_actor::ServerActorHandler,
    traits::{ActorTrait, ServerActorTrait},
};
use crate::actor_impl::tcp_impl::{SingleConnectionHandler, SingleConnectionState};

pub struct CentralController {}

impl ActorTrait for CentralController {
    type InitParams = ();
    type State = ServerState;
    type ActorMessage = ConnectionMessage;
    type PoisonPill = ();
    type Answer = ();
    type Ask = ();

    fn startup(_params: Self::InitParams) -> Self::State {
        ServerState::new()
    }

    async fn handle(_state: &mut Self::State, _msg: Self::ActorMessage) -> () {
        println!("Received {:?}", _msg);
        for (_addr, handle) in _state._connections.iter() {
            if _msg._addr != *_addr {
                handle.send(_msg._message.clone()).await;
            }
        }
        ()
    }

    fn ask(_state: &mut Self::State, _msg: Self::Ask) -> () {
        ()
    }

    fn cleanup(_state: &mut Self::State, _signal: Self::PoisonPill) -> () {
        ()
    }
}

impl ServerActorTrait for CentralController {
    fn handle_connection(_state: &mut Self::State, stream: TcpStream, addr: SocketAddr) -> () {
        let this_handle = Arc::new(CENTRAL_CONTROLLER_HANDLE
            .get()
            .unwrap());
        println!("Connection request from : {:?}", addr);
        let this_connection : TcpActorHandle<SingleConnectionHandler> = TcpActorHandle::new(1024, stream, SingleConnectionState::new(this_handle, addr));
        _state._connections.insert(addr, this_connection);
    }
}

#[derive(Debug)]
pub struct ConnectionMessage {
    _message: String,
    _addr: SocketAddr,
}

impl ConnectionMessage {
    pub fn new(message: String, addr: SocketAddr) -> Self {
        Self { _message: message, _addr: addr }
    }
}

pub struct ServerState {
    _connections: HashMap<SocketAddr, TcpActorHandle<SingleConnectionHandler>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            _connections: HashMap::new(),
        }
    }
}

pub static CENTRAL_CONTROLLER_HANDLE: LazyLock<OnceCell<ServerActorHandler<CentralController>>> =
    LazyLock::new(|| OnceCell::new());

pub async fn init_central_controller(size: usize, listener: TcpListener) -> JoinHandle<()> {
    let (this_handle,join_handle): (ServerActorHandler<CentralController>,JoinHandle<()>) =
        ServerActorHandler::new(size, listener, ());
    CENTRAL_CONTROLLER_HANDLE
        .set(this_handle)
        .map_err(|_| "Failed to initialize central actor")
        .unwrap();
    join_handle
}
