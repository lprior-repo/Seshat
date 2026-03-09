#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::Serialize;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::models::envelope::{encode_event_envelope, EventEnvelope};

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
    #[error("Revision gap detected: expected sequential revision {expected}, but found gap at {found}")]
    RevisionGap { expected: i64, found: i64 },
    #[error("Duplicate op_id with conflict: {0}")]
    DuplicateWithConflict(String),
    #[error("Empty batch: cannot append zero events")]
    EmptyBatch,
}

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
pub const fn map_error_code(err: &AsyncStoreError) -> CliErrorCode {
    match err {
        AsyncStoreError::RevisionMismatch { .. }
        | AsyncStoreError::RevisionGap { .. }
        | AsyncStoreError::DuplicateWithConflict(_) => CliErrorCode::RevisionMismatch,
        AsyncStoreError::ValidationFailed(_) | AsyncStoreError::EmptyBatch => CliErrorCode::ValidationFailed,
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

/// Creates an async pool.
///
/// # Errors
/// Returns an error if the connection fails or pragmas cannot be set.
pub async fn create_async_pool(db_path: &Path) -> Result<SqlitePool, AsyncStoreError> {
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

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

/// Bootstraps the async store.
///
/// # Errors
/// Returns an error if the connection fails or migrations fail.
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

/// Fetches the latest revision.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn fetch_latest_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    let revision: Option<i64> = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_optional(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    Ok(revision.unwrap_or(0))
}

/// Gets current revision.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn current_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    fetch_latest_revision(pool).await
}

/// Gets next revision.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn next_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    let current = current_revision(pool).await?;
    Ok(current + 1)
}

/// Appends an event asynchronously.
///
/// # Errors
/// Returns an error on serialization or database failure.
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

/// Appends a batch of events asynchronously.
///
/// # Errors
/// Returns an error if any append fails.
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
    let end_revision = current_revision + i64::try_from(batch_size).unwrap_or(0);
    let mut op_ids = Vec::with_capacity(batch_size);
    let mut last_timestamp = 0i64;

    for (idx, envelope) in ops.into_iter().enumerate() {
        let new_revision = current_revision + 1 + i64::try_from(idx).unwrap_or(0);

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

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub op_id: String,
    pub revision: i64,
    pub timestamp: i64,
    pub payload: String,
}

/// Looks up an existing operation by ID.
///
/// # Errors
/// Returns an error if the query fails or timestamp parsing fails.
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

/// Classifies a duplicate.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn classify_duplicate_async(
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

