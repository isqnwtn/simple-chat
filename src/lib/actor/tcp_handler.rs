use crate::{
    actor_impl::{server_impl::ConnectionMessage, tcp_impl::SingleConnectionState},
    msg::{ClientMessage, TcpMessage},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

/// The Actor struct, responsible for spawning the actor that receive the
/// messages and then handle them, the actor itself may have a state that can
/// be affected by the message
pub struct TcpActor {
    // a handle to the receiver from mpsc::channel so that we can use it to
    // receive messages
    receiver: mpsc::Receiver<ControllerMessages>,
    // a receiver to handle poison pill
    poison_pill: mpsc::Receiver<()>,
    // the state of the actor that can be modified by the handle function
    state: SingleConnectionState,
    // a tcp stream
    stream: TcpStream,
}

pub enum ControllerMessages {
    WriteStream(ClientMessage),
    Null,
}

impl TcpActor {
    pub fn new(
        rx: mpsc::Receiver<ControllerMessages>,
        krx: mpsc::Receiver<()>,
        stream: TcpStream,
        init_params: SingleConnectionState,
    ) -> Self {
        TcpActor {
            receiver: rx,
            poison_pill: krx,
            state: init_params,
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
                            // let _ = <A as TcpConnectionHandlerActor>::handle_controller_message(&mut self.state, msg, &mut self.stream).await;
                            let bytes = msg.to_bytes();
                            if let Some(bytes) = bytes {
                                if let Err(e) = self.stream.write_all(&bytes).await {
                                    eprintln!("failed to write to addr: {}, error: {}", self.state.addr, e);
                                } else {
                                    println!("TCP handler {} writing to stream, msg: {:?}",self.state.addr,msg);
                                }
                            } else {
                                eprintln!("failed to serialize message");
                            }
                        }
                        ControllerMessages::Null => {}
                    }
                }
                Some(_p) = self.poison_pill.recv() => {
                    // <A as ActorTrait>::cleanup(&mut self.state,p);
                    eprintln!("killing actor");
                    return 1;
                }
                Ok(size) = self.stream.read(&mut buffer) => {
                   if size == 0 {
                       break;
                   } else {
                       let _parsed = ClientMessage::from_bytes(&buffer[0..size]);
                       if let Some(parsed) = _parsed {
                           // let _a = <A as TcpConnectionHandlerActor>::handle_message(&mut self.state, parsed).await;
                           match parsed {
                               ClientMessage::UserName(_name) => {}
                               ClientMessage::Message(msg) => {
                                   self.state
                                       .controller_handle
                                       .send(ConnectionMessage::UserMessage {
                                           addr: self.state.addr,
                                           message: msg,
                                       })
                                       .await;
                               }
                           }
                       } else {
                           eprintln!("failed to parse message");
                       }
                   }
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
pub struct TcpActorHandle {
    // holds a handle to the sender from mpsc::channel
    id: mpsc::Sender<ControllerMessages>,
    // holds a handle to send poison pill
    kid: mpsc::Sender<()>,
}

impl TcpActorHandle {
    // when we create a new actor what we only return is the handle, during
    // it's creation we launch the actor and create a handle to it as well.
    pub fn new(size: usize, stream: TcpStream, init_params: SingleConnectionState) -> Self {
        let (tx, rx): (
            mpsc::Sender<ControllerMessages>,
            mpsc::Receiver<ControllerMessages>,
        ) = mpsc::channel(size);
        let (ktx, krx): (mpsc::Sender<()>, mpsc::Receiver<()>) = mpsc::channel(1);

        let actor: TcpActor = TcpActor::new(rx, krx, stream, init_params);
        tokio::spawn(async move {
            let res = actor.start().await;
            eprintln!("Actor exited with : {}", res);
        });
        let handle = TcpActorHandle { id: tx, kid: ktx };
        return handle;
    }
    pub async fn send(&self, msg: ClientMessage) -> () {
        if let Err(e) = self.id.send(ControllerMessages::WriteStream(msg)).await {
            eprintln!("{:?}", e);
        }
    }
    pub async fn terminate(&self, msg: ()) -> () {
        if let Err(e) = self.kid.send(msg).await {
            eprintln!("{:?}", e);
        }
    }
}
