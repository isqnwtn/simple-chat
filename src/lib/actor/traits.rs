use std::{fmt::Debug, future::Future, net::SocketAddr};
use tokio::net::TcpStream;

use crate::msg::TcpMessage;

pub trait ActorTrait: 'static {
    type InitParams: Send + 'static;
    // the state of the actor
    type State: Send + 'static;
    // startup code to run before the actor is created, helps create the starting
    // state of the actor
    fn startup(params: Self::InitParams) -> Self::State;

    // messages that can be send to an actor but won't receive any reply from
    // the actor
    type ActorMessage:  Debug + Send + 'static;
    // how to process such messages, this could modify the state of the actor
    fn handle(state: &mut Self::State, msg: Self::ActorMessage) -> impl Future<Output = ()> + Send;

    // send a kill signal to the actor, causing it to run the cleanup code and
    // drop all the receiver handles it has
    type PoisonPill: Default + Send + 'static;
    // cleanup code to run once the actor is ready to terminate
    fn cleanup(state: &mut Self::State, signal: Self::PoisonPill) -> ();
}

pub trait ServerActorTrait: ActorTrait {
    fn handle_connection(state: &mut Self::State, stream: TcpStream, addr: SocketAddr) -> ();
}

pub trait TcpConnectionHandlerActor : ActorTrait {
    type Message: TcpMessage;
    fn handle_message(state: &mut Self::State, msg: Self::Message) -> impl Future<Output = ()> + Send;
    fn handle_controller_message(state: &mut Self::State, msg: Self::Message, stream:&mut TcpStream) -> impl Future<Output = ()> + Send;
}
