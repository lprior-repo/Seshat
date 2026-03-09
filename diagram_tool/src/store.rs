//! Async `SQLite` storage module
//!
//! Provides async SQLite-based storage with WAL mode and connection pooling.
//! This is the async counterpart to the synchronous `store` module.
//!
//! ## Benefits over synchronous `SQLite`
//!
//! - **True concurrency**: Multiple operations can run simultaneously
//! - **Non-blocking**: Async operations don't block the thread
//! - **Connection pooling**: Efficient management of database connections
//! - **Better resource utilization**: Under high concurrent load

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

use serde::Serialize;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::models::envelope::{encode_event_envelope, parse_event_envelope, EventEnvelope};

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

/// Creates an async `SQLite` connection pool.
///
/// # Errors
///
/// Returns a `StoreError` if the connection fails.
pub async fn create_pool(db_path: &Path) -> Result<SqlitePool, StoreError> {
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    // Configure pragmas for optimal concurrent performance
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA synchronous=FULL")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA wal_autocheckpoint=1000")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;

    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(&pool)
        .await?;

    Ok(pool)
}

/// Bootstraps a new async store, creating the database and running migrations
///
/// # Errors
///
/// Returns a `StoreError` if the store cannot be created or migrated.
pub async fn bootstrap_store(db_path: &Path) -> Result<StoreBootstrap, StoreError> {
    let pool = create_pool(db_path).await?;

    run_schema_migration(&pool).await?;

    let schema_version = sqlx::query_scalar::<_, i32>("SELECT version FROM schema_version")
        .fetch_one(&pool)
        .await
        .map_err(|_| StoreError::SchemaVersionMissing)?;

    Ok(StoreBootstrap {
        pool,
        db_path: db_path.to_path_buf(),
        schema_version,
    })
}

/// Runs schema migrations for the async store
async fn run_schema_migration(pool: &SqlitePool) -> Result<(), StoreError> {
    let table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
    )
    .fetch_one(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    if table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL DEFAULT 1
            )",
        )
        .execute(pool)
        .await?;

        sqlx::query("INSERT OR IGNORE INTO schema_version (version) VALUES (1)")
            .execute(pool)
            .await?;
    }

    let events_table_exists: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'")
            .fetch_one(pool)
            .await
            .map_err(StoreError::Sqlx)?;

    if events_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id TEXT NOT NULL UNIQUE,
                revision INTEGER NOT NULL,
                payload TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_operation_id ON events(operation_id)")
            .execute(pool)
            .await?;
    }

    let snapshot_table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='snapshots'",
    )
    .fetch_one(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    if snapshot_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER NOT NULL PRIMARY KEY,
                revision INTEGER NOT NULL UNIQUE,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC)",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Fetches the latest revision number from the store
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn fetch_latest_revision(pool: &SqlitePool) -> Result<i64, StoreError> {
    sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)
}

/// Gets the current revision
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn current_revision(pool: &SqlitePool) -> Result<i64, StoreError> {
    fetch_latest_revision(pool).await
}

/// Gets the next revision number
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn next_revision(pool: &SqlitePool) -> Result<i64, StoreError> {
    let current = current_revision(pool).await?;
    Ok(current + 1)
}

