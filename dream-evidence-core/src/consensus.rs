// src/consensus.rs
//! Módulo de consenso — alcança consenso com resistência bizantina

use crate::{
    types::*,
    labeling::Labeling, Result,
};
use std::collections::HashSet;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Epoch not found: {0}")]
    EpochNotFound(EpochId),
    #[error("Consensus already reached for epoch {0}")]
    AlreadyReached(EpochId),
    #[error("Insufficient labels: {0} < {1}")]
    InsufficientLabels(usize, usize),
    #[error("No majority: max count {0} < threshold {1}")]
    NoMajority(usize, usize),
    #[error("Tie between verdicts")]
    Tie,
    #[error("Byzantine majority: malicious raters > honest")]
    ByzantineMajority,
}

#[derive(Debug, Clone, Default)]
pub struct Consensus {
    results: Vec<ConsensusResult>,
    config: Config,
}

impl Consensus {
    pub fn new(config: &Config) -> Self {
        Self {
            results: Vec::new(),
            config: config.clone(),
        }
    }

    pub fn reach_threshold(
        &mut self,
        epoch_id: EpochId,
        labeling: &Labeling,
    ) -> Result<ConsensusResult> {
        if self.results.iter().any(|r| r.epoch_id == epoch_id) {
            return Err(ConsensusError::AlreadyReached(epoch_id).into());
        }

        let labels = labeling.processed_labels_for_epoch(epoch_id);
        if labels.len() < self.config.threshold {
            return Err(ConsensusError::InsufficientLabels(
                labels.len(),
                self.config.threshold,
            ).into());
        }

        let mut counts = std::collections::HashMap::new();
        let mut honest_agreed = std::collections::HashMap::new();
        let mut malicious_agreed = std::collections::HashMap::new();

        for label in &labels {
            *counts.entry(label.verdict).or_insert(0) += 1;

            if self.config.is_malicious(&label.rater) {
                *malicious_agreed.entry(label.verdict).or_insert(0) += 1;
            } else {
                *honest_agreed.entry(label.verdict).or_insert(0) += 1;
            }
        }

        // Guarda para conjunto vazio (correção P8)
        if counts.is_empty() {
            return Err(ConsensusError::InsufficientLabels(0, self.config.threshold).into());
        }

        let max_count = counts.values().max().copied().unwrap_or(0);
        let winners: Vec<Verdict> = counts
            .iter()
            .filter(|(_, &count)| count == max_count)
            .map(|(&v, _)| v)
            .collect();

        if winners.len() != 1 {
            return Err(ConsensusError::Tie.into());
        }

        let winner = winners[0];

        if max_count < self.config.threshold {
            return Err(ConsensusError::NoMajority(max_count, self.config.threshold).into());
        }

        let honest_count = honest_agreed.get(&winner).copied().unwrap_or(0);
        let malicious_count = malicious_agreed.get(&winner).copied().unwrap_or(0);

        if honest_count <= malicious_count {
            return Err(ConsensusError::ByzantineMajority.into());
        }

        let raters_agreed: HashSet<Rater> = labels
            .iter()
            .filter(|l| l.verdict == winner)
            .map(|l| l.rater.clone())
            .collect();

        let final_hash = hash_consensus(epoch_id, winner, 0);

        let result = ConsensusResult {
            epoch_id,
            verdict: winner,
            raters_agreed,
            final_hash,
        };

        self.results.push(result.clone());

        info!(
            epoch_id = epoch_id,
            verdict = %winner,
            raters = result.raters_agreed.len(),
            "Consensus reached"
        );

        Ok(result)
    }

    pub fn get_result(&self, epoch_id: EpochId) -> Option<&ConsensusResult> {
        self.results.iter().find(|r| r.epoch_id == epoch_id)
    }

    pub fn has_consensus(&self, epoch_id: EpochId) -> bool {
        self.results.iter().any(|r| r.epoch_id == epoch_id)
    }

    pub fn all_results(&self) -> &[ConsensusResult] {
        &self.results
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }

    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }
}