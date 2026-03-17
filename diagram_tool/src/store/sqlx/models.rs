use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sqlx::SqlitePool;
use crate::store::sqlx::error::*;
use thiserror::Error;

/// Current schema version for the async store
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Duplicate detection kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    /// Exact duplicate (same payload)
    Exact,
    /// Conflicting duplicate (same `op_id`, different payload)
    Conflict,
}
/// Result of a single append operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendResult {
    pub revision: i64,
    pub op_id: String,
    pub timestamp: i64,
}

/// Result of a batch append operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchAppendResult {
    pub start_revision: i64,
    pub end_revision: i64,
    pub count: usize,
    pub op_ids: Vec<String>,
    pub last_timestamp: i64,
}

/// Bootstrap result containing the pool and metadata
pub struct StoreBootstrap {
    pub pool: SqlitePool,
    pub db_path: PathBuf,
    pub schema_version: i32,
}

/// Pragma configuration for the store
pub struct StorePragmas {
    pub journal_mode: String,
    pub synchronous: i32,
    pub wal_autocheckpoint: i32,
    pub foreign_keys: bool,
    pub busy_timeout: i32,
}
/// Event record as stored in the database
#[derive(Debug, Clone)]
pub struct EventRecord {
    pub op_id: String,
    pub revision: i64,
    pub timestamp: i64,
    pub payload: String,
}
/// Current configuration of an existing async store
pub struct StoreConfig {
    pub pragmas: StorePragmas,
    pub schema_version: i32,
}
/// Errors that can occur during async database recovery operations
#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("Database integrity check failed: {0}")]
    CorruptDatabase(String),
    #[error("SQLx error during recovery: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("IO error during recovery: {0}")]
    Io(#[from] std::io::Error),
    #[error("Async store error: {0}")]
    Store(#[from] StoreError),
}

/// Result of an async integrity check
#[derive(Debug, Clone)]
pub struct IntegrityStatus {
    pub is_valid: bool,
    pub page_count: u32,
    pub free_pages: u32,
    pub corrupted_pages: u32,
    pub schema_version: Option<i32>,
    pub event_count: u64,
    pub latest_revision: Option<i64>,
    pub error_message: Option<String>,
}

/// Handle for read-only recovery mode operations (async)
#[derive(Debug)]
pub struct RecoveryHandle {
    pub pool: SqlitePool,
    pub db_path: PathBuf,
    pub export_path: Option<PathBuf>,
}

/// Alias for `RecoveryHandle`
pub type RecoverySession = RecoveryHandle;
use serde::Deserialize;

/// Metadata about a stored snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotMeta {
    /// Unique snapshot identifier (database row id)
    pub id: i64,
    /// Revision number this snapshot represents
    pub revision: i64,
    /// Timestamp when snapshot was created (Unix timestamp)
    pub created_at: i64,
}
