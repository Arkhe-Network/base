// src/acquisition.rs
//! Módulo de aquisição de dados — hardware e REM window

use crate::{
    types::*,
    rem::RemWindow, Result,
};
use std::collections::HashSet;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct Acquisition {
    config: Config,
    packets: Vec<DataPacket>,
    hardware_active: bool,
    experimenters: HashSet<Rater>,
    /// PLACEHOLDER: Em produção, integrar com TPM 2.0 (tss-esapi) ou HSM PKCS#11
    hardware_key: Rater,
    acquired_history: HashSet<PacketId>,
    next_id: PacketId,
    rem_window: RemWindow,
}

#[derive(Debug, thiserror::Error)]
pub enum AcquisitionError {
    #[error("Hardware inactive")]
    HardwareInactive,
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Invalid sensor: {0}")]
    InvalidSensor(String),
    #[error("Invalid sample length: {0}")]
    InvalidSampleLength(usize),
    #[error("Packet ID already used: {0}")]
    DuplicatePacketId(PacketId),
    #[error("Stimulus outside REM window")]
    StimulusOutsideRem,
    #[error("Packet limit reached: {0}")]
    PacketLimitReached(PacketId),
    #[error("REM window error: {0}")]
    RemWindow(#[from] crate::rem::RemWindowError),
}

impl Acquisition {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            packets: Vec::new(),
            hardware_active: true,
            experimenters: config.experimenters.clone(),
            hardware_key: "hardware-key".to_string(),
            acquired_history: HashSet::new(),
            next_id: 0,
            rem_window: RemWindow::new(),
        }
    }

    pub fn acquire(
        &mut self,
        ts: Timestamp,
        sensor: SensorType,
        sample: &[Sample],
        now: Timestamp,
    ) -> Result<DataPacket> {
        if !self.hardware_active {
            return Err(AcquisitionError::HardwareInactive.into());
        }

        if ts < now.saturating_sub(self.config.max_clock_skew)
            || ts > now + self.config.max_clock_skew
        {
            return Err(AcquisitionError::InvalidTimestamp(
                format!("ts={} outside [{}, {}]", ts,
                    now.saturating_sub(self.config.max_clock_skew),
                    now + self.config.max_clock_skew)
            ).into());
        }

        if sample.is_empty() || sample.len() > self.config.max_sample_length {
            return Err(AcquisitionError::InvalidSampleLength(sample.len()).into());
        }

        if self.next_id >= self.config.max_packet_id {
            return Err(AcquisitionError::PacketLimitReached(self.next_id).into());
        }

        if sensor == SensorType::STIM
            && !self.rem_window.is_active(ts) {
                return Err(AcquisitionError::StimulusOutsideRem.into());
            }

        let packet = DataPacket {
            ts,
            sensor,
            sample: sample.to_vec(),
            id: self.next_id,
        };

        self.packets.push(packet.clone());
        self.acquired_history.insert(packet.id);
        self.next_id += 1;

        info!(
            packet_id = packet.id,
            sensor = %sensor,
            ts = ts,
            "Packet acquired"
        );

        Ok(packet)
    }

    pub fn update_rem_window(&mut self, start: Timestamp, end: Timestamp, now: Timestamp) -> Result<()> {
        self.rem_window.update(start, end, now, &self.config)?;
        Ok(())
    }

    pub fn hardware_failure(&mut self) {
        self.hardware_active = false;
        warn!("Hardware failure");
    }

    pub fn hardware_recovery(&mut self) {
        self.hardware_active = true;
        self.packets.clear();
        info!("Hardware recovered");
    }

    pub fn lose_packet(&mut self, index: usize) -> Option<DataPacket> {
        if index < self.packets.len() {
            Some(self.packets.remove(index))
        } else {
            None
        }
    }

    pub fn packets(&self) -> &[DataPacket] {
        &self.packets
    }

    pub fn packets_in_range(&self, start: Timestamp, end: Timestamp) -> Vec<DataPacket> {
        self.packets
            .iter()
            .filter(|p| p.ts >= start && p.ts <= end)
            .cloned()
            .collect()
    }

    pub fn has_packet(&self, id: PacketId) -> bool {
        self.acquired_history.contains(&id)
    }

    pub fn next_id(&self) -> PacketId {
        self.next_id
    }

    pub fn is_hardware_active(&self) -> bool {
        self.hardware_active
    }

    pub fn rem_window(&self) -> &RemWindow {
        &self.rem_window
    }
}