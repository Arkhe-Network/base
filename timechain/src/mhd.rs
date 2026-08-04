use ndarray::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlasmaConfig {
    pub nx: usize,
    pub ny: usize,
    pub length: f64,
    pub nu_base: f64,
}
impl PlasmaConfig {
    pub fn new(nx: usize, ny: usize, length: f64, nu_base: f64) -> Self {
        Self { nx, ny, length, nu_base }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvoField {
    pub omega_x: Array2<f64>,
    pub omega_y: Array2<f64>,
    pub omega_z: Array2<f64>,
    pub config: PlasmaConfig,
    pub nu_eff: f64,
}
impl EvoField {
    pub fn harris_sheet(config: PlasmaConfig) -> Self {
        Self {
            omega_x: Array2::zeros((config.nx, config.ny)),
            omega_y: Array2::zeros((config.nx, config.ny)),
            omega_z: Array2::zeros((config.nx, config.ny)),
            config,
            nu_eff: config.nu_base,
        }
    }
    pub fn advance(&mut self, _dt: f64, _ux: &Array2<f64>, _uy: &Array2<f64>) {}
    pub fn helicity(&self) -> f64 {
        0.0
    }
    pub fn energy(&self) -> f64 {
        0.0
    }
}

pub struct ReconnectionDetector {
    pub handover_count: usize,
}
impl ReconnectionDetector {
    pub fn new(_t: f64) -> Self {
        Self { handover_count: 0 }
    }
    pub fn detect(&mut self, _f: &EvoField) -> bool {
        false
    }
}
