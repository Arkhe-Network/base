use crate::domain::types::*;

pub struct GitAdapter;

impl GitAdapter {
    pub fn load_commit(
        &self,
        _repo_path: &str,
        commit_oid: &str,
    ) -> Result<CommitData, crate::GitGovernanceError> {
        Ok(CommitData {
            metadata: CommitMetadata {
                oid: commit_oid.to_string(),
                author: "test".to_string(),
                email: "test@example.com".to_string(),
                message: "stub".to_string(),
                parent_oids: vec![],
            },
            diff: vec![],
            raw_signature: None,
        })
    }
}
