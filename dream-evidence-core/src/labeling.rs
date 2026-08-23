// src/labeling.rs
//! Módulo de rotulagem — raters atribuem labels a epochs

use crate::{
    types::*, Result,
};
use std::collections::HashSet;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum LabelingError {
    #[error("Rater already labeled epoch {0}")]
    AlreadyLabeled(EpochId),
    #[error("Experimenter cannot label: {0}")]
    ExperimenterCannotLabel(String),
    #[error("Epoch not found: {0}")]
    EpochNotFound(EpochId),
    #[error("Invalid signature")]
    InvalidSignature,
}

#[derive(Debug, Clone, Default)]
pub struct Labeling {
    pub labels: HashSet<RaterLabel>,
}

impl Labeling {
    pub fn new() -> Self {
        Self {
            labels: HashSet::new(),
        }
    }

    pub fn assign_label(
        &mut self,
        rater: Rater,
        epoch_id: EpochId,
        verdict: Verdict,
        timestamp: Timestamp,
        config: &Config,
    ) -> Result<RaterLabel> {
        if config.is_experimenter(&rater) {
            return Err(LabelingError::ExperimenterCannotLabel(rater).into());
        }

        if self.labels.iter().any(|l| l.rater == rater && l.epoch_id == epoch_id) {
            return Err(LabelingError::AlreadyLabeled(epoch_id).into());
        }

        let signature = sign(&rater, epoch_id, verdict);

        let label = RaterLabel {
            rater: rater.clone(),
            epoch_id,
            verdict,
            timestamp,
            signature,
            processed: false,
        };

        self.labels.insert(label.clone());

        info!(
            rater = %rater,
            epoch_id = epoch_id,
            verdict = %verdict,
            "Label assigned"
        );

        Ok(label)
    }

    pub fn process_label(&mut self, label: &RaterLabel) -> Result<RaterLabel> {
        let mut updated = label.clone();
        updated.processed = true;

        if !verify_signature(&label.rater, label.epoch_id, label.verdict, &label.signature) {
            return Err(LabelingError::InvalidSignature.into());
        }

        self.labels.remove(label);
        self.labels.insert(updated.clone());

        Ok(updated)
    }

    pub fn processed_labels_for_epoch(&self, epoch_id: EpochId) -> Vec<&RaterLabel> {
        self.labels
            .iter()
            .filter(|l| l.epoch_id == epoch_id && l.processed)
            .collect()
    }

    pub fn labels_for_epoch(&self, epoch_id: EpochId) -> Vec<&RaterLabel> {
        self.labels
            .iter()
            .filter(|l| l.epoch_id == epoch_id)
            .collect()
    }

    pub fn labels_by_rater(&self, rater: &Rater) -> Vec<&RaterLabel> {
        self.labels
            .iter()
            .filter(|l| &l.rater == rater)
            .collect()
    }

    pub fn count_verdicts(&self, epoch_id: EpochId) -> std::collections::HashMap<Verdict, usize> {
        let mut counts = std::collections::HashMap::new();
        for label in self.labels.iter().filter(|l| l.epoch_id == epoch_id && l.processed) {
            *counts.entry(label.verdict).or_insert(0) += 1;
        }
        counts
    }

    pub fn has_sufficient_labels(&self, epoch_id: EpochId, threshold: usize) -> bool {
        self.labels
            .iter()
            .filter(|l| l.epoch_id == epoch_id && l.processed)
            .count() >= threshold
    }

    pub fn get_winner(&self, epoch_id: EpochId) -> Option<(Verdict, usize)> {
        let counts = self.count_verdicts(epoch_id);
        counts.into_iter().max_by_key(|(_, count)| *count)
    }

    pub fn all_labels(&self) -> &HashSet<RaterLabel> {
        &self.labels
    }
}