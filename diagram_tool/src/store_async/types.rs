//! Types for the async store.

use serde::Serialize;
use sqlx::SqlitePool;
use std::path::PathBuf;

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

#[must_use]
pub const fn map_error_code(err: &super::error::AsyncStoreError) -> CliErrorCode {
    use super::error::AsyncStoreError;
    match err {
        AsyncStoreError::RevisionMismatch { .. }
        | AsyncStoreError::RevisionGap { .. }
        | AsyncStoreError::DuplicateWithConflict(_) => CliErrorCode::RevisionMismatch,
        AsyncStoreError::ValidationFailed(_)
        | AsyncStoreError::EmptyBatch
        | AsyncStoreError::BatchTooLarge
        | AsyncStoreError::InvalidTimestamp
        | AsyncStoreError::InvalidOperationId
        | AsyncStoreError::OperationIdTooLong
        | AsyncStoreError::PayloadTooLarge => CliErrorCode::ValidationFailed,
        AsyncStoreError::Sqlx(_)
        | AsyncStoreError::Io(_)
        | AsyncStoreError::InvalidPragma(_)
        | AsyncStoreError::SchemaVersionMismatch { .. }
        | AsyncStoreError::Serialization(_)
        | AsyncStoreError::TransactionAborted { .. } => CliErrorCode::Unknown,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncAppendResult {
    pub revision: i64,
    pub op_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncBatchAppendResult {
    pub start_revision: i64,
    pub end_revision: i64,
    pub count: usize,
    pub op_ids: Vec<String>,
    pub last_timestamp: i64,
}

pub struct AsyncStoreBootstrap {
    pub pool: SqlitePool,
    pub db_path: PathBuf,
    pub schema_version: i32,
}

pub struct AsyncStorePragmas {
    pub journal_mode: String,
    pub synchronous: i32,
    pub wal_autocheckpoint: i32,
    pub foreign_keys: bool,
    pub busy_timeout: i32,
}

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub op_id: String,
    pub revision: i64,
    pub timestamp: i64,
    pub payload: String,
}
