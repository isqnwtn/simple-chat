/*
 *  An implementation that can be used as a tcp connection handler
 */

use std::net::SocketAddr;
use std::sync::Arc;

use crate::{
    actor::{
        server_actor::ServerActorHandler,
        traits::{ActorTrait, TcpConnectionHandlerActor},
    },
    actor_impl::server_impl::{CentralController, ConnectionMessage}, msg::ClientMessage,
};

pub struct SingleConnectionHandler {}

impl ActorTrait for SingleConnectionHandler {
    type InitParams = SingleConnectionState;
    type State = SingleConnectionState;
    type ActorMessage = String;
    type PoisonPill = ();
    type Answer = ();
    type Ask = ();

    fn startup(params: Self::InitParams) -> Self::State {
        params
    }

    async fn handle(_state: &mut Self::State, _msg: Self::ActorMessage) -> () {
        ()
    }

    fn ask(_state: &mut Self::State, _msg: Self::Ask) -> () {
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
            ClientMessage::UserName(_name) => {
            }
            ClientMessage::Message(msg) => {
                state
                    .controller_handle
                    .send(ConnectionMessage::new(msg, state.addr))
                    .await;
            }
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
