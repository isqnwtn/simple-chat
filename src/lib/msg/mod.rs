use std::{clone::Clone, fmt::Debug, marker::Send, marker::Sync};
use serde::{Deserialize, Serialize};

pub trait TcpMessage: Debug + Send + Sync + Clone + 'static {
    fn from_bytes(bytes: &[u8]) -> Option<Self>;
    fn to_bytes(&self) -> Option<Vec<u8>>;
}

impl TcpMessage for String {
    fn from_bytes(bytes: &[u8]) -> Option<String> {
        Some(String::from_utf8_lossy(bytes).to_string())
    }
    fn to_bytes(&self) -> Option<Vec<u8>> {
        Some(self.as_bytes().to_vec())
    }
}


#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ClientMessage {
    UserName(String),
    Message(String),
}

impl TcpMessage for ClientMessage {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
    fn to_bytes(&self) -> Option<Vec<u8>> {
        bincode::serialize(&self).ok()
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ServerResponse {
    Broadcast{username: String, message: String},
    ConnectionRefused
}

impl TcpMessage for ServerResponse {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
    fn to_bytes(&self) -> Option<Vec<u8>> {
        bincode::serialize(&self).ok()
    }
}
