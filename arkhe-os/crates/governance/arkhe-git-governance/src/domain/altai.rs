use crate::domain::types::*;

pub struct AltaiCriterionResult {
    pub criterion: AltaiCriterion,
    pub score: u32,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AltaiCriterion {
    HumanAgency,
    TechnicalRobustness,
    Privacy,
    Transparency,
    Fairness,
    SocialWellbeing,
    Accountability,
}

pub trait AltaiCriterionEvaluator: Send + Sync {
    fn criterion(&self) -> AltaiCriterion;
    fn evaluate(&self, commit: &CommitData) -> AltaiCriterionResult;
}

pub trait AltaiAggregator: Send + Sync {
    fn aggregate(&self, results: &[AltaiCriterionResult]) -> AltaiResult;
}

pub struct AltaiResult {
    pub score: u32,
    pub findings: Vec<Finding>,
}

pub struct AltaiComposite {
    criteria: Vec<Box<dyn AltaiCriterionEvaluator>>,
    aggregator: Box<dyn AltaiAggregator>,
}

impl AltaiComposite {
    pub fn new(
        criteria: Vec<Box<dyn AltaiCriterionEvaluator>>,
        aggregator: Box<dyn AltaiAggregator>,
    ) -> Self {
        Self {
            criteria,
            aggregator,
        }
    }

    pub fn evaluate(&self, commit: &CommitData) -> AltaiResult {
        let results: Vec<AltaiCriterionResult> = self
            .criteria
            .iter()
            .map(|c| c.evaluate(commit))
            .collect();
        self.aggregator.aggregate(&results)
    }
}
