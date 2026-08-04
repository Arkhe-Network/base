use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{
    error::TimechainError, handshake::HandshakeMessage, retro::EchoSignal, timechain::TimeBlock,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Echo { echo: EchoSignal, block: TimeBlock },
    Handshake(HandshakeMessage),
}
pub struct PeerInfo {}
pub struct P2PNode {}
impl P2PNode {
    pub async fn new(_: SocketAddr, _: crate::mhd::PlasmaConfig) -> Result<Self, TimechainError> {
        Ok(Self {})
    }
    pub async fn initiate_handshake(&mut self, _: SocketAddr) -> Result<(), TimechainError> {
        Ok(())
    }
    pub async fn run(&mut self) -> Result<(), TimechainError> {
        Ok(())
    }
}
