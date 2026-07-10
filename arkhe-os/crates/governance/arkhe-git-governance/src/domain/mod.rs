pub mod altai;
pub mod evidence;
pub mod policy;
pub mod types;
pub mod provenance;
pub mod pattern;
pub mod pipeline;

pub use altai::{AltaiCriterionEvaluator, AltaiAggregator, AltaiResult};
pub use evidence::{EvidenceStore};
pub use policy::{PolicyEngine};
pub use types::{
    CommitId, CommitMetadata, DiffFile, EvidenceEntry, Finding, FindingCategory,
    GovernanceDecision, Hash, Severity,
};