/// Appends an event idempotently.
///
/// # Errors
/// Returns an error if serialization or database execution fails.
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
                let existing = lookup_existing_op_async(pool, &envelope.op_id).await?;

                match existing {
                    Some(record) => {
                        let kind = classify_duplicate_async(&record, &envelope)?;

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

/// Fetches events since a given revision.
///
/// # Errors
/// Returns an error on query failure.
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

/// Fetches all events.
///
/// # Errors
/// Returns an error on query failure.
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

/// Reads store pragmas asynchronously.
///
/// # Errors
/// Returns an error if any pragma query fails.
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

/// Performs an integrity check on the database.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn integrity_check_async(db_path: &Path) -> Result<Vec<String>, AsyncStoreError> {
    let connection_string = format!("sqlite:{}?mode=ro", db_path.display());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let results: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_all(&pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    pool.close().await;

    Ok(results)
}

/// Opens the database in recovery mode.
///
/// # Errors
/// Returns an error if the connection fails or pragmas cannot be set.
pub async fn open_recovery_mode_async(db_path: &Path) -> Result<SqlitePool, AsyncStoreError> {
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    sqlx::query("PRAGMA journal_mode=DELETE")
        .execute(&pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    sqlx::query("PRAGMA synchronous=NORMAL")
        .execute(&pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_bootstrap_async_store_creates_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store");

        assert_eq!(
            bootstrap.schema_version,
            CURRENT_SCHEMA_VERSION
        );
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
    }

    #[tokio::test]
    async fn test_current_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let rev = current_revision(&pool).await.expect("Failed to get revision");
        assert_eq!(rev, 0);
    }

    #[tokio::test]
    async fn test_fetch_events_since_empty() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let events = fetch_events_since(&pool, 0).await.expect("Failed to fetch events");
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_events_since_with_data() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope1 = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Node 1".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        let envelope2 = EventEnvelope {
            op_id: "test-op-2".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 30.0,
                y: 40.0,
                width: 100.0,
                height: 50.0,
                label: "Node 2".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000002,
        };

        append_event_async(&pool, envelope1, None)
            .await
            .expect("Failed to append event 1");
        append_event_async(&pool, envelope2, None)
            .await
            .expect("Failed to append event 2");

        let events = fetch_events_since(&pool, 1).await.expect("Failed to fetch events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].op_id, "test-op-2");

        let all_events = fetch_events_since(&pool, 0).await.expect("Failed to fetch all events");
        assert_eq!(all_events.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_all_events() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let events = fetch_all_events(&pool).await.expect("Failed to fetch all events");
        assert!(events.is_empty());

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

        append_event_async(&pool, envelope, None)
            .await
            .expect("Failed to append event");

        let events = fetch_all_events(&pool).await.expect("Failed to fetch all events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].op_id, "test-op-1");
    }

    #[tokio::test]
    async fn test_read_store_pragmas_async() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let pragmas = read_store_pragmas_async(&pool).await.expect("Failed to read pragmas");
        
        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 2); // FULL = 2
        assert_eq!(pragmas.wal_autocheckpoint, 1000);
        assert!(pragmas.foreign_keys);
        assert_eq!(pragmas.busy_timeout, 5000);
    }

    #[tokio::test]
    async fn test_integrity_check_async() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store");

        let results = integrity_check_async(&db_path).await.expect("Failed to run integrity check");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "ok");
    }

    #[tokio::test]
    async fn test_open_recovery_mode_async() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store");

        let pool = open_recovery_mode_async(&db_path).await.expect("Failed to open recovery mode");
        
        let pragmas = read_store_pragmas_async(&pool).await.expect("Failed to read pragmas");
        assert_eq!(pragmas.journal_mode, "delete");
        
        pool.close().await;
    }

    #[tokio::test]
    async fn test_classify_duplicate_exact_match() {
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

        let _result = append_event_async(&pool, envelope.clone(), None)
            .await
            .expect("Failed to append event");

        let record = lookup_existing_op_async(&pool, "test-op-1")
            .await
            .expect("Failed to lookup")
            .expect("Record should exist");

        let kind = classify_duplicate_async(&record, &envelope)
            .expect("classify_duplicate_async should succeed");

        assert_eq!(kind, DuplicateKind::Exact);
    }

    #[tokio::test]
    async fn test_classify_duplicate_conflict() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope1 = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Original".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        append_event_async(&pool, envelope1.clone(), None)
            .await
            .expect("Failed to append event");

        let envelope2 = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 50.0,
                y: 60.0,
                width: 200.0,
                height: 100.0,
                label: "Modified".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        let record = lookup_existing_op_async(&pool, "test-op-1")
            .await
            .expect("Failed to lookup")
            .expect("Record should exist");

        let kind = classify_duplicate_async(&record, &envelope2)
            .expect("classify_duplicate_async should succeed");

        assert_eq!(kind, DuplicateKind::Conflict);
    }

    #[tokio::test]
    async fn test_append_idempotent_new_operation() {
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

        let result = append_idempotent_async(&pool, envelope)
            .await
            .expect("Failed to append idempotent");

        assert_eq!(result.revision, 1);
        assert_eq!(result.op_id, "test-op-1");
    }

    #[tokio::test]
    async fn test_append_idempotent_exact_duplicate_returns_existing() {
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
        assert_eq!(result1.timestamp, result2.timestamp);

        let events = fetch_all_events(&pool).await.expect("Failed to fetch all");
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_append_idempotent_conflicting_duplicate_returns_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope1 = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Original".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let envelope2 = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 50.0,
                y: 60.0,
                width: 200.0,
                height: 100.0,
                label: "Modified".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        append_idempotent_async(&pool, envelope1)
            .await
            .expect("Failed to append first");

        let result = append_idempotent_async(&pool, envelope2).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AsyncStoreError::DuplicateWithConflict(_)));
    }
}
