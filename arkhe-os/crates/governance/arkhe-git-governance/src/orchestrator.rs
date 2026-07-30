use crate::domain::*;
use crate::infrastructure::*;
use crate::GitGovernanceError;
use crate::domain::pattern::PatternEngine;
use crate::domain::evidence::{EvidenceBuilder, CanonicalHasher};
use crate::domain::types::{AnalysisResult, CanonicalFormatVersion, PolicyVersion};

pub struct CommitEvaluator {
    git_adapter: GitAdapter,
    pattern_engine: PatternEngine,
    altai: Box<dyn AltaiCriterionEvaluator>,
    policy: Box<dyn PolicyEngine>,
    evidence_store: InMemoryEvidenceStore,
    config: GovernanceConfig,
}

pub struct GovernanceConfig {
    pub policy_version: PolicyVersion,
    pub require_signed_commits: bool,
    pub altai_threshold: u32,
    pub max_diff_size: usize,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            policy_version: PolicyVersion("0.1.0".to_string()),
            require_signed_commits: true,
            altai_threshold: 7000,
            max_diff_size: 10_000_000,
        }
    }
}

impl CommitEvaluator {
    pub fn new(
        git_adapter: GitAdapter,
        pattern_engine: PatternEngine,
        altai: Box<dyn AltaiCriterionEvaluator>,
        policy: Box<dyn PolicyEngine>,
        config: GovernanceConfig,
    ) -> Self {
        Self {
            git_adapter,
            pattern_engine,
            altai,
            policy,
            evidence_store: InMemoryEvidenceStore::new(),
            config,
        }
    }

    pub fn evaluate(
        &mut self,
        repo_path: &str,
        commit_oid: &str,
    ) -> Result<AnalysisResult, GitGovernanceError> {
        let commit = self.git_adapter.load_commit(repo_path, commit_oid)?;

        let pattern_findings = self.pattern_engine.check_commit(&commit);

        let altai_result = self.altai.evaluate(&commit);
        let mut all_findings = pattern_findings;
        all_findings.extend(altai_result.findings);

        let decision = self.policy.decide(&all_findings, altai_result.score);

        let config_hash = self.compute_config_hash();
        let previous_hash = self.evidence_store.last_hash();
        let entry = EvidenceBuilder::build(
            CommitId(commit.metadata.oid.clone()),
            decision,
            altai_result.score,
            all_findings.clone(),
            self.config.policy_version.clone(),
            config_hash,
            previous_hash.clone(),
        );
        self.evidence_store.append(entry)?;

        Ok(AnalysisResult {
            decision,
            findings: all_findings,
            altai_score: altai_result.score,
        })
    }

    fn compute_config_hash(&self) -> Hash {
        let mut hasher = CanonicalHasher::new(CanonicalFormatVersion::V1);
        hasher.write_str(1, &self.config.policy_version.0);
        hasher.write_bool(2, self.config.require_signed_commits);
        hasher.write_u32(3, self.config.altai_threshold);
        hasher.finalize()
    }
}
