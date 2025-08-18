
use crate::actor::traits::{ActorTrait, ServerActorTrait};
use tokio::{
    net::TcpListener,
    sync::mpsc, task::JoinHandle,
};

/// The Actor struct, responsible for spawning the actor that receive the
/// messages and then handle them, the actor itself may have a state that can
/// be affected by the message
pub struct ServerActor<A: ServerActorTrait> {
    // a handle to the receiver from mpsc::channel so that we can use it to
    // receive messages
    receiver: mpsc::Receiver<<A as ActorTrait>::ActorMessage>,
    // a receiver to handle poison pill
    poison_pill: mpsc::Receiver<<A as ActorTrait>::PoisonPill>,
    // the state of the actor that can be modified by the handle function
    state: <A as ActorTrait>::State,
    // a tcp stream
    listener: TcpListener,
}

impl<A: ServerActorTrait> ServerActor<A> {
    pub fn new(
        rx: mpsc::Receiver<<A as ActorTrait>::ActorMessage>,
        krx: mpsc::Receiver<<A as ActorTrait>::PoisonPill>,
        stream: TcpListener,
        init_params: <A as ActorTrait>::InitParams,
    ) -> Self {
        ServerActor {
            receiver: rx,
            poison_pill: krx,
            state: <A as ActorTrait>::startup(init_params),
            listener: stream,
        }
    }
    pub async fn start(mut self) -> u8 {
        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    <A as ActorTrait>::handle(&mut self.state, msg).await;
                }
                Some(p) = self.poison_pill.recv() => {
                    <A as ActorTrait>::cleanup(&mut self.state,p);
                    eprintln!("killing actor");
                    return 1;
                }
                Ok((stream, addr)) = self.listener.accept() => {
                    <A as ServerActorTrait>::handle_connection(&mut self.state, stream, addr);
                }
                else => {
                    eprintln!("all senders dropped");
                    <A as ActorTrait>::cleanup(
                        &mut self.state,
                        <A as ActorTrait>::PoisonPill::default()
                    );
                    break;
                }
            }
        }
        return 0;
    }
}

/// ActorHandle can be used to send messages to the respective actor.
#[derive(Clone)]
pub struct ServerActorHandler<A: ActorTrait> {
    // holds a handle to the sender from mpsc::channel
    id: mpsc::Sender<<A as ActorTrait>::ActorMessage>,
    // holds a handle to send poison pill
    kid: mpsc::Sender<<A as ActorTrait>::PoisonPill>,
}

impl<A: ServerActorTrait> ServerActorHandler<A> {
    // when we create a new actor what we only return is the handle, during
    // it's creation we launch the actor and create a handle to it as well.
    pub fn new(
        size: usize,
        listener: TcpListener,
        init_params: <A as ActorTrait>::InitParams,
    ) -> (Self, JoinHandle<()>) {
        let (tx, rx): (
            mpsc::Sender<<A as ActorTrait>::ActorMessage>,
            mpsc::Receiver<<A as ActorTrait>::ActorMessage>,
        ) = mpsc::channel(size);
        let (ktx, krx): (
            mpsc::Sender<<A as ActorTrait>::PoisonPill>,
            mpsc::Receiver<<A as ActorTrait>::PoisonPill>,
        ) = mpsc::channel(1);

        let actor: ServerActor<A> = ServerActor::new(rx, krx, listener, init_params);
        let join_handle = tokio::spawn(async move {
            let res = actor.start().await;
            eprintln!("Actor exited with : {}", res);
        });
        let handle = ServerActorHandler { id: tx, kid: ktx };
        return (handle,join_handle);
    }
    pub async fn send(&self, msg: <A as ActorTrait>::ActorMessage) -> () {
        if let Err(e) = self.id.send(msg).await {
            eprintln!("{:?}", e);
        }
    }
    pub async fn terminate(&self, msg: <A as ActorTrait>::PoisonPill) -> () {
        if let Err(e) = self.kid.send(msg).await {
            eprintln!("{:?}", e);
        }
    }
}
