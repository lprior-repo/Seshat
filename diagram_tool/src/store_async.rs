//! Async SQLite storage module
//!
//! Provides async SQLite-based storage with WAL mode and connection pooling.
//! This is the async counterpart to the synchronous `store` module.
//!
//! ## Benefits over synchronous rusqlite
//!
//! - **True concurrency**: Multiple operations can run simultaneously
//! - **Non-blocking**: Async operations don't block the thread
//! - **Connection pooling**: Efficient management of database connections
//! - **Better resource utilization**: Under high concurrent load

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::Serialize;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use crate::models::envelope::{encode_event_envelope, EventEnvelope};

/// Current schema version for the async store
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Duplicate detection kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    /// Exact duplicate (same payload)
    Exact,
    /// Conflicting duplicate (same op_id, different payload)
    Conflict,
}

/// Errors for async store operations
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
    #[error("Revision gap detected: expected sequential revision {expected}, but found gap at {found}")]
    RevisionGap { expected: i64, found: i64 },
    #[error("Duplicate op_id with conflict: {0}")]
    DuplicateWithConflict(String),
    #[error("Empty batch: cannot append zero events")]
    EmptyBatch,
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
pub const fn map_error_code(err: &AsyncStoreError) -> CliErrorCode {
    match err {
        AsyncStoreError::RevisionMismatch { .. } => CliErrorCode::RevisionMismatch,
        AsyncStoreError::RevisionGap { .. } => CliErrorCode::RevisionMismatch,
        AsyncStoreError::ValidationFailed(_) => CliErrorCode::ValidationFailed,
        AsyncStoreError::Sqlx(_) => CliErrorCode::Unknown,
        AsyncStoreError::Io(_) => CliErrorCode::Unknown,
        AsyncStoreError::InvalidPragma(_) => CliErrorCode::Unknown,
        AsyncStoreError::SchemaVersionMismatch { .. } => CliErrorCode::Unknown,
        AsyncStoreError::Serialization(_) => CliErrorCode::Unknown,
        AsyncStoreError::TransactionAborted { .. } => CliErrorCode::Unknown,
        AsyncStoreError::DuplicateWithConflict(_) => CliErrorCode::RevisionMismatch,
        AsyncStoreError::EmptyBatch => CliErrorCode::ValidationFailed,
    }
}

/// Result of a single append operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncAppendResult {
    pub revision: i64,
    pub op_id: String,
    pub timestamp: i64,
}

/// Result of a batch append operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncBatchAppendResult {
    pub start_revision: i64,
    pub end_revision: i64,
    pub count: usize,
    pub op_ids: Vec<String>,
    pub last_timestamp: i64,
}

/// Bootstrap result containing the pool and metadata
pub struct AsyncStoreBootstrap {
    pub pool: SqlitePool,
    pub db_path: PathBuf,
    pub schema_version: i32,
}

/// Pragma configuration for the store
pub struct AsyncStorePragmas {
    pub journal_mode: String,
    pub synchronous: i32,
    pub wal_autocheckpoint: i32,
    pub foreign_keys: bool,
    pub busy_timeout: i32,
}

/// Creates an async SQLite connection pool with the given max_connections
pub async fn create_async_pool(db_path: &Path) -> Result<SqlitePool, AsyncStoreError> {
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

    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(&pool)
        .await?;

    Ok(pool)
}

/// Bootstraps a new async store, creating the database and running migrations
pub async fn bootstrap_async_store(db_path: &Path) -> Result<AsyncStoreBootstrap, AsyncStoreError> {
    let pool = create_async_pool(db_path).await?;

    run_async_schema_migration(&pool).await?;

    let schema_version = sqlx::query_scalar::<_, i32>("SELECT version FROM schema_version")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    Ok(AsyncStoreBootstrap {
        pool,
        db_path: db_path.to_path_buf(),
        schema_version,
    })
}

