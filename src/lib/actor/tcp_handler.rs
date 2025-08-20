use crate::{
    actor::traits::{ActorTrait, TcpConnectionHandlerActor},
    msg::TcpMessage,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

/// The Actor struct, responsible for spawning the actor that receive the
/// messages and then handle them, the actor itself may have a state that can
/// be affected by the message
pub struct TcpActor<A: TcpConnectionHandlerActor> {
    // a handle to the receiver from mpsc::channel so that we can use it to
    // receive messages
    receiver: mpsc::Receiver<ControllerMessages<<A as TcpConnectionHandlerActor>::Message>>,
    // a receiver to handle poison pill
    poison_pill: mpsc::Receiver<<A as ActorTrait>::PoisonPill>,
    // the state of the actor that can be modified by the handle function
    state: <A as ActorTrait>::State,
    // a tcp stream
    stream: TcpStream,
}

pub enum ControllerMessages<A> {
    WriteStream(A),
    Null,
}

impl<A: TcpConnectionHandlerActor> TcpActor<A> {
    pub fn new(
        rx: mpsc::Receiver<ControllerMessages<<A as TcpConnectionHandlerActor>::Message>>,
        krx: mpsc::Receiver<<A as ActorTrait>::PoisonPill>,
        stream: TcpStream,
        init_params: <A as ActorTrait>::InitParams,
    ) -> Self {
        TcpActor {
            receiver: rx,
            poison_pill: krx,
            state: <A as ActorTrait>::startup(init_params),
            stream,
        }
    }
    pub async fn start(mut self) -> u8 {
        let mut buffer = [0; 512];
        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    match msg {
                        ControllerMessages::WriteStream(msg) => {
                            let bytes = msg.to_bytes();
                            if let Some(bytes) = bytes {
                                let _ = self.stream.write(&bytes).await;
                            } else {
                                eprintln!("failed to serialize message");
                            }
                        }
                        ControllerMessages::Null => {}
                    }
                }
                Some(p) = self.poison_pill.recv() => {
                    <A as ActorTrait>::cleanup(&mut self.state,p);
                    eprintln!("killing actor");
                    return 1;
                }
                Ok(size) = self.stream.read(&mut buffer) => {
                   if size == 0 {
                       break;
                   } else {
                       let _parsed = <A as TcpConnectionHandlerActor>::Message::from_bytes(&buffer[0..size]);
                       if let Some(parsed) = _parsed {
                           let _a = <A as TcpConnectionHandlerActor>::handle_message(&mut self.state, parsed).await;
                       } else {
                           eprintln!("failed to parse message");
                       }
                   }
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
pub struct TcpActorHandle<A: TcpConnectionHandlerActor> {
    // holds a handle to the sender from mpsc::channel
    id: mpsc::Sender<ControllerMessages<<A as TcpConnectionHandlerActor>::Message>>,
    // holds a handle to send poison pill
    kid: mpsc::Sender<<A as ActorTrait>::PoisonPill>,
}

impl<A: TcpConnectionHandlerActor> TcpActorHandle<A> {
    // when we create a new actor what we only return is the handle, during
    // it's creation we launch the actor and create a handle to it as well.
    pub fn new(size: usize, stream: TcpStream, init_params: <A as ActorTrait>::InitParams) -> Self {
        let (tx, rx): (
            mpsc::Sender<ControllerMessages<<A as TcpConnectionHandlerActor>::Message>>,
            mpsc::Receiver<ControllerMessages<<A as TcpConnectionHandlerActor>::Message>>,
        ) = mpsc::channel(size);
        let (ktx, krx): (
            mpsc::Sender<<A as ActorTrait>::PoisonPill>,
            mpsc::Receiver<<A as ActorTrait>::PoisonPill>,
        ) = mpsc::channel(1);

        let actor: TcpActor<A> = TcpActor::new(rx, krx, stream, init_params);
        tokio::spawn(async move {
            let res = actor.start().await;
            eprintln!("Actor exited with : {}", res);
        });
        let handle = TcpActorHandle { id: tx, kid: ktx };
        return handle;
    }
    pub async fn send(&self, msg: <A as TcpConnectionHandlerActor>::Message) -> () {
        if let Err(e) = self.id.send(ControllerMessages::WriteStream(msg)).await {
            eprintln!("{:?}", e);
        }
    }
    pub async fn terminate(&self, msg: <A as ActorTrait>::PoisonPill) -> () {
        if let Err(e) = self.kid.send(msg).await {
            eprintln!("{:?}", e);
        }
    }
}
