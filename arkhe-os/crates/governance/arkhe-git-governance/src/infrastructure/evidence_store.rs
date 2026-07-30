use crate::domain::types::*;
use std::collections::HashMap;

pub struct InMemoryEvidenceStore {
    entries: HashMap<CommitId, EvidenceEntry>,
    last_hash: Option<Hash>,
}

impl InMemoryEvidenceStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            last_hash: None,
        }
    }

    pub fn append(&mut self, entry: EvidenceEntry) -> Result<Hash, crate::GitGovernanceError> {
        let hash = entry.current_hash.clone();
        self.entries.insert(entry.commit_id.clone(), entry);
        self.last_hash = Some(hash.clone());
        Ok(hash)
    }

    pub fn last_hash(&self) -> Option<Hash> {
        self.last_hash.clone()
    }

    pub fn verify_chain(&self) -> Result<(), crate::GitGovernanceError> {
        Ok(())
    }

    pub fn get(
        &self,
        commit_id: &CommitId,
    ) -> Result<Option<EvidenceEntry>, crate::GitGovernanceError> {
        Ok(self.entries.get(commit_id).cloned())
    }

    pub fn list(
        &self,
        cursor: Option<String>,
        limit: usize,
    ) -> Result<EvidencePage, crate::GitGovernanceError> {
        let entries: Vec<EvidenceEntry> = self.entries.values().cloned().collect();
        let page = if let Some(cursor) = cursor {
            let start = cursor.parse::<usize>().unwrap_or(0);
            let end = (start + limit).min(entries.len());
            EvidencePage {
                entries: entries[start..end].to_vec(),
                next_cursor: if end < entries.len() {
                    Some(end.to_string())
                } else {
                    None
                },
            }
        } else {
            let end = limit.min(entries.len());
            EvidencePage {
                entries: entries[..end].to_vec(),
                next_cursor: if end < entries.len() {
                    Some(end.to_string())
                } else {
                    None
                },
            }
        };
        Ok(page)
    }
}
