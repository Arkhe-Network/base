// src/byzantine.rs
//! Módulo de detecção bizantina — identificação de comportamento malicioso
//! NOTA: A verificação de assinatura é UNIVERSAL. Todos os labels são verificados,
//! independentemente de estarem na lista de maliciosos.

use crate::{
    types::*,
    labeling::Labeling, Result,
};
use std::collections::HashSet;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum ByzantineError {
    #[error("No forged signatures detected")]
    NoForgeryDetected,
    #[error("No collusion detected")]
    NoCollusionDetected,
    #[error("Forgery detected: {0}")]
    ForgeryDetected(String),
    #[error("Collusion detected: {0}")]
    CollusionDetected(String),
}

#[derive(Debug, Clone, Default)]
pub struct ByzantineDetector {
    config: Config,
    detected_malicious: HashSet<Rater>,
}

impl ByzantineDetector {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            detected_malicious: HashSet::new(),
        }
    }

    /// Detecta assinaturas forjadas em TODOS os labels (universal)
    pub fn detect_forgery(&mut self, labeling: &Labeling) -> Result<Vec<Rater>> {
        let mut forged = Vec::new();

        for label in labeling.all_labels() {
            if !verify_signature(&label.rater, label.epoch_id, label.verdict, &label.signature) {
                forged.push(label.rater.clone());
                self.detected_malicious.insert(label.rater.clone());
            }
        }

        if forged.is_empty() {
            return Err(ByzantineError::NoForgeryDetected.into());
        }

        info!(
            forged_count = forged.len(),
            "Forgery detected"
        );

        Ok(forged)
    }

    /// Detecta colusão entre raters maliciosos
    pub fn detect_collusion(
        &mut self,
        labeling: &Labeling,
        epoch_id: EpochId,
    ) -> Result<Vec<Rater>> {
        let malicious_labels: Vec<&RaterLabel> = labeling
            .labels_for_epoch(epoch_id)
            .into_iter()
            .filter(|l| self.config.is_malicious(&l.rater))
            .collect();

        if malicious_labels.len() < 2 {
            return Err(ByzantineError::NoCollusionDetected.into());
        }

        let verdicts: HashSet<Verdict> = malicious_labels
            .iter()
            .map(|l| l.verdict)
            .collect();

        if verdicts.len() == 1 {
            let colluders: Vec<Rater> = malicious_labels
                .iter()
                .map(|l| l.rater.clone())
                .collect();

            for r in &colluders {
                self.detected_malicious.insert(r.clone());
            }

            info!(
                colluders = colluders.len(),
                verdict = %verdicts.iter().next().unwrap(),
                "Collusion detected"
            );

            Ok(colluders)
        } else {
            Err(ByzantineError::NoCollusionDetected.into())
        }
    }

    pub fn is_trustworthy(&self, rater: &Rater) -> bool {
        !self.detected_malicious.contains(rater) && !self.config.is_malicious(rater)
    }

    pub fn detected_malicious(&self) -> &HashSet<Rater> {
        &self.detected_malicious
    }
}