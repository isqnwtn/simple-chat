/*
 *  An implementation that can be used as a server
 */

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::LazyLock;

use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tokio::task::JoinHandle;

use crate::actor::server_actor::ServerActorHandler;
use crate::actor::tcp_handler::TcpActorHandle;

pub struct CentralController {}

#[derive(Debug)]
pub enum ConnectionMessage {
    UserMessage { addr: SocketAddr, message: String },
    UserCreationRequest { _addr: SocketAddr, _name: String },
    ConnectionDropped { addr: SocketAddr },
}

pub struct ServerState {
    pub connections: HashMap<SocketAddr, (TcpActorHandle, Option<String>)>,
    pub user_names: HashSet<String>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            user_names: HashSet::new(),
        }
    }
}

pub static CENTRAL_CONTROLLER_HANDLE: LazyLock<OnceCell<ServerActorHandler>> =
    LazyLock::new(OnceCell::new);

pub async fn init_central_controller(size: usize, listener: TcpListener) -> JoinHandle<()> {
    let (this_handle, join_handle): (ServerActorHandler, JoinHandle<()>) =
        ServerActorHandler::new(size, listener);
    CENTRAL_CONTROLLER_HANDLE
        .set(this_handle)
        .map_err(|_| "Failed to initialize central actor")
        .unwrap();
    join_handle
}
