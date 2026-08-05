use serde::{Deserialize, Serialize};

use crate::{
    auth_kem::{PqcIdentity, PqcKeyMaterial},
    error::TimechainError,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {}
pub struct PqcHandshake {}
impl PqcHandshake {
    pub fn new(_: PqcKeyMaterial) -> Self {
        Self {}
    }
    pub fn initiate(&mut self) -> Result<HandshakeMessage, TimechainError> {
        Ok(HandshakeMessage {})
    }
    pub fn handle_hello(
        &mut self,
        _: HandshakeMessage,
    ) -> Result<HandshakeMessage, TimechainError> {
        Ok(HandshakeMessage {})
    }
    pub fn handle_response(
        &mut self,
        _: HandshakeMessage,
    ) -> Result<HandshakeMessage, TimechainError> {
        Ok(HandshakeMessage {})
    }
    pub fn handle_confirm(&mut self, _: HandshakeMessage) -> Result<(), TimechainError> {
        Ok(())
    }
    pub fn complete_initiator(&mut self) -> Result<PqcIdentity, TimechainError> {
        let (dsa_pk, _dsa_sk) = crate::pqc::generate_node_keys();
        let (kem_pk, _kem_sk) = pqcrypto_kyber::kyber1024::keypair();
        Ok(PqcIdentity::new(&dsa_pk, &kem_pk))
    }
}
