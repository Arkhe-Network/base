use crate::domain::types::*;
use crate::GitGovernanceError;

pub trait EvidenceStore: Send + Sync {
    fn last_hash(&self) -> Result<Option<Hash>, GitGovernanceError>;
    fn append(&mut self, entry: EvidenceEntry) -> Result<(), GitGovernanceError>;
    fn get(&self, commit_id: &CommitId) -> Result<Option<EvidenceEntry>, GitGovernanceError>;
    fn list(
        &self,
        cursor: Option<String>,
        limit: usize,
    ) -> Result<EvidencePage, GitGovernanceError>;
}

pub struct CanonicalHasher {
    version: CanonicalFormatVersion,
    hasher: blake3::Hasher,
}

impl CanonicalHasher {
    pub fn new(version: CanonicalFormatVersion) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"ARKHE-EVIDENCE");
        hasher.update(&[version as u8]);
        Self { version, hasher }
    }

    pub fn write_field(&mut self, tag: u8, data: &[u8]) {
        self.hasher.update(&[tag]);
        let len = data.len() as u64;
        self.hasher.update(&len.to_le_bytes());
        self.hasher.update(data);
    }

    pub fn write_u8(&mut self, tag: u8, value: u8) {
        self.write_field(tag, &[value]);
    }

    pub fn write_u32(&mut self, tag: u8, value: u32) {
        self.write_field(tag, &value.to_le_bytes());
    }

    pub fn write_str(&mut self, tag: u8, value: &str) {
        self.write_field(tag, value.as_bytes());
    }

    pub fn write_bool(&mut self, tag: u8, value: bool) {
        self.write_field(tag, &[value as u8]);
    }

    pub fn finalize(self) -> Hash {
        let mut out = [0u8; 32];
        out.copy_from_slice(self.hasher.finalize().as_bytes());
        Hash(out)
    }
}

pub struct EvidenceBuilder;

impl EvidenceBuilder {
    pub fn build(
        commit_id: CommitId,
        decision: GovernanceDecision,
        altai_score: u32,
        mut findings: Vec<Finding>,
        policy_version: PolicyVersion,
        config_hash: Hash,
        previous_hash: Option<Hash>,
    ) -> EvidenceEntry {
        findings.sort_by_key(|f| {
            let key = f.canonical_sort_key();
            (key.0, key.1.to_string(), key.2, key.3)
        });

        let entry = EvidenceEntry {
            commit_id,
            decision,
            altai_score,
            findings,
            policy_version,
            config_hash,
            previous_hash,
            current_hash: Hash::zero(),
        };

        let current_hash = Self::compute_hash(&entry);
        EvidenceEntry {
            current_hash,
            ..entry
        }
    }

    fn compute_hash(entry: &EvidenceEntry) -> Hash {
        let mut hasher = CanonicalHasher::new(CanonicalFormatVersion::V1);

        hasher.write_str(1, &entry.commit_id.0);
        hasher.write_u8(2, entry.decision as u8);
        hasher.write_u32(3, entry.altai_score);

        for f in &entry.findings {
            hasher.write_u8(4, f.severity as u8);
            hasher.write_u8(5, f.category as u8);
            let code_val = match &f.code {
                FindingCode::LargeBinary => 1,
                FindingCode::UnsignedCommit => 2,
                FindingCode::UntrustedKey => 3,
                FindingCode::InvalidSignature => 4,
                FindingCode::Other(_) => 255,
            };
            hasher.write_u8(6, code_val);
            if let FindingCode::Other(msg) = &f.code {
                hasher.write_str(7, msg);
            }
            hasher.write_str(8, f.file_path.as_deref().unwrap_or(""));
            hasher.write_u32(9, f.line_number.unwrap_or(0) as u32);
        }

        hasher.write_str(10, &entry.policy_version.0);
        hasher.write_field(11, entry.config_hash.as_bytes());

        if let Some(prev) = &entry.previous_hash {
            hasher.write_field(12, prev.as_bytes());
        }

        hasher.finalize()
    }
}
