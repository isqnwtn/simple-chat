/*
 *  An implementation that can be used as a tcp connection handler
 */

use std::net::SocketAddr;
use std::sync::Arc;


use crate::{
    actor::{
        server_actor::ServerActorHandler,
    },
};

pub struct SingleConnectionHandler {}

pub struct SingleConnectionState {
    pub controller_handle: Arc<&'static ServerActorHandler>,
    pub addr: SocketAddr,
}

impl SingleConnectionState {
    pub fn new(
        controller_handle: Arc<&'static ServerActorHandler>,
        addr: SocketAddr,
    ) -> Self {
        Self {
            controller_handle,
            addr,
        }
    }
}
