use ndarray::prelude::*;
use serde::{Deserialize, Serialize};

use crate::mhd::EvoField;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shadow {
    pub energy_ratio: f64,
}
impl Shadow {
    pub fn strength(&self) -> f64 {
        0.0
    }
    pub fn from_svd(_: &(Option<Array2<f64>>, Array1<f64>, Option<Array2<f64>>), _: usize) -> Self {
        Self { energy_ratio: 0.0 }
    }
}
pub struct ShadowHealer {}
impl ShadowHealer {
    pub fn new(_: f64) -> Self {
        Self {}
    }
    pub fn heal(&self, _: &mut EvoField, _: &Shadow) {}
}
