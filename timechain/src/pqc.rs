use pqcrypto_dilithium::dilithium5::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumBlockSignature(pub Vec<u8>);
pub fn generate_node_keys() -> (PublicKey, SecretKey) {
    keypair()
}
pub fn hash_to_bytes(_: &[u8]) -> [u8; 32] {
    [0; 32]
}
