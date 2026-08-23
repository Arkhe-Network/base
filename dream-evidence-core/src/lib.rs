// src/lib.rs
//! # dream-evidence-core
//!
//! DEP v2.5 — Distributed Evidence Protocol with Byzantine fault tolerance.
//!
//! Este crate implementa o protocolo de evidência distribuída especificado em TLA+,
//! com verificação formal via Kani. O protocolo garante:
//!
//! - **Cadeia de custódia imutável**: audit trail com hashes encadeados
//! - **Consenso bizantino**: raters honestos superam maliciosos (Threshold > f)
//! - **Janela REM**: estimulação apenas dentro da janela válida
//! - **Segregação de deveres**: experimentadores não são raters
//! - **Verificação de assinaturas**: hashes criptográficos com Blake3
//!
//! # Arquitetura
//!
//! ```text
//! Acquisition → Epoch → Labels → Consensus → Audit Trail
//!      ↑           ↑         ↑          ↑
//!   Hardware    Storage   Rater      Byzantine
//!   REM Window            Process    Resistance
//! ```
//!
//! # Nota de Segurança
//!
//! A chave de hardware (`hardware_key`) é um **PLACEHOLDER**. Em produção,
//! deve ser substituída por integração com TPM 2.0 (tss-esapi) ou HSM PKCS#11.
//!
//! # Nota de Liveness
//!
//! Propriedades de liveness (eventualidade de consenso) requerem um scheduler
//! externo. Este crate fornece **garantias de safety apenas**.

pub mod types;
pub mod acquisition;
pub mod storage;
pub mod labeling;
pub mod consensus;
pub mod audit;
pub mod rem;
pub mod byzantine;

#[cfg(kani)]
pub mod kani_proofs;

pub use types::{
    Config, DataPacket, Epoch, RaterLabel, ConsensusResult, AuditEntry,
    Rater, Verdict, Hash, Signature, SensorType, Sample,
    PacketId, EpochId, Timestamp,
};

pub use acquisition::Acquisition;
pub use storage::Storage;
pub use labeling::{Labeling, LabelingError};
pub use consensus::{Consensus, ConsensusError};
pub use audit::{AuditTrail, AuditError};
pub use rem::{RemWindow, RemWindowError};
pub use byzantine::{ByzantineDetector, ByzantineError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Acquisition error: {0}")]
    Acquisition(#[from] acquisition::AcquisitionError),
    #[error("Storage error: {0}")]
    Storage(#[from] storage::StorageError),
    #[error("Labeling error: {0}")]
    Labeling(#[from] labeling::LabelingError),
    #[error("Consensus error: {0}")]
    Consensus(#[from] consensus::ConsensusError),
    #[error("Audit error: {0}")]
    Audit(#[from] audit::AuditError),
    #[error("REM window error: {0}")]
    Rem(#[from] rem::RemWindowError),
    #[error("Byzantine error: {0}")]
    Byzantine(#[from] byzantine::ByzantineError),
    #[error("Config error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Threshold not met: {0}")]
    ThresholdNotMet(String),
}