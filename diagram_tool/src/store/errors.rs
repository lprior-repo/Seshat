use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Invalid pragma configuration: {0}")]
    InvalidPragma(String),
    #[error("Schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: i32, found: i32 },
    #[error("Migration forbidden: schema version {version} cannot be migrated")]
    MigrationForbidden { version: i32 },
    #[error("Revision mismatch: expected {expected}, found {found}")]
    RevisionMismatch { expected: i64, found: i64 },
    #[error("Human priority block: {0}")]
    HumanPriorityBlock(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Transaction aborted: {0}")]
    TransactionAborted(String),
    #[error(
        "Revision gap detected: expected sequential revision {expected}, but found gap at {found}"
    )]
    RevisionGap { expected: i64, found: i64 },
    #[error("Duplicate op_id with conflict: {0}")]
    DuplicateWithConflict(String),
    #[error("Empty batch: cannot append zero events")]
    EmptyBatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

pub const fn map_error_code(err: &StoreError) -> CliErrorCode {
    match err {
        StoreError::RevisionMismatch { .. } => CliErrorCode::RevisionMismatch,
        StoreError::RevisionGap { .. } => CliErrorCode::RevisionMismatch,
        StoreError::HumanPriorityBlock(_) => CliErrorCode::HumanPriorityBlock,
        StoreError::ValidationFailed(_) => CliErrorCode::ValidationFailed,
        StoreError::Sqlite(_) => CliErrorCode::Unknown,
        StoreError::Io(_) => CliErrorCode::Unknown,
        StoreError::InvalidPragma(_) => CliErrorCode::Unknown,
        StoreError::SchemaVersionMismatch { .. } => CliErrorCode::Unknown,
        StoreError::MigrationForbidden { .. } => CliErrorCode::Unknown,
        StoreError::Serialization(_) => CliErrorCode::Unknown,
        StoreError::TransactionAborted(_) => CliErrorCode::Unknown,
        StoreError::DuplicateWithConflict(_) => CliErrorCode::RevisionMismatch,
        StoreError::EmptyBatch => CliErrorCode::ValidationFailed,
    }
}

pub fn render_error_json(code: CliErrorCode, message: &str) -> String {
    serde_json::json!({
        "code": code.code(),
        "message": message
    })
    .to_string()
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Store failure: {0}")]
    StoreFailure(#[from] StoreError),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl CliError {
    #[must_use]
    pub fn error_code(&self) -> CliErrorCode {
        match self {
            Self::InvalidInput(_) => CliErrorCode::ValidationFailed,
            Self::StoreFailure(err) => map_error_code(err),
            Self::Conflict(_) => CliErrorCode::RevisionMismatch,
            Self::Serialization(_) => CliErrorCode::Unknown,
        }
    }
}

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("Database integrity check failed: {0}")]
    CorruptDatabase(String),
    #[error("SQLite error during recovery: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error during recovery: {0}")]
    Io(#[from] std::io::Error),
    #[error("Backup file unavailable: {0}")]
    BackupUnavailable(String),
}
