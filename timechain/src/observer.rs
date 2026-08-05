use serde::{Deserialize, Serialize};

use crate::mhd::EvoField;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverState {
    pub attachment: f64,
}
impl ObserverState {
    pub fn new() -> Self {
        Self { attachment: 0.0 }
    }
    pub fn apply_to_field(&self, _f: &mut EvoField) {}
    pub fn update(&mut self, _dt: f64) {}
}

impl Default for ObserverState {
    fn default() -> Self {
        Self::new()
    }
}
