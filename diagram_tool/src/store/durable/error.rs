use thiserror::Error;

use crate::store::types::OperationState;
use crate::store_async::AsyncStoreError as StoreError;

#[derive(Debug, Error)]
pub enum DurableError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Operation not found: {0}")]
    OperationNotFound(String),
    #[error("Operation in invalid state: expected {expected:?}, found {found:?}")]
    OperationStateInvalid {
        expected: OperationState,
        found: OperationState,
    },
    #[error("Step not found: operation {operation_id}, step {step_index}")]
    StepNotFound {
        operation_id: String,
        step_index: u32,
    },
    #[error("Step already completed: operation {operation_id}, step {step_index}")]
    StepAlreadyCompleted {
        operation_id: String,
        step_index: u32,
    },
    #[error("Outbox entry not found: {0}")]
    OutboxNotFound(String),
    #[error("Outbox max retries exceeded: {0}")]
    OutboxMaxRetriesExceeded(String),
    #[error("Cursor parse error: {0}")]
    CursorParseError(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<StoreError> for DurableError {
    fn from(err: StoreError) -> Self {
        match err {
            StoreError::Io(e) => Self::Io(e),
            StoreError::Sqlx(e) => Self::Sqlx(e),
            StoreError::ValidationFailed(s) => Self::ValidationFailed(s),
            StoreError::Serialization(s) => Self::Serialization(s),
            other => Self::ValidationFailed(other.to_string()),
        }
    }
}
