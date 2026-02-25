#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::validation::ValidationIssue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MutationError {
    #[error("transform error: {0}")]
    Transform(String),
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
