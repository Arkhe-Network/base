use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoSignal {
    pub strength: f64,
    pub predicted_helicity: f64,
    pub origin_height: u64,
    pub timestamp: f64,
}
impl EchoSignal {
    pub fn new(_: f64, _: f64, _: u64, _: Vec<f64>) -> Self {
        Self { strength: 0.0, predicted_helicity: 0.0, origin_height: 0, timestamp: 0.0 }
    }
}
pub struct RetroCausalChannel {}
impl RetroCausalChannel {
    pub fn new(_: f64, _: f64) -> Self {
        Self {}
    }
    pub fn emit(&mut self, _: EchoSignal) {}
    pub fn receive(&mut self, _: f64) -> Vec<EchoSignal> {
        vec![]
    }
}
