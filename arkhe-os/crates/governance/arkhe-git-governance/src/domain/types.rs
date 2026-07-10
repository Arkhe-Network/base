use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CanonicalFormatVersion {
    V1 = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitId(pub String);

impl CommitId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyVersion(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceDecision {
    Allow,
    Deny,
    NeedsReview,
    Quarantine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FindingCategory {
    AltaiHumanAgency,
    AltaiTechnicalRobustness,
    AltaiPrivacy,
    AltaiTransparency,
    AltaiFairness,
    AltaiSocialWellbeing,
    AltaiAccountability,
    SecurityVulnerability,
    SecretLeak,
    AiGeneratedCode,
    ProvenanceFailure,
    PolicyViolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingCode {
    LargeBinary,
    UnsignedCommit,
    UntrustedKey,
    InvalidSignature,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub category: FindingCategory,
    pub code: FindingCode,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
}

impl Finding {
    pub fn canonical_sort_key(&self) -> (u8, &str, usize, u8) {
        (
            self.category as u8,
            self.file_path.as_deref().unwrap_or(""),
            self.line_number.unwrap_or(0),
            match &self.code {
                FindingCode::LargeBinary => 1,
                FindingCode::UnsignedCommit => 2,
                FindingCode::UntrustedKey => 3,
                FindingCode::InvalidSignature => 4,
                FindingCode::Other(_) => 255,
            },
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceEntry {
    pub commit_id: CommitId,
    pub decision: GovernanceDecision,
    pub altai_score: u32,
    pub findings: Vec<Finding>,
    pub policy_version: PolicyVersion,
    pub config_hash: Hash,
    pub previous_hash: Option<Hash>,
    pub current_hash: Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePage {
    pub entries: Vec<EvidenceEntry>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMetadata {
    pub oid: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub parent_oids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: String,
    pub is_binary: bool,
    pub size: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub is_rename: bool,
    pub old_path: Option<String>,
    pub mode_changed: bool,
    pub submodule_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitData {
    pub metadata: CommitMetadata,
    pub diff: Vec<DiffFile>,
    pub raw_signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub decision: GovernanceDecision,
    pub findings: Vec<Finding>,
    pub altai_score: u32,
}
