// ============================================================================
// Timechain (Arkhe) — Ledger Topológico Quântico
// v0.4.0-pre-alpha | Architecture-Frozen
// ============================================================================

pub mod accelerate;
pub mod auth_kem;
pub mod error;
pub mod handshake;
pub mod mhd;
pub mod network;
pub mod observer;
pub mod pqc;
pub mod retro;
pub mod shadow;
pub mod storage;
pub mod svd_compress;
pub mod timechain;
pub mod utxo;

pub use error::TimechainError;
pub use mhd::{EvoField, PlasmaConfig, ReconnectionDetector};

pub type Result<T> = std::result::Result<T, TimechainError>;
