//! Error types for the async store.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    Exact,
    Conflict,
}

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Error)]
pub enum AsyncStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Invalid pragma configuration: {0}")]
    InvalidPragma(String),
    #[error("Schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: i32, found: i32 },
    #[error("Revision mismatch: expected {expected}, found {found}")]
    RevisionMismatch { expected: i64, found: i64 },
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Transaction aborted: {source}")]
    TransactionAborted {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error(
        "Revision gap detected: expected sequential revision {expected}, but found gap at {found}"
    )]
    RevisionGap { expected: i64, found: i64 },
    #[error("Duplicate op_id with conflict: {0}")]
    DuplicateWithConflict(String),
    #[error("Empty batch: cannot append zero events")]
    EmptyBatch,
    #[error("Batch too large: cannot exceed max batch size")]
    BatchTooLarge,
    #[error("Invalid timestamp: must be non-zero")]
    InvalidTimestamp,
    #[error("Invalid operation ID: cannot be empty or contain null bytes")]
    InvalidOperationId,
    #[error("Operation ID too long: cannot exceed 255 bytes")]
    OperationIdTooLong,
    #[error("Payload too large: cannot exceed 100MB")]
    PayloadTooLarge,
}
