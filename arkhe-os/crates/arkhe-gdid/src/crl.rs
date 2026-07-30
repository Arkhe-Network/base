use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrlBundle {
    pub epoch: u64,
    pub revoked_count: u64,
    pub root_hash: [u8; 32],
    pub sparse_proof: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrlError {
    InvalidProof,
}

impl core::fmt::Display for CrlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidProof => write!(f, "invalid CRL proof"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CrlError {}
