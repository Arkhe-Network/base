pub mod domain;
pub mod infrastructure;
pub mod orchestrator;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitGovernanceError {
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),
    #[error("Invalid pattern")]
    InvalidPattern,
}
