// src/audit.rs
//! Módulo de auditoria — cadeia de custódia imutável (HashChain linear)
//! NOTA: Esta implementação é uma cadeia linear de hashes, não uma Merkle tree.
//! Para Merkle tree completa com proofs de inclusão O(log n), usar rs_merkle.

use crate::{
    types::*, Result,
};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Invalid chain: hash mismatch at entry {0}")]
    InvalidChain(usize),
    #[error("Empty chain")]
    EmptyChain,
    #[error("Duplicate hash")]
    DuplicateHash,
}

#[derive(Debug, Clone, Default)]
pub struct AuditTrail {
    entries: Vec<AuditEntry>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, hash: Hash, timestamp: Timestamp) -> Result<&AuditEntry> {
        let parent_hash = self
            .entries
            .last()
            .map(|e| e.hash)
            .unwrap_or(Hash::ZERO);

        if self.entries.iter().any(|e| e.hash == hash) {
            return Err(AuditError::DuplicateHash.into());
        }

        let entry = AuditEntry {
            hash,
            parent_hash,
            timestamp,
        };

        self.entries.push(entry);

        info!(
            entry_index = self.entries.len() - 1,
            hash = %hash,
            parent = %parent_hash,
            "Audit entry appended"
        );

        Ok(self.entries.last().unwrap())
    }

    pub fn append_consensus(&mut self, result: &ConsensusResult) -> Result<&AuditEntry> {
        self.append(result.final_hash, result.epoch_id)
    }

    pub fn verify_chain(&self) -> Result<bool> {
        if self.entries.is_empty() {
            return Ok(true);
        }

        let mut expected_parent = Hash::ZERO;

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.parent_hash != expected_parent {
                return Err(AuditError::InvalidChain(i).into());
            }
            expected_parent = entry.hash;
        }

        Ok(true)
    }

    pub fn root_hash(&self) -> Hash {
        self.entries
            .last()
            .map(|e| e.hash)
            .unwrap_or(Hash::ZERO)
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}