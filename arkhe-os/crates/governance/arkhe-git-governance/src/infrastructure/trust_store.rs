use crate::domain::provenance::*;
use crate::domain::types::*;

pub struct GpgTrustStore;

impl TrustStore for GpgTrustStore {
    fn verify_signature(
        &self,
        _commit_id: &CommitId,
        _signature: &[u8],
        _author: &str,
    ) -> Result<TrustDecision, crate::GitGovernanceError> {
        Ok(TrustDecision {
            status: TrustStatus::Verified,
            fingerprint: None,
            signer: None,
            reason: None,
        })
    }
}

pub struct SshTrustStore;

impl TrustStore for SshTrustStore {
    fn verify_signature(
        &self,
        _commit_id: &CommitId,
        _signature: &[u8],
        _author: &str,
    ) -> Result<TrustDecision, crate::GitGovernanceError> {
        Ok(TrustDecision {
            status: TrustStatus::Verified,
            fingerprint: None,
            signer: None,
            reason: None,
        })
    }
}