/// Appends a single event to the store
///
/// # Errors
///
/// Returns a `StoreError` if the append fails.
pub async fn append_event(
    pool: &SqlitePool,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendResult, StoreError> {
    let mut tx = pool.begin().await.map_err(StoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(StoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    let payload =
        encode_event_envelope(&envelope).map_err(|e| StoreError::Serialization(e.to_string()))?;

    let new_revision: i64 = sqlx::query_scalar(
        "INSERT INTO events (operation_id, revision, payload, timestamp) 
         VALUES (?1, (SELECT COALESCE(MAX(revision), 0) + 1 FROM events), ?2, ?3)
         RETURNING revision",
    )
    .bind(&envelope.op_id)
    .bind(&payload)
    .bind(envelope.timestamp.to_string())
    .fetch_one(&mut *tx)
    .await
    .map_err(StoreError::Sqlx)?;

    tx.commit().await.map_err(StoreError::Sqlx)?;

    Ok(AppendResult {
        revision: new_revision,
        op_id: envelope.op_id,
        timestamp: envelope.timestamp,
    })
}

/// Appends a batch of events atomically
///
/// # Errors
///
/// Returns a `StoreError` if the batch is empty or the append fails.
#[allow(clippy::cast_possible_wrap)]
pub async fn append_batch(
    pool: &SqlitePool,
    ops: Vec<EventEnvelope>,
    expected_revision: Option<i64>,
) -> Result<BatchAppendResult, StoreError> {
    if ops.is_empty() {
        return Err(StoreError::EmptyBatch);
    }

    let mut tx = pool.begin().await.map_err(StoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(StoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    let batch_size = ops.len();
    // Cast usize to i64 - batch_size is small (bounded by MAX_OPS), safe for revision numbers
    let start_revision = current_revision + 1;
    let end_revision = current_revision + batch_size as i64;
    let mut op_ids = Vec::with_capacity(batch_size);
    let mut last_timestamp = 0i64;

    for (idx, envelope) in ops.into_iter().enumerate() {
        // Cast usize to i64 - idx is bounded by batch_size, safe for revision numbers
        let new_revision = current_revision + 1 + idx as i64;

        let payload = encode_event_envelope(&envelope)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        sqlx::query(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(&envelope.op_id)
        .bind(new_revision)
        .bind(&payload)
        .bind(envelope.timestamp.to_string())
        .execute(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

        op_ids.push(envelope.op_id);
        last_timestamp = envelope.timestamp;
    }

    tx.commit().await.map_err(StoreError::Sqlx)?;

    Ok(BatchAppendResult {
        start_revision,
        end_revision,
        count: batch_size,
        op_ids,
        last_timestamp,
    })
}

/// Event record as stored in the database
#[derive(Debug, Clone)]
pub struct EventRecord {
    pub op_id: String,
    pub revision: i64,
    pub timestamp: i64,
    pub payload: String,
}

/// Looks up an existing operation by ID
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn lookup_existing_op(
    pool: &SqlitePool,
    op_id: &str,
) -> Result<Option<EventRecord>, StoreError> {
    let result = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events WHERE operation_id = ?1",
    )
    .bind(op_id)
    .fetch_optional(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    match result {
        Some((op_id, revision, timestamp_str, payload)) => {
            let timestamp: i64 = timestamp_str
                .parse()
                .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;
            Ok(Some(EventRecord {
                op_id,
                revision,
                timestamp,
                payload,
            }))
        }
        None => Ok(None),
    }
}

/// Classifies a duplicate as exact or conflicting
///
/// # Errors
///
/// Returns a `StoreError` if serialization fails.
#[allow(clippy::unused_async)]
pub async fn classify_duplicate(
    existing: &EventRecord,
    incoming: &EventEnvelope,
) -> Result<DuplicateKind, StoreError> {
    let incoming_payload =
        encode_event_envelope(incoming).map_err(|e| StoreError::Serialization(e.to_string()))?;

    if existing.payload == incoming_payload {
        Ok(DuplicateKind::Exact)
    } else {
        Ok(DuplicateKind::Conflict)
    }
}

/// Appends an event idempotently (handles duplicates gracefully)
///
/// # Errors
///
/// Returns a `StoreError` if the append fails.
pub async fn append_idempotent(
    pool: &SqlitePool,
    envelope: EventEnvelope,
) -> Result<AppendResult, StoreError> {
    let mut tx = pool.begin().await.map_err(StoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    let payload =
        encode_event_envelope(&envelope).map_err(|e| StoreError::Serialization(e.to_string()))?;

    let new_revision = current_revision + 1;
    let insert_result = sqlx::query(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&envelope.op_id)
    .bind(new_revision)
    .bind(&payload)
    .bind(envelope.timestamp.to_string())
    .execute(&mut *tx)
    .await;

    match insert_result {
        Ok(_) => {
            tx.commit().await.map_err(StoreError::Sqlx)?;
            Ok(AppendResult {
                revision: new_revision,
                op_id: envelope.op_id,
                timestamp: envelope.timestamp,
            })
        }
        Err(e) => {
            let is_unique_constraint = e.to_string().contains("UNIQUE constraint failed")
                || e.to_string().contains("constraint failed")
                || e.to_string().contains("constraint");

            if is_unique_constraint {
                // Rollback this transaction and look up the existing record
                drop(tx);
                let existing = lookup_existing_op(pool, &envelope.op_id).await?;

                match existing {
                    Some(record) => {
                        let kind = classify_duplicate(&record, &envelope).await?;

                        match kind {
                            DuplicateKind::Exact => Ok(AppendResult {
                                revision: record.revision,
                                op_id: record.op_id,
                                timestamp: record.timestamp,
                            }),
                            DuplicateKind::Conflict => {
                                Err(StoreError::DuplicateWithConflict(envelope.op_id))
                            }
                        }
                    }
                    None => Err(StoreError::Sqlx(e)),
                }
            } else {
                Err(StoreError::Sqlx(e))
            }
        }
    }
}

/// Fetches all events since a given revision
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn fetch_events_since(
    pool: &SqlitePool,
    revision: i64,
) -> Result<Vec<EventRecord>, StoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events WHERE revision > ?1 ORDER BY revision ASC"
    )
    .bind(revision)
    .fetch_all(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    let mut events = Vec::with_capacity(rows.len());
    for (op_id, revision, timestamp_str, payload) in rows {
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;
        events.push(EventRecord {
            op_id,
            revision,
            timestamp,
            payload,
        });
    }

    Ok(events)
}

/// Fetches all events from the store
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn fetch_all_events(pool: &SqlitePool) -> Result<Vec<EventRecord>, StoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events ORDER BY revision ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    let mut events = Vec::with_capacity(rows.len());
    for (op_id, revision, timestamp_str, payload) in rows {
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;
        events.push(EventRecord {
            op_id,
            revision,
            timestamp,
            payload,
        });
    }

    Ok(events)
}

/// Reads store configuration pragmas
///
/// # Errors
///
/// Returns a `StoreError` if the pragma query fails.
pub async fn read_store_pragmas(pool: &SqlitePool) -> Result<StorePragmas, StoreError> {
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    let synchronous: i32 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    let wal_autocheckpoint: i32 = sqlx::query_scalar("PRAGMA wal_autocheckpoint")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    let foreign_keys: i32 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    let busy_timeout: i32 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    Ok(StorePragmas {
        journal_mode,
        synchronous,
        wal_autocheckpoint,
        foreign_keys: foreign_keys != 0,
        busy_timeout,
    })
}

/// Current configuration of an existing async store
pub struct StoreConfig {
    pub pragmas: StorePragmas,
    pub schema_version: i32,
}

/// Gets the current store configuration (async version)
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn current_store_config(pool: &SqlitePool) -> Result<StoreConfig, StoreError> {
    let pragmas = read_store_pragmas(pool).await?;

    let schema_version: Option<i32> = sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_optional(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    Ok(StoreConfig {
        pragmas,
        schema_version: schema_version.ok_or(StoreError::SchemaVersionMissing)?,
    })
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

/// Runs integrity check on the database at startup (async version)
///
/// # Errors
///
/// Returns a `RecoveryError` if the integrity check fails.
pub async fn startup_integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError> {
    if !db_path.exists() {
        return Ok(IntegrityStatus {
            is_valid: false,
            page_count: 0,
            free_pages: 0,
            corrupted_pages: 0,
            schema_version: None,
            event_count: 0,
            latest_revision: None,
            error_message: Some("Database file does not exist".to_string()),
        });
    }

    let pool = create_pool(db_path).await?;

    let integrity_result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let is_valid = integrity_result == "ok";

    let page_count: u32 = sqlx::query_scalar("PRAGMA page_count")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let free_pages: u32 = sqlx::query_scalar("PRAGMA freelist_count")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let corrupted_pages: u32 = u32::from(!is_valid && integrity_result.contains("corrupt"));

    let schema_version: Option<i32> = sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_optional(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let event_count: u64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let latest_revision: Option<i64> =
        sqlx::query_scalar::<_, Option<i64>>("SELECT COALESCE(MAX(revision), 0) FROM events")
            .fetch_optional(&pool)
            .await
            .map_err(RecoveryError::Sqlx)?
            .flatten()
            .filter(|&rev| rev > 0);

    pool.close().await;

    let error_message = if !is_valid {
        Some(integrity_result)
    } else if corrupted_pages > 0 {
        Some(format!("{corrupted_pages} corrupted pages found"))
    } else {
        None
    };

    Ok(IntegrityStatus {
        is_valid,
        page_count,
        free_pages,
        corrupted_pages,
        schema_version,
        event_count,
        latest_revision,
        error_message,
    })
}

/// Opens the database in read-only recovery mode (async version)
///
/// # Errors
///
/// Returns a `RecoveryError` if the database is corrupt or cannot be opened.
pub async fn open_recovery_mode(db_path: &Path) -> Result<RecoveryHandle, RecoveryError> {
    let connection_string = format!("sqlite:{}?mode=ro", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .map_err(RecoveryError::Sqlx)?;

    let integrity_result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .map_err(RecoveryError::Sqlx)?;

    if integrity_result != "ok" {
        pool.close().await;
        return Err(RecoveryError::CorruptDatabase(integrity_result));
    }

    Ok(RecoveryHandle {
        pool,
        db_path: db_path.to_path_buf(),
        export_path: None,
    })
}

/// Opens the database in recovery-only mode (async version - alias)
///
/// # Errors
///
/// Returns a `RecoveryError` if the database is corrupt or cannot be opened.
pub async fn open_recovery_only(db_path: &Path) -> Result<RecoverySession, RecoveryError> {
    open_recovery_mode(db_path).await
}

/// Runs integrity check on the database (async version - alias)
///
/// # Errors
///
/// Returns a `RecoveryError` if the integrity check fails.
pub async fn integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError> {
    startup_integrity_check(db_path).await
}

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

/// Save a snapshot of the current projection state
///
/// This function:
/// 1. Validates the projection revision matches current latest revision
/// 2. Serializes the projection to JSON
/// 3. Stores in the snapshots table
///
/// # Errors
/// Returns `StoreError::SnapshotStale` if projection revision doesn't match
/// Returns `StoreError::Serialization` if encoding fails
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn save_snapshot(
    pool: &SqlitePool,
    projection: &crate::models::projection::DiagramProjection,
) -> Result<SnapshotMeta, StoreError> {
    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(pool)
        .await?;

    let projection_revision = i64::try_from(projection.revision)
        .map_err(|_| StoreError::Serialization("Revision too large for i64".to_string()))?;

    if projection_revision != current_revision {
        return Err(StoreError::SnapshotStale {
            expected: current_revision,
            found: projection_revision,
        });
    }

    let payload =
        serde_json::to_string(projection).map_err(|e| StoreError::Serialization(e.to_string()))?;

    let mut tx = pool.begin().await?;

    let now_ts: i64 = sqlx::query_scalar("SELECT CAST(strftime('%s', 'now') AS INTEGER)")
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT OR REPLACE INTO snapshots (revision, payload, created_at) VALUES (?1, ?2, ?3)",
    )
    .bind(projection_revision)
    .bind(&payload)
    .bind(now_ts)
    .execute(&mut *tx)
    .await?;

    let id: i64 = sqlx::query_scalar("SELECT id FROM snapshots WHERE revision = ?1")
        .bind(projection_revision)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(SnapshotMeta {
        id,
        revision: projection_revision,
        created_at: now_ts,
    })
}

/// Load projection from latest snapshot with tail replay
///
/// This function:
/// 1. Loads the latest snapshot from the database
/// 2. Fetches all events with revision greater than snapshot revision
/// 3. Replays events on top of the snapshot to produce the final projection
///
/// If no snapshot exists, falls back to full replay from revision 0.
///
/// # Errors
/// Returns `StoreError::NotFound` if no snapshot exists
/// Returns `StoreError::Serialization` if deserialization fails
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn load_projection_from_snapshot(
    pool: &SqlitePool,
) -> Result<crate::models::projection::DiagramProjection, StoreError> {
    let snapshot_result = sqlx::query_as::<_, (i64, i64, String, i64)>(
        "SELECT id, revision, payload, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    // If no snapshot exists, fall back to full replay from events
    let Some((_snapshot_id, snapshot_revision, payload, _created_at)) = snapshot_result else {
        return load_projection_from_events(pool).await;
    };

    let mut base_projection: crate::models::projection::DiagramProjection =
        serde_json::from_str(&payload).map_err(|e| StoreError::Serialization(e.to_string()))?;

    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, payload, timestamp FROM events WHERE revision > ?1 ORDER BY revision ASC",
    )
    .bind(snapshot_revision)
    .fetch_all(pool)
    .await?;

    for (op_id, _revision, event_payload, timestamp_str) in rows {
        let envelope = parse_event_envelope(&event_payload)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;

        let event = crate::models::projection::EventRecord {
            op_id,
            revision: base_projection.revision,
            operation: envelope.operation,
            author: envelope.author,
            timestamp,
        };

        base_projection = crate::models::projection::apply_event(base_projection, &event)
            .map_err(|e| StoreError::Serialization(format!("Replay error: {e}")))?;
    }

    Ok(base_projection)
}

/// Load projection by replaying all events from scratch (fallback when no snapshot)
async fn load_projection_from_events(
    pool: &SqlitePool,
) -> Result<crate::models::projection::DiagramProjection, StoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, payload, timestamp FROM events ORDER BY revision ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut projection = crate::models::projection::DiagramProjection::empty();

    for (op_id, _revision, event_payload, timestamp_str) in rows {
        let envelope = parse_event_envelope(&event_payload)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;

        let event = crate::models::projection::EventRecord {
            op_id,
            revision: projection.revision(),
            operation: envelope.operation,
            author: envelope.author,
            timestamp,
        };

        projection = crate::models::projection::apply_event(projection, &event)
            .map_err(|e| StoreError::Serialization(format!("Replay error: {e}")))?;
    }

    Ok(projection)
}

/// Get metadata for the latest snapshot
///
/// Returns `Ok(Some(meta))` if a snapshot exists, `Ok(None)` if no snapshots exist.
///
/// # Errors
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn get_latest_snapshot_meta(
    pool: &SqlitePool,
) -> Result<Option<SnapshotMeta>, StoreError> {
    let result = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, revision, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    match result {
        Some((id, revision, created_at)) => Ok(Some(SnapshotMeta {
            id,
            revision,
            created_at,
        })),
        None => Ok(None),
    }
}

/// Delete a snapshot by revision
///
/// # Errors
/// Returns `StoreError::InvalidInput` if revision is negative
/// Returns `StoreError::NotFound` if no snapshot exists at the given revision
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn delete_snapshot(pool: &SqlitePool, revision: i64) -> Result<(), StoreError> {
    if revision < 0 {
        return Err(StoreError::InvalidInput(
            "revision must be non-negative".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM snapshots WHERE revision = ?1")
        .bind(revision)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(StoreError::NotFound(format!(
            "no snapshot at revision {revision}"
        )));
    }

    Ok(())
}

/// List all snapshot metadata, ordered by revision descending
///
/// # Errors
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn list_snapshots(pool: &SqlitePool) -> Result<Vec<SnapshotMeta>, StoreError> {
    let rows = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, revision, created_at FROM snapshots ORDER BY revision DESC",
    )
    .fetch_all(pool)
    .await?;

    let snapshots = rows
        .into_iter()
        .map(|(id, revision, created_at)| SnapshotMeta {
            id,
            revision,
            created_at,
        })
        .collect();

    Ok(snapshots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_pool() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.expect("Failed to create pool");

        // Verify pragmas are set correctly
        let pragmas = read_store_pragmas(&pool)
            .await
            .expect("Failed to read pragmas");

        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 2); // FULL = 2
        assert_eq!(pragmas.wal_autocheckpoint, 1000);
        assert!(pragmas.foreign_keys);
        assert_eq!(pragmas.busy_timeout, 5000);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_bootstrap_store() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store");

        assert_eq!(bootstrap.schema_version, CURRENT_SCHEMA_VERSION);

        bootstrap.pool.close().await;
    }

    #[tokio::test]
    async fn test_append_event() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result = append_event(&pool, envelope, None)
            .await
            .expect("Failed to append event");

        assert_eq!(result.revision, 1);
        assert_eq!(result.op_id, "test-op-1");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_append_idempotent() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result1 = append_idempotent(&pool, envelope.clone())
            .await
            .expect("Failed to append first");

        let result2 = append_idempotent(&pool, envelope)
            .await
            .expect("Failed to append second (should be exact duplicate)");

        assert_eq!(result1.revision, result2.revision);
        assert_eq!(result1.op_id, result2.op_id);

        let events = fetch_all_events(&pool).await.expect("Failed to fetch all");
        assert_eq!(events.len(), 1);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_fetch_events_since() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        // Add some events
        for i in 0..5 {
            let envelope = EventEnvelope {
                op_id: format!("test-op-{}", i),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{}", i),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Test Node {}", i),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("Failed to append");
        }

        let events = fetch_events_since(&pool, 2).await.expect("Failed to fetch");
        assert_eq!(events.len(), 3); // revisions 3, 4, 5

        pool.close().await;
    }

    #[tokio::test]
    async fn test_phase2_store_exports_bootstrap_store() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        assert_eq!(bootstrap.schema_version, 1);
        assert_eq!(bootstrap.db_path, db_path);

        bootstrap.pool.close().await;
    }

    #[tokio::test]
    async fn test_phase2_store_exports_append_event() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result = append_event(&bootstrap.pool, envelope, None)
            .await
            .expect("append_event failed");

        assert_eq!(result.revision, 1);
        assert_eq!(result.op_id, "test-op-1");

        bootstrap.pool.close().await;
    }

    #[tokio::test]
    async fn test_phase2_store_exports_fetch_latest_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let initial = fetch_latest_revision(&bootstrap.pool)
            .await
            .expect("fetch_latest_revision failed");
        assert_eq!(initial, 0);

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };
        append_event(&bootstrap.pool, envelope, None)
            .await
            .expect("append failed");

        let after = fetch_latest_revision(&bootstrap.pool)
            .await
            .expect("fetch_latest_revision failed");
        assert_eq!(after, 1);

        bootstrap.pool.close().await;
    }

    #[tokio::test]
    async fn test_phase2_store_exports_read_store_pragmas() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let pragmas = read_store_pragmas(&bootstrap.pool)
            .await
            .expect("read_store_pragmas failed");

        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 2);
        assert_eq!(pragmas.wal_autocheckpoint, 1000);
        assert!(pragmas.foreign_keys);
        assert_eq!(pragmas.busy_timeout, 5000);

        bootstrap.pool.close().await;
    }

    #[tokio::test]
    async fn test_phase2_store_exports_current_store_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let config = current_store_config(&bootstrap.pool)
            .await
            .expect("current_store_config failed");

        assert_eq!(config.pragmas.journal_mode, "wal");
        assert_eq!(config.schema_version, 1);

        bootstrap.pool.close().await;
    }

    #[tokio::test]
    async fn test_phase2_startup_integrity_check_valid_db() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };
        append_event(&bootstrap.pool, envelope, None)
            .await
            .expect("append failed");

        bootstrap.pool.close().await;

        let status = startup_integrity_check(&db_path)
            .await
            .expect("startup_integrity_check failed");

        assert!(status.is_valid);
        assert!(status.error_message.is_none());
        assert_eq!(status.schema_version, Some(1));
        assert_eq!(status.event_count, 1);
    }

    #[tokio::test]
    async fn test_phase2_startup_integrity_check_nonexistent_db() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let nonexistent_path = temp_dir.path().join("nonexistent.db");

        let status = startup_integrity_check(&nonexistent_path)
            .await
            .expect("startup_integrity_check failed");

        assert!(!status.is_valid);
        assert!(status.error_message.is_some());
        assert_eq!(status.page_count, 0);
    }

    #[tokio::test]
    async fn test_phase2_open_recovery_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };
        append_event(&bootstrap.pool, envelope, None)
            .await
            .expect("append failed");

        bootstrap.pool.close().await;

        let handle = open_recovery_mode(&db_path)
            .await
            .expect("open_recovery_mode failed");

        assert_eq!(handle.db_path, db_path);

        handle.pool.close().await;
    }

    #[tokio::test]
    async fn test_phase2_open_recovery_only() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };
        append_event(&bootstrap.pool, envelope, None)
            .await
            .expect("append failed");

        bootstrap.pool.close().await;

        let session = open_recovery_only(&db_path)
            .await
            .expect("open_recovery_only failed");

        assert_eq!(session.db_path, db_path);

        session.pool.close().await;
    }

    #[tokio::test]
    async fn test_phase2_integrity_check() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };
        append_event(&bootstrap.pool, envelope, None)
            .await
            .expect("append failed");

        bootstrap.pool.close().await;

        let status = integrity_check(&db_path)
            .await
            .expect("integrity_check failed");

        assert!(status.is_valid);
        assert_eq!(status.schema_version, Some(1));
    }

    #[tokio::test]
    async fn test_save_snapshot_returns_meta_with_correct_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let projection = crate::models::projection::DiagramProjection::with_revision(5);
        let meta = save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");
        assert_eq!(meta.revision, 5);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_load_projection_from_snapshot_replays_tail() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=3 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let projection_rev3 = crate::models::projection::DiagramProjection::with_revision(3);
        save_snapshot(&pool, &projection_rev3)
            .await
            .expect("save_snapshot failed");

        for i in 4..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let loaded = load_projection_from_snapshot(&pool)
            .await
            .expect("load_projection_from_snapshot failed");
        assert_eq!(loaded.revision, 5);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_latest_snapshot_meta_returns_correct_data() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=10 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let projection = crate::models::projection::DiagramProjection::with_revision(10);
        save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");

        let meta = get_latest_snapshot_meta(&pool)
            .await
            .expect("get_latest_snapshot_meta failed");
        assert!(meta.is_some());
        let meta = meta.expect("snapshot exists");
        assert_eq!(meta.revision, 10);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_delete_snapshot_removes_record() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let projection = crate::models::projection::DiagramProjection::with_revision(5);
        save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");

        delete_snapshot(&pool, 5)
            .await
            .expect("delete_snapshot failed");

        let meta = get_latest_snapshot_meta(&pool)
            .await
            .expect("get_latest_snapshot_meta failed");
        assert!(meta.is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_list_snapshots_returns_all_snapshots() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=8 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");

            if i == 2 || i == 5 || i == 8 {
                let projection =
                    crate::models::projection::DiagramProjection::with_revision(i as u64);
                save_snapshot(&pool, &projection)
                    .await
                    .expect("save_snapshot failed");
            }
        }

        let snapshots = list_snapshots(&pool).await.expect("list_snapshots failed");
        assert_eq!(snapshots.len(), 3);

        let revisions: Vec<i64> = snapshots.iter().map(|s| s.revision).collect();
        assert_eq!(revisions, vec![8, 5, 2]);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_save_snapshot_fails_with_stale_projection() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=10 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let stale_projection = crate::models::projection::DiagramProjection::with_revision(5);
        let result = save_snapshot(&pool, &stale_projection).await;
        assert!(matches!(
            result,
            Err(StoreError::SnapshotStale {
                expected: 10,
                found: 5
            })
        ));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_load_projection_from_snapshot_falls_back_to_replay_when_no_snapshots() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        // When no snapshot exists, should fall back to replay and return empty projection
        let result = load_projection_from_snapshot(&pool).await;
        assert!(
            result.is_ok(),
            "Expected fallback to replay, got: {:?}",
            result
        );
        let projection = result.expect("checked is_ok");
        assert_eq!(
            projection.revision, 0,
            "Empty projection should have revision 0"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_delete_snapshot_fails_when_revision_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let projection = crate::models::projection::DiagramProjection::with_revision(5);
        save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");

        let result = delete_snapshot(&pool, 99).await;
        assert!(matches!(result, Err(StoreError::NotFound(_))));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_save_snapshot_at_revision_zero_succeeds() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        let projection = crate::models::projection::DiagramProjection::empty();
        let meta = save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");
        assert_eq!(meta.revision, 0);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_load_projection_with_no_tail_events() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let projection = crate::models::projection::DiagramProjection::with_revision(5);
        save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");

        let loaded = load_projection_from_snapshot(&pool)
            .await
            .expect("load_projection_from_snapshot failed");
        assert_eq!(loaded.revision, 5);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_multiple_snapshots_same_revision_replaces() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let projection1 = crate::models::projection::DiagramProjection::with_revision(5);
        let meta1 = save_snapshot(&pool, &projection1)
            .await
            .expect("save_snapshot failed");

        let projection2 = crate::models::projection::DiagramProjection::with_revision(5);
        let meta2 = save_snapshot(&pool, &projection2)
            .await
            .expect("save_snapshot failed");

        let snapshots = list_snapshots(&pool).await.expect("list_snapshots failed");
        assert_eq!(snapshots.len(), 1);

        let latest = get_latest_snapshot_meta(&pool)
            .await
            .expect("get_latest_snapshot_meta failed")
            .expect("snapshot exists");
        assert_eq!(latest.id, meta2.id);
        assert_ne!(latest.id, meta1.id);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_delete_snapshot_fails_with_negative_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        let result = delete_snapshot(&pool, -1).await;
        assert!(matches!(result, Err(StoreError::InvalidInput(_))));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_returns_error_when_invalid_db_path_provided() {
        let invalid_path = Path::new("/nonexistent directory that does not exist/test.db");

        let result = bootstrap_store(invalid_path).await;
        assert!(matches!(result, Err(StoreError::Sqlx(_))));
    }

    #[tokio::test]
    async fn test_returns_error_when_appending_with_revision_gap() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result = append_event(&pool, envelope, Some(5)).await;
        assert!(matches!(
            result,
            Err(StoreError::RevisionMismatch {
                expected: 5,
                found: 0
            })
        ));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_handles_concurrent_async_appends_gracefully() {
        use tokio::task::JoinSet;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let mut join_set = JoinSet::new();

        for i in 0..10 {
            let pool = pool.clone();
            join_set.spawn(async move {
                let envelope = EventEnvelope {
                    op_id: format!("concurrent-op-{}", i),
                    operation: crate::models::envelope::DomainOp::NodeAdd {
                        id: format!("node-{}", i),
                        x: 10.0 * i as f64,
                        y: 20.0,
                        width: 100.0,
                        height: 50.0,
                        label: format!("Node {}", i),
                    },
                    author: crate::models::envelope::Author {
                        id: "user-1".to_string(),
                        name: "Test User".to_string(),
                        email: None,
                    },
                    timestamp: 1_700_000_000 + i as i64,
                };
                append_event(&pool, envelope, None).await
            });
        }

        let mut success_count = 0;
        while let Some(result) = join_set.join_next().await {
            if matches!(result, Ok(Ok(_))) {
                success_count += 1;
            }
        }

        assert!(
            success_count > 0,
            "At least some concurrent appends should succeed, got {}",
            success_count
        );

        let final_revision = current_revision(&pool)
            .await
            .expect("Failed to get revision");

        assert_eq!(
            final_revision, success_count as i64,
            "Revision should match successful appends"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_handles_zero_byte_database_initialization() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("zero_byte.db");

        std::fs::write(&db_path, b"").expect("Failed to create zero-byte file");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("Should handle zero-byte file gracefully");

        let test_envelope = EventEnvelope {
            op_id: "init-test-op".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result = append_event(&bootstrap.pool, test_envelope, None).await;
        assert!(
            result.is_ok(),
            "Should be able to append after bootstrap from zero-byte"
        );

        let revision = current_revision(&bootstrap.pool)
            .await
            .expect("Should get revision");
        assert_eq!(revision, 1, "Should have one event after append");

        bootstrap.pool.close().await;
    }

    #[tokio::test]
    async fn test_invariant_unique_op_id_enforced_by_schema() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope = EventEnvelope {
            op_id: "duplicate-op-id".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        append_event(&pool, envelope.clone(), None)
            .await
            .expect("First append should succeed");

        let result = append_event(&pool, envelope, None).await;
        assert!(matches!(result, Err(StoreError::Sqlx(_))));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_precondition_sequential_revision_enforced() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        for i in 1..=3 {
            let envelope = EventEnvelope {
                op_id: format!("seq-op-{}", i),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{}", i),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {}", i),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, Some(i - 1))
                .await
                .expect("Sequential append should succeed");
        }

        let revision = current_revision(&pool)
            .await
            .expect("Failed to get revision");
        assert_eq!(
            revision, 3,
            "Revision should be 3 after 3 sequential appends"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_concurrent_appends_with_expected_revision_serialized() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        for i in 0..5 {
            let envelope = EventEnvelope {
                op_id: format!("serialized-op-{}", i),
                operation: crate::models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{}", i),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {}", i),
                },
                author: crate::models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            let expected_rev = i as i64;
            let result = append_event(&pool, envelope, Some(expected_rev)).await;
            assert!(result.is_ok(), "Append {} should succeed", i);
        }

        let final_revision = current_revision(&pool)
            .await
            .expect("Failed to get revision");
        assert_eq!(final_revision, 5, "Final revision should be 5");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_postcondition_wal_mode_concurrent_access_works() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        use tokio::task::JoinSet;
        let mut join_set = JoinSet::new();
        for i in 0..5 {
            let pool = pool.clone();
            join_set.spawn(async move {
                let envelope = EventEnvelope {
                    op_id: format!("wal-test-{}", i),
                    operation: crate::models::envelope::DomainOp::NodeAdd {
                        id: format!("node-{}", i),
                        x: 10.0,
                        y: 20.0,
                        width: 100.0,
                        height: 50.0,
                        label: "Test".to_string(),
                    },
                    author: crate::models::envelope::Author {
                        id: "user".to_string(),
                        name: "User".to_string(),
                        email: None,
                    },
                    timestamp: i as i64,
                };
                append_event(&pool, envelope, None).await
            });
        }

        let mut successes = 0;
        while let Some(result) = join_set.join_next().await {
            if matches!(result, Ok(Ok(_))) {
                successes += 1;
            }
        }

        assert!(successes > 0, "WAL mode should allow concurrent writes");
        assert_eq!(current_revision(&pool).await.unwrap(), successes as i64);

        pool.close().await;
    }
}
