use crate::domain::types::*;

pub trait PolicyEngine: Send + Sync {
    fn decide(&self, findings: &[Finding], altai_score: u32) -> GovernanceDecision;
}

pub struct ThresholdPolicy {
    threshold: u32,
}

impl ThresholdPolicy {
    pub fn new(threshold: u32) -> Self {
        Self { threshold }
    }
}

impl PolicyEngine for ThresholdPolicy {
    fn decide(&self, findings: &[Finding], altai_score: u32) -> GovernanceDecision {
        let has_critical = findings.iter().any(|f| f.severity == Severity::Critical);
        let has_secret = findings
            .iter()
            .any(|f| f.category == FindingCategory::SecretLeak);
        let error_count = findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();

        if has_secret || has_critical {
            return GovernanceDecision::Deny;
        }

        if altai_score < self.threshold {
            return GovernanceDecision::NeedsReview;
        }

        if error_count > 5 {
            return GovernanceDecision::Quarantine;
        }

        if error_count > 0 || altai_score < self.threshold + 1000 {
            return GovernanceDecision::NeedsReview;
        }

        GovernanceDecision::Allow
    }
}
