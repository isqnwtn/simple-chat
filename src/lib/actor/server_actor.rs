use std::sync::Arc;

// use crate::actor::traits::{ActorTrait, ServerActorTrait};
use tokio::{net::TcpListener, sync::mpsc, task::JoinHandle};

use crate::{
    actor::tcp_handler::TcpActorHandle,
    actor_impl::{
        server_impl::{ConnectionMessage, ServerState, CENTRAL_CONTROLLER_HANDLE},
        tcp_impl::SingleConnectionState,
    },
    msg::ServerResponse,
};

/// The Actor struct, responsible for spawning the actor that receive the
/// messages and then handle them, the actor itself may have a state that can
/// be affected by the message
pub struct ServerActor {
    // a handle to the receiver from mpsc::channel so that we can use it to
    // receive messages
    receiver: mpsc::Receiver<ConnectionMessage>,
    // a receiver to handle poison pill
    poison_pill: mpsc::Receiver<()>,
    // the state of the actor that can be modified by the handle function
    state: ServerState,
    // a tcp stream
    listener: TcpListener,
}

impl ServerActor {
    pub fn new(
        rx: mpsc::Receiver<ConnectionMessage>,
        krx: mpsc::Receiver<()>,
        stream: TcpListener,
    ) -> Self {
        ServerActor {
            receiver: rx,
            poison_pill: krx,
            state: ServerState::new(),
            listener: stream,
        }
    }
    pub async fn start(mut self) -> u8 {
        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    println!("Received {:?}", msg);
                    match msg {
                        ConnectionMessage::UserMessage { addr, message } => {
                            if let Some((_,Some(sender_name))) = self.state.connections.get(&addr) {
                                for (_addr, (handle,_)) in self.state.connections.iter() {
                                    if *_addr != addr {
                                        handle.send(ServerResponse::Broadcast { username: format!("{:?}",sender_name), message: message.clone() }).await;
                                    }
                                }
                            }
                        },
                        ConnectionMessage::UserCreationRequest { _addr, _name } => {
                            if !self.state.user_names.contains(&_name) {
                                self.state.user_names.insert(_name.clone());
                                self.state.connections.entry(_addr).and_modify(|(_,name)| {
                                   *name = Some(_name);
                                });
                                if let Some((handle,_)) = self.state.connections.get_mut(&_addr) {
                                    handle.send(ServerResponse::UsernameAccepted).await;
                                }
                            } else {
                                if let Some((handle,_)) = self.state.connections.get_mut(&_addr) {
                                    handle.send(ServerResponse::UsernameExists).await;
                                    self.state.user_names.insert(_name.clone());
                                }
                            }
                        }
                        ConnectionMessage::ConnectionDropped { addr } => {
                            println!("Connection dropped : {:?}", addr);
                            if let Some((_,Some(name))) = self.state.connections.get(&addr) {
                                self.state.user_names.remove(name);
                            }
                            self.state.connections.remove(&addr);
                        }
                    }
                    ()
                }
                Some(_p) = self.poison_pill.recv() => {
                    eprintln!("killing actor");
                    return 1;
                }
                Ok((stream, addr)) = self.listener.accept() => {
                    // <A as ServerActorTrait>::handle_connection(&mut self.state, stream, addr);
                    let this_handle = Arc::new(CENTRAL_CONTROLLER_HANDLE
                        .get()
                        .unwrap());
                    println!("Connection request from : {:?}", addr);
                    let this_connection : TcpActorHandle = TcpActorHandle::new(1024, stream, SingleConnectionState::new(this_handle, addr));
                    self.state.connections.insert(addr, (this_connection,None));
                }
                else => {
                    eprintln!("all senders dropped");
                    // <A as ActorTrait>::cleanup(
                    //     &mut self.state,
                    //     <A as ActorTrait>::PoisonPill::default()
                    // );
                    break;
                }
            }
        }
        return 0;
    }
}

/// ActorHandle can be used to send messages to the respective actor.
#[derive(Clone)]
pub struct ServerActorHandler {
    // holds a handle to the sender from mpsc::channel
    id: mpsc::Sender<ConnectionMessage>,
    // holds a handle to send poison pill
    kid: mpsc::Sender<()>,
}

impl ServerActorHandler {
    // when we create a new actor what we only return is the handle, during
    // it's creation we launch the actor and create a handle to it as well.
    pub fn new(
        size: usize,
        listener: TcpListener,
        // init_params: <A as ActorTrait>::InitParams,
    ) -> (Self, JoinHandle<()>) {
        let (tx, rx): (
            mpsc::Sender<ConnectionMessage>,
            mpsc::Receiver<ConnectionMessage>,
        ) = mpsc::channel(size);
        let (ktx, krx): (mpsc::Sender<()>, mpsc::Receiver<()>) = mpsc::channel(1);

        let actor: ServerActor = ServerActor::new(rx, krx, listener);
        let join_handle = tokio::spawn(async move {
            let res = actor.start().await;
            eprintln!("Actor exited with : {}", res);
        });
        let handle = ServerActorHandler { id: tx, kid: ktx };
        return (handle, join_handle);
    }
    pub async fn send(&self, msg: ConnectionMessage) -> () {
        if let Err(e) = self.id.send(msg).await {
            eprintln!("{:?}", e);
        }
    }
    pub async fn terminate(&self, msg: ()) -> () {
        if let Err(e) = self.kid.send(msg).await {
            eprintln!("{:?}", e);
        }
    }
}
