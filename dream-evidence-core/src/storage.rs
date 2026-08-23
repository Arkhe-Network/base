// src/storage.rs
//! Módulo de armazenamento de epochs

use crate::{
    types::*, Result,
};
use std::collections::HashSet;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Epoch already exists: {0}")]
    EpochAlreadyExists(EpochId),
    #[error("Epoch not found: {0}")]
    EpochNotFound(EpochId),
    #[error("Invalid packets: {0}")]
    InvalidPackets(String),
    #[error("Duplicate packet ID in epoch")]
    DuplicatePacketId,
    #[error("Epoch limit reached: {0}")]
    EpochLimitReached(EpochId),
}

#[derive(Debug, Clone, Default)]
pub struct Storage {
    epochs: Vec<Epoch>,
    next_id: EpochId,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            epochs: Vec::new(),
            next_id: 0,
        }
    }

    pub fn close_epoch(
        &mut self,
        start_ts: Timestamp,
        packets: Vec<DataPacket>,
        config: &Config,
    ) -> Result<Epoch> {
        if self.next_id >= config.max_epoch_id {
            return Err(StorageError::EpochLimitReached(self.next_id).into());
        }

        if packets.is_empty() || packets.len() > config.max_packets_per_epoch {
            return Err(StorageError::InvalidPackets(
                format!("packets len={} not in [1, {}]", packets.len(), config.max_packets_per_epoch)
            ).into());
        }

        if !DataPacket::is_sorted_by_time(&packets) {
            return Err(StorageError::InvalidPackets("packets not sorted by time".to_string()).into());
        }

        let ids: HashSet<PacketId> = packets.iter().map(|p| p.id).collect();
        if ids.len() != packets.len() {
            return Err(StorageError::DuplicatePacketId.into());
        }

        let end_ts = start_ts + config.epoch_duration;

        let epoch = Epoch {
            id: self.next_id,
            start_ts,
            end_ts,
            packets,
        };

        if self.epochs.iter().any(|e| e.id == epoch.id) {
            return Err(StorageError::EpochAlreadyExists(epoch.id).into());
        }

        self.epochs.push(epoch.clone());
        self.next_id += 1;

        info!(
            epoch_id = epoch.id,
            packets = epoch.packets.len(),
            "Epoch closed"
        );

        Ok(epoch)
    }

    pub fn get_epoch(&self, id: EpochId) -> Option<&Epoch> {
        self.epochs.iter().find(|e| e.id == id)
    }

    pub fn all_epochs(&self) -> &[Epoch] {
        &self.epochs
    }

    pub fn has_epoch(&self, id: EpochId) -> bool {
        self.epochs.iter().any(|e| e.id == id)
    }

    pub fn len(&self) -> usize {
        self.epochs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.epochs.is_empty()
    }

    pub fn next_id(&self) -> EpochId {
        self.next_id
    }
}