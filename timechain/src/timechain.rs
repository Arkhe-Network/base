use serde::{Deserialize, Serialize};

use crate::retro::EchoSignal;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShadowHash(pub [u8; 32]);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBlock {
    pub height: u64,
}
pub struct ChernSimonsOracle {}
impl ChernSimonsOracle {
    pub fn new(_: f64) -> Self {
        Self {}
    }
    pub fn verify(&self, _: &TimeBlock, _: &crate::mhd::EvoField) -> bool {
        true
    }
}
pub struct ConsensusEngine {}
impl ConsensusEngine {
    pub fn new(_: f64) -> Self {
        Self {}
    }
    pub fn add_echo(&mut self, _: EchoSignal) {}
    pub fn check_finality(&self, _: u64, _: f64) -> bool {
        true
    }
}
