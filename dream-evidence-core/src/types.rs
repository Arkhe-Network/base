// src/types.rs
//! Tipos fundamentais do protocolo DEP v2.5

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

// ─── Constantes de Configuração ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub threshold: usize,
    pub epoch_duration: u64,
    pub max_clock_skew: u64,
    pub max_sample_length: usize,
    pub max_packet_id: u64,
    pub max_epoch_id: u64,
    pub max_packets_per_epoch: usize,
    pub min_rem_duration: u64,
    pub max_rem_duration: u64,
    pub malicious_raters: HashSet<Rater>,
    pub experimenters: HashSet<Rater>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threshold: 3,
            epoch_duration: 2,
            max_clock_skew: 2,
            max_sample_length: 2,
            max_packet_id: 10,
            max_epoch_id: 5,
            max_packets_per_epoch: 2,
            min_rem_duration: 1,
            max_rem_duration: 3,
            malicious_raters: HashSet::new(),
            experimenters: HashSet::new(),
        }
    }
}

impl Config {
    pub fn is_malicious(&self, rater: &Rater) -> bool {
        self.malicious_raters.contains(rater)
    }

    pub fn is_experimenter(&self, rater: &Rater) -> bool {
        self.experimenters.contains(rater)
    }

    pub fn malicious_count(&self) -> usize {
        self.malicious_raters.len()
    }
}

// ─── Tipos Básicos ──────────────────────────────────────────────────────

pub type Rater = String;
pub type PacketId = u64;
pub type EpochId = u64;
pub type Timestamp = u64;
pub type Sample = u64;

/// Hash criptográfico — Blake3 de 32 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub const ZERO: Self = Self([0u8; 32]);
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Hash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(Self(bytes))
    }
}

/// Assinatura — Blake3 de 32 bytes
pub type Signature = Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SensorType {
    EEG,
    EOG,
    EMG,
    STIM,
}

impl fmt::Display for SensorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorType::EEG => write!(f, "EEG"),
            SensorType::EOG => write!(f, "EOG"),
            SensorType::EMG => write!(f, "EMG"),
            SensorType::STIM => write!(f, "STIM"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Verdict {
    Lucid,
    NonLucid,
    Inconclusive,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Lucid => write!(f, "lucid"),
            Verdict::NonLucid => write!(f, "non-lucid"),
            Verdict::Inconclusive => write!(f, "inconclusive"),
        }
    }
}

impl Verdict {
    pub fn all() -> &'static [Verdict] {
        &[Verdict::Lucid, Verdict::NonLucid, Verdict::Inconclusive]
    }
}

// ─── Pacote de Dados ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataPacket {
    pub ts: Timestamp,
    pub sensor: SensorType,
    pub sample: Vec<Sample>,
    pub id: PacketId,
}

impl DataPacket {
    pub fn new(ts: Timestamp, sensor: SensorType, sample: Vec<Sample>, id: PacketId) -> Self {
        Self { ts, sensor, sample, id }
    }

    pub fn is_sorted_by_time(packets: &[DataPacket]) -> bool {
        packets.windows(2).all(|w| w[0].ts <= w[1].ts)
    }
}

// ─── Epoch ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Epoch {
    pub id: EpochId,
    pub start_ts: Timestamp,
    pub end_ts: Timestamp,
    pub packets: Vec<DataPacket>,
}

impl Epoch {
    pub fn new(id: EpochId, start_ts: Timestamp, end_ts: Timestamp, packets: Vec<DataPacket>) -> Self {
        Self { id, start_ts, end_ts, packets }
    }

    pub fn is_sorted(&self) -> bool {
        DataPacket::is_sorted_by_time(&self.packets)
    }

    pub fn duration(&self) -> u64 {
        self.end_ts - self.start_ts
    }
}

// Implementação manual de Hash para Epoch (baseada no id)
impl std::hash::Hash for Epoch {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// ─── Label do Rater ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaterLabel {
    pub rater: Rater,
    pub epoch_id: EpochId,
    pub verdict: Verdict,
    pub timestamp: Timestamp,
    pub signature: Signature,
    pub processed: bool,
}

impl RaterLabel {
    pub fn new(rater: Rater, epoch_id: EpochId, verdict: Verdict, timestamp: Timestamp, signature: Signature) -> Self {
        Self {
            rater,
            epoch_id,
            verdict,
            timestamp,
            signature,
            processed: false,
        }
    }

    pub fn mark_processed(&mut self) {
        self.processed = true;
    }
}

// Implementação manual de Hash para RaterLabel (baseada em rater + epoch_id)
impl std::hash::Hash for RaterLabel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rater.hash(state);
        self.epoch_id.hash(state);
    }
}

// ─── Resultado de Consenso ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub epoch_id: EpochId,
    pub verdict: Verdict,
    pub raters_agreed: HashSet<Rater>,
    pub final_hash: Hash,
}

impl ConsensusResult {
    pub fn new(epoch_id: EpochId, verdict: Verdict, raters_agreed: HashSet<Rater>, final_hash: Hash) -> Self {
        Self {
            epoch_id,
            verdict,
            raters_agreed,
            final_hash,
        }
    }
}

// Implementação manual de Hash para ConsensusResult (baseada em epoch_id)
impl std::hash::Hash for ConsensusResult {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.epoch_id.hash(state);
    }
}

// ─── Entrada de Auditoria ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub hash: Hash,
    pub parent_hash: Hash,
    pub timestamp: Timestamp,
}

impl AuditEntry {
    pub fn new(hash: Hash, parent_hash: Hash, timestamp: Timestamp) -> Self {
        Self { hash, parent_hash, timestamp }
    }

    pub fn root() -> Hash {
        Hash::ZERO
    }
}

// ─── Funções Criptográficas — Serialização Binária Determinística ──────

/// Calcula o hash Blake3 de dados
pub fn hash_data(data: &[u8]) -> Hash {
    let hash = blake3::hash(data);
    Hash(*hash.as_bytes())
}

/// Assina dados com uma chave — serialização binária canônica
/// Formato: [key_len: u8][key bytes][epoch_id: u64 LE][verdict: u8]
pub fn sign(key: &Rater, epoch_id: EpochId, verdict: Verdict) -> Signature {
    let mut buf = Vec::with_capacity(1 + key.len() + 8 + 1);
    buf.push(key.len() as u8);
    buf.extend_from_slice(key.as_bytes());
    buf.extend_from_slice(&epoch_id.to_le_bytes());
    buf.push(verdict as u8);
    hash_data(&buf)
}

/// Verifica uma assinatura
pub fn verify_signature(key: &Rater, epoch_id: EpochId, verdict: Verdict, signature: &Signature) -> bool {
    let expected = sign(key, epoch_id, verdict);
    expected == *signature
}

/// Hash de um epoch para consenso — com nonce monotônico
pub fn hash_consensus(epoch_id: EpochId, verdict: Verdict, nonce: u64) -> Hash {
    let mut buf = Vec::with_capacity(8 + 1 + 8);
    buf.extend_from_slice(&epoch_id.to_le_bytes());
    buf.push(verdict as u8);
    buf.extend_from_slice(&nonce.to_le_bytes());
    hash_data(&buf)
}