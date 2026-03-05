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

impl From<crate::models::dag::CycleError> for MutationError {
    fn from(err: crate::models::dag::CycleError) -> Self {
        Self::Schema(format!("cycle error: {}", err))
    }
}

impl From<crate::models::conflict::ConflictError> for MutationError {
    fn from(err: crate::models::conflict::ConflictError) -> Self {
        Self::Semantic(format!("conflict: {}", err))
    }
}

impl From<crate::models::sync::SyncError> for MutationError {
    fn from(err: crate::models::sync::SyncError) -> Self {
        Self::Schema(format!("sync error: {}", err))
    }
}
