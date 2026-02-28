//! Error types for locking operations.

use thiserror::Error;

/// Errors that can occur during locking operations.
#[derive(Debug, Error)]
pub enum LockError {
    #[error("Lock acquisition timeout for diagram: {0}")]
    Timeout(String),

    #[error("Lock release failed: {0}")]
    ReleaseError(String),

    #[error("IO error during lock operation: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Queue error: {0}")]
    QueueError(String),

    #[error("Mutation error: {0}")]
    MutationError(String),
}

impl From<crate::mutation::error::MutationError> for LockError {
    fn from(err: crate::mutation::error::MutationError) -> Self {
        Self::MutationError(err.to_string())
    }
}
