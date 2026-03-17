#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use diagram_models::validation::ValidationIssue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MutationError {
    #[error("schema error: {0}")]
    Schema(String),
    #[error("semantic validation error: {0}")]
    Semantic(String),
}

impl MutationError {
    #[must_use]
    pub fn from_issue(issue: &ValidationIssue) -> Self {
        Self::Semantic(format!("{}: {}", issue.code, issue.message))
    }
}

impl From<diagram_models::dag::CycleError> for MutationError {
    fn from(err: diagram_models::dag::CycleError) -> Self {
        Self::Schema(format!("cycle error: {err}"))
    }
}

impl From<diagram_models::conflict::ConflictError> for MutationError {
    fn from(err: diagram_models::conflict::ConflictError) -> Self {
        Self::Semantic(format!("conflict: {err}"))
    }
}

impl From<anyhow::Error> for MutationError {
    fn from(err: anyhow::Error) -> Self {
        Self::Schema(err.to_string())
    }
}
