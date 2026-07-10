use crate::domain::types::*;

pub struct RuleId(pub String);

pub trait PatternRule: Send + Sync {
    fn id(&self) -> RuleId;
    fn check_file(&self, path: &str, content: &[u8]) -> Option<Finding>;
}

pub struct PatternEngine {
    rules: Vec<Box<dyn PatternRule>>,
}

impl PatternEngine {
    pub fn new(rules: Vec<Box<dyn PatternRule>>) -> Self {
        Self { rules }
    }

    pub fn check_commit(&self, commit: &CommitData) -> Vec<Finding> {
        let mut findings = Vec::new();
        for file in &commit.diff {
            if file.is_binary && file.size > 10_000_000 {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: FindingCategory::SecurityVulnerability,
                    code: FindingCode::LargeBinary,
                    file_path: Some(file.path.clone()),
                    line_number: None,
                });
            }
        }
        findings
    }
}
