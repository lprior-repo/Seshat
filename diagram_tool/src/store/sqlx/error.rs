use serde::Serialize;
use thiserror::Error;

/// Errors for async store operations
#[derive(Debug, Error)]
pub enum StoreError {
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
    #[error("Migration forbidden: cannot migrate from version {version}")]
    MigrationForbidden { version: i32 },
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Snapshot stale: expected revision {expected}, found {found}")]
    SnapshotStale { expected: i64, found: i64 },
    #[error("Schema version not found in database")]
    SchemaVersionMissing,
}

/// Structured error codes for CLI output
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliErrorCode {
    RevisionMismatch,
    HumanPriorityBlock,
    PolicyViolation,
    ValidationFailed,
    Unknown,
}

impl CliErrorCode {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::RevisionMismatch => "revision_mismatch",
            Self::HumanPriorityBlock => "human_priority_block",
            Self::PolicyViolation => "policy_violation",
            Self::ValidationFailed => "validation_failed",
            Self::Unknown => "unknown",
        }
    }
}

/// Maps an async store error to a CLI error code
#[must_use]
pub const fn map_error_code(err: &StoreError) -> CliErrorCode {
    match err {
        StoreError::RevisionMismatch { .. }
        | StoreError::RevisionGap { .. }
        | StoreError::DuplicateWithConflict(_)
        | StoreError::SnapshotStale { .. } => CliErrorCode::RevisionMismatch,
        StoreError::ValidationFailed(_) | StoreError::EmptyBatch | StoreError::InvalidInput(_) => {
            CliErrorCode::ValidationFailed
        }
        StoreError::NotFound(_)
        | StoreError::Sqlx(_)
        | StoreError::Io(_)
        | StoreError::InvalidPragma(_)
        | StoreError::SchemaVersionMismatch { .. }
        | StoreError::Serialization(_)
        | StoreError::TransactionAborted { .. }
        | StoreError::MigrationForbidden { .. }
        | StoreError::SchemaVersionMissing => CliErrorCode::Unknown,
    }
}
