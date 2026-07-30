use crate::domain::types::*;

pub struct TrustDecision {
    pub status: TrustStatus,
    pub fingerprint: Option<String>,
    pub signer: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStatus {
    Verified,
    ValidKey,
    Invalid,
    Missing,
}

pub trait TrustStore: Send + Sync {
    fn verify_signature(
        &self,
        commit_id: &CommitId,
        signature: &[u8],
        author: &str,
    ) -> Result<TrustDecision, crate::GitGovernanceError>;
}

pub struct NoOpTrustStore;

impl TrustStore for NoOpTrustStore {
    fn verify_signature(
        &self,
        _commit_id: &CommitId,
        signature: &[u8],
        _author: &str,
    ) -> Result<TrustDecision, crate::GitGovernanceError> {
        if signature.is_empty() {
            Ok(TrustDecision {
                status: TrustStatus::Missing,
                fingerprint: None,
                signer: None,
                reason: None,
            })
        } else {
            Ok(TrustDecision {
                status: TrustStatus::Verified,
                fingerprint: Some("stub".to_string()),
                signer: Some("stub".to_string()),
                reason: None,
            })
        }
    }
}

pub struct ProvenanceVerifier<T: TrustStore> {
    trust_store: T,
    require_signed: bool,
}

impl<T: TrustStore> ProvenanceVerifier<T> {
    pub fn new(trust_store: T, require_signed: bool) -> Self {
        Self {
            trust_store,
            require_signed,
        }
    }

    pub fn verify(&self, commit: &CommitData) -> Vec<Finding> {
        let signature = commit.raw_signature.as_deref().unwrap_or(&[]);
        let result = self.trust_store.verify_signature(
            &CommitId(commit.metadata.oid.clone()),
            signature,
            &commit.metadata.author,
        );

        match result {
            Ok(decision) => match decision.status {
                TrustStatus::Verified => vec![],
                TrustStatus::ValidKey => {
                    if self.require_signed {
                        vec![Finding {
                            severity: Severity::Warning,
                            category: FindingCategory::ProvenanceFailure,
                            code: FindingCode::UntrustedKey,
                            file_path: None,
                            line_number: None,
                        }]
                    } else {
                        vec![]
                    }
                }
                TrustStatus::Invalid => {
                    vec![Finding {
                        severity: Severity::Error,
                        category: FindingCategory::ProvenanceFailure,
                        code: FindingCode::InvalidSignature,
                        file_path: None,
                        line_number: None,
                    }]
                }
                TrustStatus::Missing => {
                    if self.require_signed {
                        vec![Finding {
                            severity: Severity::Error,
                            category: FindingCategory::ProvenanceFailure,
                            code: FindingCode::UnsignedCommit,
                            file_path: None,
                            line_number: None,
                        }]
                    } else {
                        vec![]
                    }
                }
            },
            Err(e) => {
                vec![Finding {
                    severity: Severity::Error,
                    category: FindingCategory::ProvenanceFailure,
                    code: FindingCode::Other(e.to_string()),
                    file_path: None,
                    line_number: None,
                }]
            }
        }
    }
}
