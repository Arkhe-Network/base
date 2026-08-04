use crate::error::TimechainError;
use crate::handshake::HandshakeMessage;
use crate::retro::EchoSignal;
use crate::timechain::TimeBlock;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
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
