use std::{clone::Clone, fmt::Debug, marker::Send, marker::Sync};

pub trait TcpMessage: Debug + Send + Sync + Clone + 'static {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}

impl TcpMessage for String {
    fn from_bytes(bytes: &[u8]) -> Self {
        String::from_utf8_lossy(bytes).to_string()
    }
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}
