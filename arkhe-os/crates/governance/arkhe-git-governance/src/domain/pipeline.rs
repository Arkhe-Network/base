use crate::domain::types::*;

pub trait Stage: Send + Sync {
    type Input;
    type Output;

    fn process(&self, input: Self::Input) -> Result<Self::Output, crate::GitGovernanceError>;
}

pub struct NoOpStage;

impl Stage for NoOpStage {
    type Input = CommitData;
    type Output = CommitData;

    fn process(&self, input: Self::Input) -> Result<Self::Output, crate::GitGovernanceError> {
        Ok(input)
    }
}
