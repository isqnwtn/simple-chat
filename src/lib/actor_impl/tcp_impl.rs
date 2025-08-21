/*
 *  An implementation that can be used as a tcp connection handler
 */

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    actor::{
        server_actor::ServerActorHandler,
        traits::{ActorTrait, TcpConnectionHandlerActor},
    },
    actor_impl::server_impl::{CentralController, ConnectionMessage},
    msg::ClientMessage,
};

pub struct SingleConnectionHandler {}

impl ActorTrait for SingleConnectionHandler {
    type InitParams = SingleConnectionState;
    type State = SingleConnectionState;
    type ActorMessage = String;
    type PoisonPill = ();

    fn startup(params: Self::InitParams) -> Self::State {
        params
    }

    async fn handle(_state: &mut Self::State, _msg: Self::ActorMessage) -> () {
        ()
    }

    fn cleanup(_state: &mut Self::State, _signal: Self::PoisonPill) -> () {
        ()
    }
}

impl TcpConnectionHandlerActor for SingleConnectionHandler {
    type Message = ClientMessage;

    async fn handle_message(state: &mut Self::State, msg: Self::Message) -> () {
        match msg {
            ClientMessage::UserName(_name) => {}
            ClientMessage::Message(msg) => {
                state
                    .controller_handle
                    .send(ConnectionMessage::UserMessage {
                        _addr: state.addr,
                        _message: msg,
                    })
                    .await;
            }
        }
    }
    async fn handle_controller_message(
        _state: &mut Self::State,
        _msg: Self::Message,
        _stream: &mut TcpStream,
    ) -> () {
        let bytes = <Self::Message as crate::msg::TcpMessage>::to_bytes(&_msg);
        if let Some(bytes) = bytes {
            let _ = _stream.write(&bytes).await;
        } else {
            eprintln!("failed to serialize message");
        }
    }
}

pub struct SingleConnectionState {
    controller_handle: Arc<&'static ServerActorHandler<CentralController>>,
    addr: SocketAddr,
}

impl SingleConnectionState {
    pub fn new(
        controller_handle: Arc<&'static ServerActorHandler<CentralController>>,
        addr: SocketAddr,
    ) -> Self {
        Self {
            controller_handle,
            addr,
        }
    }
}