/// Runs schema migrations for the async store
async fn run_async_schema_migration(pool: &SqlitePool) -> Result<(), AsyncStoreError> {
    let table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'"
    )
    .fetch_one(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    if table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL DEFAULT 1
            )"
        )
        .execute(pool)
        .await?;

        sqlx::query("INSERT OR IGNORE INTO schema_version (version) VALUES (1)")
            .execute(pool)
            .await?;
    }

    let events_table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'"
    )
    .fetch_one(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    if events_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id TEXT NOT NULL UNIQUE,
                revision INTEGER NOT NULL,
                payload TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )"
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
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='snapshots'"
    )
    .fetch_one(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    if snapshot_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER NOT NULL PRIMARY KEY,
                revision INTEGER NOT NULL UNIQUE,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )"
        )
        .execute(pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC)")
            .execute(pool)
            .await?;
    }

    Ok(())
}

/// Fetches the latest revision number from the store
pub async fn fetch_latest_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    let revision: Option<i64> = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_optional(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    Ok(revision.unwrap_or(0))
}

/// Gets the current revision
pub async fn current_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    fetch_latest_revision(pool).await
}

/// Gets the next revision number
pub async fn next_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    let current = current_revision(pool).await?;
    Ok(current + 1)
}

/// Appends a single event to the store
pub async fn append_event_async(
    pool: &SqlitePool,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AsyncAppendResult, AsyncStoreError> {
    let mut tx = pool.begin().await.map_err(AsyncStoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(AsyncStoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    let new_revision = current_revision + 1;

    let payload = encode_event_envelope(&envelope)
        .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;

    sqlx::query(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind(&envelope.op_id)
    .bind(new_revision)
    .bind(&payload)
    .bind(envelope.timestamp.to_string())
    .execute(&mut *tx)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    tx.commit().await.map_err(AsyncStoreError::Sqlx)?;

    Ok(AsyncAppendResult {
        revision: new_revision,
        op_id: envelope.op_id,
        timestamp: envelope.timestamp,
    })
}

/// Appends a batch of events atomically
pub async fn append_batch_async(
    pool: &SqlitePool,
    ops: Vec<EventEnvelope>,
    expected_revision: Option<i64>,
) -> Result<AsyncBatchAppendResult, AsyncStoreError> {
    if ops.is_empty() {
        return Err(AsyncStoreError::EmptyBatch);
    }

    let mut tx = pool.begin().await.map_err(AsyncStoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(AsyncStoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    let batch_size = ops.len();
    let start_revision = current_revision + 1;
    let end_revision = current_revision + batch_size as i64;
    let mut op_ids = Vec::with_capacity(batch_size);
    let mut last_timestamp = 0i64;

    for (idx, envelope) in ops.into_iter().enumerate() {
        let new_revision = current_revision + 1 + idx as i64;

        let payload = encode_event_envelope(&envelope)
            .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;

        sqlx::query(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(&envelope.op_id)
        .bind(new_revision)
        .bind(&payload)
        .bind(envelope.timestamp.to_string())
        .execute(&mut *tx)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

        op_ids.push(envelope.op_id);
        last_timestamp = envelope.timestamp;
    }

    tx.commit().await.map_err(AsyncStoreError::Sqlx)?;

    Ok(AsyncBatchAppendResult {
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
pub async fn lookup_existing_op_async(
    pool: &SqlitePool,
    op_id: &str,
) -> Result<Option<EventRecord>, AsyncStoreError> {
    let result = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events WHERE operation_id = ?1"
    )
    .bind(op_id)
    .fetch_optional(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    match result {
        Some((op_id, revision, timestamp_str, payload)) => {
            let timestamp: i64 = timestamp_str.parse().map_err(|_| {
                AsyncStoreError::Serialization("Invalid timestamp format".to_string())
            })?;
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
pub async fn classify_duplicate_async(
    existing: &EventRecord,
    incoming: &EventEnvelope,
) -> Result<DuplicateKind, AsyncStoreError> {
    let incoming_payload = encode_event_envelope(incoming)
        .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;

    if existing.payload == incoming_payload {
        Ok(DuplicateKind::Exact)
    } else {
        Ok(DuplicateKind::Conflict)
    }
}

/// Appends an event idempotently (handles duplicates gracefully)
pub async fn append_idempotent_async(
    pool: &SqlitePool,
    envelope: EventEnvelope,
) -> Result<AsyncAppendResult, AsyncStoreError> {
    let mut tx = pool.begin().await.map_err(AsyncStoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let payload = encode_event_envelope(&envelope)
        .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;

    let new_revision = current_revision + 1;
    let insert_result = sqlx::query(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind(&envelope.op_id)
    .bind(new_revision)
    .bind(&payload)
    .bind(envelope.timestamp.to_string())
    .execute(&mut *tx)
    .await;

    match insert_result {
        Ok(_) => {
            tx.commit().await.map_err(AsyncStoreError::Sqlx)?;
            Ok(AsyncAppendResult {
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
                let existing = lookup_existing_op_async(pool, &envelope.op_id).await?;

                match existing {
                    Some(record) => {
                        let kind = classify_duplicate_async(&record, &envelope).await?;

                        match kind {
                            DuplicateKind::Exact => {
                                Ok(AsyncAppendResult {
                                    revision: record.revision,
                                    op_id: record.op_id,
                                    timestamp: record.timestamp,
                                })
                            }
                            DuplicateKind::Conflict => {
                                Err(AsyncStoreError::DuplicateWithConflict(envelope.op_id))
                            }
                        }
                    }
                    None => Err(AsyncStoreError::Sqlx(e)),
                }
            } else {
                Err(AsyncStoreError::Sqlx(e))
            }
        }
    }
}

/// Fetches all events since a given revision
pub async fn fetch_events_since(
    pool: &SqlitePool,
    revision: i64,
) -> Result<Vec<EventRecord>, AsyncStoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events WHERE revision > ?1 ORDER BY revision ASC"
    )
    .bind(revision)
    .fetch_all(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    let mut events = Vec::with_capacity(rows.len());
    for (op_id, revision, timestamp_str, payload) in rows {
        let timestamp: i64 = timestamp_str.parse().map_err(|_| {
            AsyncStoreError::Serialization("Invalid timestamp format".to_string())
        })?;
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
pub async fn fetch_all_events(pool: &SqlitePool) -> Result<Vec<EventRecord>, AsyncStoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events ORDER BY revision ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    let mut events = Vec::with_capacity(rows.len());
    for (op_id, revision, timestamp_str, payload) in rows {
        let timestamp: i64 = timestamp_str.parse().map_err(|_| {
            AsyncStoreError::Serialization("Invalid timestamp format".to_string())
        })?;
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
pub async fn read_store_pragmas_async(pool: &SqlitePool) -> Result<AsyncStorePragmas, AsyncStoreError> {
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let synchronous: i32 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let wal_autocheckpoint: i32 = sqlx::query_scalar("PRAGMA wal_autocheckpoint")
        .fetch_one(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let foreign_keys: i32 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let busy_timeout: i32 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    Ok(AsyncStorePragmas {
        journal_mode,
        synchronous,
        wal_autocheckpoint,
        foreign_keys: foreign_keys != 0,
        busy_timeout,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_async_pool() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = create_async_pool(&db_path).await.expect("Failed to create pool");

        // Verify pragmas are set correctly
        let pragmas = read_store_pragmas_async(&pool).await.expect("Failed to read pragmas");

        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 2); // FULL = 2
        assert_eq!(pragmas.wal_autocheckpoint, 1000);
        assert!(pragmas.foreign_keys);
        assert_eq!(pragmas.busy_timeout, 5000);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_bootstrap_async_store() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store");

        assert_eq!(bootstrap.schema_version, CURRENT_SCHEMA_VERSION);

        bootstrap.pool.close().await;
    }

    #[tokio::test]
    async fn test_append_event_async() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
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
            timestamp: 1700000000,
        };

        let result = append_event_async(&pool, envelope, None)
            .await
            .expect("Failed to append event");

        assert_eq!(result.revision, 1);
        assert_eq!(result.op_id, "test-op-1");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_append_idempotent_async() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
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
            timestamp: 1700000000,
        };

        let result1 = append_idempotent_async(&pool, envelope.clone())
            .await
            .expect("Failed to append first");

        let result2 = append_idempotent_async(&pool, envelope)
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

        let pool = bootstrap_async_store(&db_path)
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
                timestamp: 1700000000 + i as i64,
            };
            append_event_async(&pool, envelope, None).await.expect("Failed to append");
        }

        let events = fetch_events_since(&pool, 2).await.expect("Failed to fetch");
        assert_eq!(events.len(), 3); // revisions 3, 4, 5

        pool.close().await;
    }
}
