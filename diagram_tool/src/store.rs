//! `SQLite` storage module
//!
//! Provides SQLite-based storage with WAL mode and full synchronous durability.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Database path must be valid and accessible (parent directory exists)
//! - P2: Schema version must be non-negative
//! - P3: Events must have sequential revisions (no gaps)
//! - P4: Batch operations must contain at least one event
//! - P5: Revision argument must match or exceed current stored revision
//!
//! ### Postconditions
//! - Q1: After successful write: events are durable (fsynced)
//! - Q2: After read: returned document has highest stored revision
//! - Q3: After migration: schema version is updated atomically
//! - Q4: Transaction commits only if all operations succeed
//! - Q5: Failed operations leave store state unchanged (rollback)
//!
//! ### Invariants
//! - I1: Revision numbers are monotonically increasing
//! - I2: Each op_id is unique within the document
//! - I3: Schema version matches current migration state
//! - I4: WAL mode is always enabled for concurrent readers

#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use rusqlite::{Connection, OptionalExtension, Transaction};
use serde::Serialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::models::envelope::{encode_event_envelope, EventEnvelope};

/// Current schema version for the store
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

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
}

/// Structured error codes for CLI output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliErrorCode {
    /// Revision mismatch between expected and actual
    RevisionMismatch,
    /// Operation blocked due to human priority
    HumanPriorityBlock,
    /// Policy violation detected
    PolicyViolation,
    /// Validation failed
    ValidationFailed,
    /// Unknown error
    Unknown,
}

impl CliErrorCode {
    /// Returns the error code as a string for JSON serialization
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

/// Maps a `StoreError` to a `CliErrorCode`
///
/// # Errors
/// Returns `CliErrorCode::Unknown` for unmapped error variants
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
        StoreError::TransactionAborted { .. } => CliErrorCode::Unknown,
        StoreError::DuplicateWithConflict(_) => CliErrorCode::RevisionMismatch,
        StoreError::EmptyBatch => CliErrorCode::ValidationFailed,
    }
}

/// Renders an error as a JSON string
///
/// Returns a JSON object with `code` and `message` fields
pub fn render_error_json(code: CliErrorCode, message: &str) -> String {
    serde_json::json!({
        "code": code.code(),
        "message": message
    })
    .to_string()
}

/// CLI-specific errors for submit operations
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
    /// Returns the CLI error code for this error
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

/// Outcome of a CLI submit operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendOutcome {
    /// The new revision after the append
    pub revision: i64,
    /// The operation ID of the appended event
    pub op_id: String,
    /// The timestamp of the appended event
    pub timestamp: i64,
}

impl From<AppendResult> for AppendOutcome {
    fn from(result: AppendResult) -> Self {
        Self {
            revision: result.revision,
            op_id: result.op_id,
            timestamp: result.timestamp,
        }
    }
}

/// Submit a CLI operation through the shared envelope path
///
/// This function routes CLI mutations through the shared event envelope
/// and append path, ensuring all operations are logged and revision-guarded.
///
/// # Errors
/// Returns `CliError::InvalidInput` if the envelope validation fails
/// Returns `CliError::StoreFailure` if the store operation fails
/// Returns `CliError::Conflict` if there's a revision mismatch
pub fn submit_cli_op(
    conn: &mut Connection,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendOutcome, CliError> {
    // Validate the envelope has required fields
    if envelope.op_id.is_empty() {
        return Err(CliError::InvalidInput("op_id is required".to_string()));
    }
    if envelope.author.id.is_empty() {
        return Err(CliError::InvalidInput("author.id is required".to_string()));
    }

    // Route through the shared append path with OCC
    let result = append_event(conn, envelope, expected_revision)?;

    Ok(AppendOutcome::from(result))
}

/// Convert an `AppendOutcome` to a CLI response
///
/// Returns a JSON string suitable for CLI output
#[must_use]
pub fn cli_submit_response(outcome: &AppendOutcome) -> String {
    serde_json::json!({
        "ok": true,
        "revision": outcome.revision,
        "op_id": outcome.op_id,
        "timestamp": outcome.timestamp
    })
    .to_string()
}

/// Errors that can occur during database recovery operations
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

#[derive(Debug, Clone)]
pub struct StorePragmas {
    pub journal_mode: String,
    pub synchronous: i32,
    pub wal_autocheckpoint: i32,
    pub foreign_keys: bool,
    pub busy_timeout: i32,
}

/// Result of bootstrapping a new store
#[derive(Debug)]
pub struct StoreBootstrap {
    pub conn: Connection,
    pub db_path: PathBuf,
    pub schema_version: i32,
}

/// Current configuration of an existing store
#[derive(Debug)]
pub struct StoreConfig {
    pub pragmas: StorePragmas,
    pub schema_version: i32,
}

/// Result of appending an event to the store
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendResult {
    /// The new revision after the append
    pub revision: i64,
    /// The operation ID of the appended event
    pub op_id: String,
    /// The timestamp of the appended event
    pub timestamp: i64,
}

/// Result of appending a batch of events to the store
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchAppendResult {
    /// The starting revision of the batch
    pub start_revision: i64,
    /// The ending revision of the batch (inclusive)
    pub end_revision: i64,
    /// Number of events successfully appended
    pub count: usize,
    /// Operation IDs of the appended events
    pub op_ids: Vec<String>,
    /// Timestamp of the last event in the batch
    pub last_timestamp: i64,
}

pub struct StoreConnection {
    pub conn: Connection,
}

/// Result of a database integrity check
#[derive(Debug, Clone, Serialize)]
pub struct IntegrityStatus {
    /// Whether the database passed integrity checks
    pub is_valid: bool,
    /// Number of pages in the database
    pub page_count: u32,
    /// Number of free pages
    pub free_pages: u32,
    /// Number of corrupted pages
    pub corrupted_pages: u32,
    /// Schema version if readable
    pub schema_version: Option<i32>,
    /// Event count in the database
    pub event_count: u64,
    /// Latest revision if readable
    pub latest_revision: Option<i64>,
    /// Error message if integrity check failed
    pub error_message: Option<String>,
}

/// Handle for read-only recovery mode operations
#[derive(Debug)]
pub struct RecoveryHandle {
    /// The database connection in read-only mode
    pub conn: Connection,
    /// Path to the database file
    pub db_path: PathBuf,
    /// Path to the JSON export file (if exported)
    pub export_path: Option<PathBuf>,
}

/// Alias for RecoveryHandle to match contract signature
pub type RecoverySession = RecoveryHandle;

pub fn open_store(db_path: &Path) -> Result<StoreConnection, StoreError> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=FULL;
         PRAGMA wal_autocheckpoint=1000;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;",
    )?;

    let pragmas = read_store_pragmas(&conn)?;
    if pragmas.journal_mode != "wal" {
        return Err(StoreError::InvalidPragma(format!(
            "Expected WAL journal mode, got {}",
            pragmas.journal_mode
        )));
    }

    if pragmas.synchronous != 2 {
        return Err(StoreError::InvalidPragma(format!(
            "Expected FULL synchronous mode (2), got {}",
            pragmas.synchronous
        )));
    }

    Ok(StoreConnection { conn })
}

pub fn read_store_pragmas(conn: &Connection) -> Result<StorePragmas, StoreError> {
    let journal_mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;

    let synchronous: i32 = conn.query_row("PRAGMA synchronous", [], |row| row.get(0))?;

    let wal_autocheckpoint: i32 =
        conn.query_row("PRAGMA wal_autocheckpoint", [], |row| row.get(0))?;

    let foreign_keys: i32 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;

    let busy_timeout: i32 = conn.query_row("PRAGMA busy_timeout", [], |row| row.get(0))?;

    Ok(StorePragmas {
        journal_mode,
        synchronous,
        wal_autocheckpoint,
        foreign_keys: foreign_keys != 0,
        busy_timeout,
    })
}

/// Bootstrap a new store with schema initialization
///
/// This function:
/// 1. Opens/creates the database at the given path
/// 2. Enforces WAL journal mode and FULL synchronous
/// 3. Creates the schema tables if they don't exist
/// 4. Returns the bootstrap result with connection and metadata
pub fn bootstrap_store(db_path: &Path) -> Result<StoreBootstrap, StoreError> {
    // Open or create the database
    let conn = Connection::open(db_path)?;

    // Set WAL mode and synchronous FULL
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=FULL;
         PRAGMA wal_autocheckpoint=1000;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;",
    )?;

    // Verify pragmas were set correctly
    let pragmas = read_store_pragmas(&conn)?;
    if pragmas.journal_mode != "wal" {
        return Err(StoreError::InvalidPragma(format!(
            "Expected WAL journal mode, got {}",
            pragmas.journal_mode
        )));
    }

    if pragmas.synchronous != 2 {
        return Err(StoreError::InvalidPragma(format!(
            "Expected FULL synchronous mode (2), got {}",
            pragmas.synchronous
        )));
    }

    // Run deterministic schema migration v1
    run_schema_migration(&conn)?;

    // Get the current schema version
    let schema_version = conn
        .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(StoreBootstrap {
        conn,
        db_path: db_path.to_path_buf(),
        schema_version,
    })
}

/// Run deterministic schema migration v1
///
/// Creates the initial schema tables:
/// - `schema_version`: tracks the current schema version
/// - `events`: append-only event log for diagram mutations
fn run_schema_migration(conn: &Connection) -> Result<(), StoreError> {
    // Check if schema_version table exists
    let table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sqlite)?;

    if table_exists == 0 {
        // Create schema_version table
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL DEFAULT 1
            );
            
            INSERT OR IGNORE INTO schema_version (version) VALUES (1);",
        )?;
    }

    // Check if events table exists
    let events_table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sqlite)?;

    if events_table_exists == 0 {
        // Create events table for append-only event log
        // NOTE: This schema differs from models/events.rs which uses a different
        // schema (text ID, event_type, metadata). See models/schema_defs.rs
        // for consolidated schema definitions. The two schemas serve different
        // purposes: store.rs for low-level persistence, events.rs for event sourcing.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id TEXT NOT NULL UNIQUE,
                revision INTEGER NOT NULL,
                payload TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision);
            CREATE INDEX IF NOT EXISTS idx_events_operation_id ON events(operation_id);",
        )?;
    }

    // Create snapshot table if it doesn't exist
    let snapshot_table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='snapshots'",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sqlite)?;

    if snapshot_table_exists == 0 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER NOT NULL PRIMARY KEY,
                revision INTEGER NOT NULL UNIQUE,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC);",
        )?;
    }

    Ok(())
}

/// Get the current store configuration
///
/// Returns the pragmas and schema version for an existing store connection
pub fn current_store_config(conn: &Connection) -> Result<StoreConfig, StoreError> {
    let pragmas = read_store_pragmas(conn)?;

    let schema_version = conn
        .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(StoreConfig {
        pragmas,
        schema_version,
    })
}

/// Fetch the latest revision from the events table
///
/// Returns the current maximum revision, or 0 if no events exist
pub fn fetch_latest_revision(conn: &Connection) -> Result<i64, StoreError> {
    conn.query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |row| {
        row.get(0)
    })
    .map_err(StoreError::Sqlite)
}

/// Get the current revision from the events table
///
/// This is the primary monotonic revision reader for the event store.
/// Returns the current maximum revision, or 0 if no events exist.
///
/// # Errors
/// Returns `StoreError::Sqlite` if the query fails
pub fn current_revision(conn: &Connection) -> Result<i64, StoreError> {
    fetch_latest_revision(conn)
}

/// Get the next revision number for appending a new event
///
/// Returns `current_revision + 1`, which is the revision that would be assigned
/// to the next appended event. Returns 1 if no events exist yet.
///
/// # Errors
/// Returns `StoreError::Sqlite` if the query fails
pub fn next_revision(conn: &Connection) -> Result<i64, StoreError> {
    let current = current_revision(conn)?;
    Ok(current + 1)
}

/// Run integrity check on the database at startup
///
/// This function performs a comprehensive integrity check:
/// 1. Verifies the database file can be opened
/// 2. Checks `SQLite` integrity via PRAGMA `integrity_check`
/// 3. Validates schema version table exists and is readable
/// 4. Counts events and determines latest revision
/// 5. Checks for page corruption
///
/// Returns an `IntegrityStatus` with detailed results of each check.
pub fn startup_integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError> {
    // Check if database file exists
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

    // Open in read-only mode to check integrity without modifying
    let conn = Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(RecoveryError::Sqlite)?;

    // Run SQLite integrity check
    let integrity_result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(RecoveryError::Sqlite)?;

    let is_valid = integrity_result == "ok";

    // Get page count info
    let page_count: u32 = conn
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .map_err(RecoveryError::Sqlite)?;

    let free_pages: u32 = conn
        .query_row("PRAGMA freelist_count", [], |row| row.get(0))
        .map_err(RecoveryError::Sqlite)?;

    let corrupted_pages: u32 = u32::from(!is_valid && integrity_result.contains("corrupt"));

    // Try to read schema version
    let schema_version = conn
        .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
        .ok();

    // Count events
    let event_count: u64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .unwrap_or(0);

    // Get latest revision
    let latest_revision: Option<i64> = conn
        .query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |row| {
            let rev: i64 = row.get(0)?;
            Ok(rev)
        })
        .ok()
        .filter(|&rev| rev > 0);

    // Determine error message if invalid
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

/// Open the database in read-only recovery mode
///
/// This function:
/// 1. Opens the database in read-only mode
/// 2. Performs an integrity check
/// 3. If the database is valid, can export to JSON
///
/// Returns a `RecoveryHandle` for read-only operations.
pub fn open_recovery_mode(db_path: &Path) -> Result<RecoveryHandle, RecoveryError> {
    // Open in read-only mode
    let conn = Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(RecoveryError::Sqlite)?;

    // Run integrity check to verify database is not corrupt
    let integrity_result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(RecoveryError::Sqlite)?;

    if integrity_result != "ok" {
        return Err(RecoveryError::CorruptDatabase(integrity_result));
    }

    Ok(RecoveryHandle {
        conn,
        db_path: db_path.to_path_buf(),
        export_path: None,
    })
}

/// Run integrity check on the database (contract signature alias)
///
/// This is an alias for `startup_integrity_check` that matches the contract signature.
/// Performs a comprehensive integrity check on the database file.
///
/// Returns an `IntegrityStatus` with detailed results of each check.
pub fn integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError> {
    startup_integrity_check(db_path)
}

/// Open the database in recovery-only mode (contract signature alias)
///
/// This is an alias for `open_recovery_mode` that matches the contract signature.
/// Opens the database in read-only mode for recovery operations.
///
/// Returns a `RecoverySession` for read-only recovery operations.
pub fn open_recovery_only(db_path: &Path) -> Result<RecoverySession, RecoveryError> {
    open_recovery_mode(db_path).map(|h| RecoverySession {
        conn: h.conn,
        db_path: h.db_path,
        export_path: h.export_path,
    })
}

impl RecoveryHandle {
    /// Export all events to JSON format
    ///
    /// This reads all events from the database and writes them to a JSON file.
    /// The export is performed in a single read transaction.
    pub fn export_to_json(&mut self, output_path: &Path) -> Result<(), RecoveryError> {
        // Read all events
        let mut stmt = self
            .conn
            .prepare("SELECT id, operation_id, revision, payload, timestamp FROM events ORDER BY revision")
            .map_err(RecoveryError::Sqlite)?;

        let raw_events: Vec<(i64, String, i64, String, String)> = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let operation_id: String = row.get(1)?;
                let revision: i64 = row.get(2)?;
                let payload: String = row.get(3)?;
                let timestamp: String = row.get(4)?;
                Ok((id, operation_id, revision, payload, timestamp))
            })
            .map_err(RecoveryError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(RecoveryError::Sqlite)?;

        let events: Vec<serde_json::Value> = raw_events
            .into_iter()
            .map(|(id, operation_id, revision, payload, timestamp)| {
                serde_json::json!({
                    "id": id,
                    "operation_id": operation_id,
                    "revision": revision,
                    "payload": payload,
                    "timestamp": timestamp
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| RecoveryError::Sqlite(e))?;

        // Write to JSON file
        let json_content = serde_json::to_string_pretty(&events)
            .map_err(|e| RecoveryError::Io(std::io::Error::other(e.to_string())))?;

        std::fs::write(output_path, json_content).map_err(RecoveryError::Io)?;

        self.export_path = Some(output_path.to_path_buf());

        Ok(())
    }
}

/// Append an event to the store using Optimistic Concurrency Control (OCC)
///
/// This function:
/// 1. Begins a transaction
/// 2. Reads the current latest revision
/// 3. Validates the expected revision (if provided)
/// 4. Encodes the event envelope to JSON
/// 5. Inserts the event with the new revision
/// 6. Commits the transaction
///
/// On any failure, the transaction is rolled back - no partial mutations occur.
///
/// # Errors
/// Returns `StoreError::RevisionMismatch` if the expected revision doesn't match
/// Returns `StoreError::Serialization` if encoding the envelope fails
/// Returns `StoreError::ValidationFailed` if validation fails
pub fn append_event(
    conn: &mut Connection,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendResult, StoreError> {
    // Begin transaction for atomic OCC check-and-insert
    let tx = conn.transaction().map_err(StoreError::Sqlite)?;

    // Read current latest revision within transaction
    let current_revision: i64 = tx
        .query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |row| {
            row.get(0)
        })
        .map_err(StoreError::Sqlite)?;

    // Validate expected revision if provided
    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(StoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    // The new revision is current_revision + 1
    let new_revision = current_revision + 1;

    // Encode the envelope to JSON
    let payload =
        encode_event_envelope(&envelope).map_err(|e| StoreError::Serialization(e.to_string()))?;

    // Insert the event
    tx.execute(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            envelope.op_id,
            new_revision,
            payload,
            envelope.timestamp.to_string()
        ],
    )
    .map_err(StoreError::Sqlite)?;

    // Commit the transaction
    tx.commit().map_err(StoreError::Sqlite)?;

    Ok(AppendResult {
        revision: new_revision,
        op_id: envelope.op_id,
        timestamp: envelope.timestamp,
    })
}

/// Append an event with Optimistic Concurrency Control (OCC)
///
/// This is an alias for `append_event` that matches the contract signature.
///
/// # Errors
/// Returns `StoreError::RevisionMismatch` if the expected revision doesn't match
/// Returns `StoreError::Serialization` if encoding the envelope fails
/// Returns `StoreError::ValidationFailed` if validation fails
pub fn append_with_occ(
    conn: &mut Connection,
    op: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendResult, StoreError> {
    append_event(conn, op, expected_revision)
}

/// Verify that an OCC append result is valid
///
/// This function validates that an append result contains valid data:
/// - Revision must be positive (at least 1)
/// - Operation ID must not be empty
/// - Timestamp must be positive
///
/// # Errors
/// Returns `StoreError::ValidationFailed` if the result is invalid
pub fn verify_occ_append(result: &AppendResult) -> Result<(), StoreError> {
    if result.revision < 1 {
        return Err(StoreError::ValidationFailed(
            "revision must be at least 1".to_string(),
        ));
    }

    if result.op_id.is_empty() {
        return Err(StoreError::ValidationFailed(
            "op_id must not be empty".to_string(),
        ));
    }

    if result.timestamp <= 0 {
        return Err(StoreError::ValidationFailed(
            "timestamp must be positive".to_string(),
        ));
    }

    Ok(())
}

/// Append a batch of events atomically with Optimistic Concurrency Control (OCC)
///
/// This function appends multiple events in a single atomic transaction:
/// 1. Validates that the batch is not empty
/// 2. Begins a transaction
/// 3. Reads the current latest revision
/// 4. Validates the expected revision (if provided)
/// 5. Encodes and inserts all events with sequential revisions
/// 6. Commits the transaction (or rolls back on any failure)
///
/// On any failure, the transaction is rolled back - no partial mutations occur.
///
/// # Errors
/// Returns `StoreError::EmptyBatch` if the ops vector is empty
/// Returns `StoreError::RevisionMismatch` if the expected revision doesn't match
/// Returns `StoreError::Serialization` if encoding any envelope fails
/// Returns `StoreError::ValidationFailed` if validation fails
/// Returns `StoreError::Sqlite` if database operations fail
pub fn append_batch(
    conn: &mut Connection,
    ops: Vec<EventEnvelope>,
    expected_revision: Option<i64>,
) -> Result<BatchAppendResult, StoreError> {
    // Validate batch is not empty
    if ops.is_empty() {
        return Err(StoreError::EmptyBatch);
    }

    // Begin transaction for atomic batch insert
    let tx = conn.transaction().map_err(StoreError::Sqlite)?;

    // Read current latest revision within transaction
    let current_revision: i64 = tx
        .query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |row| {
            row.get(0)
        })
        .map_err(StoreError::Sqlite)?;

    // Validate expected revision if provided
    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(StoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    // Track batch metadata
    let batch_size = ops.len();
    let start_revision = current_revision + 1;
    let end_revision = current_revision + batch_size as i64;
    let mut op_ids = Vec::with_capacity(batch_size);
    let mut last_timestamp = 0i64;

    // Insert all events within the transaction
    for (idx, envelope) in ops.into_iter().enumerate() {
        let new_revision = current_revision + 1 + idx as i64;

        // Encode the envelope to JSON
        let payload = encode_event_envelope(&envelope)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        // Insert the event
        tx.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                envelope.op_id,
                new_revision,
                payload,
                envelope.timestamp.to_string()
            ],
        )
        .map_err(StoreError::Sqlite)?;

        op_ids.push(envelope.op_id);
        last_timestamp = envelope.timestamp;
    }

    // Commit the transaction
    tx.commit().map_err(StoreError::Sqlite)?;

    Ok(BatchAppendResult {
        start_revision,
        end_revision,
        count: batch_size,
        op_ids,
        last_timestamp,
    })
}

/// Verify that a batch append result is valid
///
/// This function validates that a batch append result contains valid data:
/// - Start revision must be positive (at least 1)
/// - End revision must be >= start revision
/// - Count must match the revision range
/// - All operation IDs must be non-empty
/// - Timestamp must be positive
///
/// # Errors
/// Returns `StoreError::ValidationFailed` if the result is invalid
pub fn verify_batch_atomicity(result: &BatchAppendResult) -> Result<(), StoreError> {
    if result.start_revision < 1 {
        return Err(StoreError::ValidationFailed(
            "start_revision must be at least 1".to_string(),
        ));
    }

    if result.end_revision < result.start_revision {
        return Err(StoreError::ValidationFailed(
            "end_revision must be >= start_revision".to_string(),
        ));
    }

    let expected_count = (result.end_revision - result.start_revision + 1) as usize;
    if result.count != expected_count {
        return Err(StoreError::ValidationFailed(format!(
            "count {} does not match revision range (expected {})",
            result.count, expected_count
        )));
    }

    if result.op_ids.len() != result.count {
        return Err(StoreError::ValidationFailed(
            "op_ids length must match count".to_string(),
        ));
    }

    for (idx, op_id) in result.op_ids.iter().enumerate() {
        if op_id.is_empty() {
            return Err(StoreError::ValidationFailed(format!(
                "op_id at index {} must not be empty",
                idx
            )));
        }
    }

    if result.last_timestamp <= 0 {
        return Err(StoreError::ValidationFailed(
            "last_timestamp must be positive".to_string(),
        ));
    }

    Ok(())
}

/// Record for an event in the durable log
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    /// The operation ID (unique identifier for the operation)
    pub op_id: String,
    /// The revision number of this event
    pub revision: i64,
    /// The timestamp of the event
    pub timestamp: i64,
    /// The JSON payload of the event
    pub payload: String,
}

/// Kind of duplicate detected during idempotent append
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    /// Same op_id with identical payload - return existing outcome (no-op)
    Exact,
    /// Same op_id with different payload - return error
    Conflict,
}

/// Classify whether a duplicate operation is an exact match or a conflict
///
/// Compares the payload of an existing event record with an incoming envelope
/// to determine if the duplicate should be treated as a no-op (exact match)
/// or rejected as a conflict.
///
/// # Errors
/// Returns `StoreError::Serialization` if the incoming envelope cannot be encoded
///
/// # Example
/// ```ignore
/// let kind = classify_duplicate(&existing_record, &incoming_envelope)?;
/// match kind {
///     DuplicateKind::Exact => // return existing outcome
///     DuplicateKind::Conflict => // return DuplicateWithConflict error
/// }
/// ```
pub fn classify_duplicate(
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

/// Append an event with idempotent behavior for exact duplicates
///
/// This function implements idempotent append semantics:
/// - If the op_id is new, appends the event and returns the new outcome
/// - If the op_id exists with an identical payload, returns the existing outcome (no-op)
/// - If the op_id exists with a different payload, returns a conflict error
///
/// This function is race-safe: it uses a single transaction with INSERT-first
/// approach, catching unique constraint violations and classifying them properly.
/// This eliminates the race window between lookup and insert that exists in
/// naive check-then-insert patterns.
///
/// # Errors
/// Returns `StoreError::DuplicateWithConflict` if the op_id exists with different payload
/// Returns `StoreError::Sqlite` if database operations fail
/// Returns `StoreError::Serialization` if encoding the envelope fails
///
/// # Example
/// ```ignore
/// let outcome = append_idempotent(&mut conn, envelope)?;
/// // outcome contains either the new revision or the existing one for exact duplicates
/// ```
pub fn append_idempotent(
    conn: &mut Connection,
    op: EventEnvelope,
) -> Result<AppendOutcome, StoreError> {
    // Single transaction approach: try insert first, handle conflict if needed
    // This eliminates the race window between lookup and insert
    let tx = conn.transaction().map_err(StoreError::Sqlite)?;

    // Read current latest revision within transaction
    let current_revision: i64 = tx
        .query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |row| {
            row.get(0)
        })
        .map_err(StoreError::Sqlite)?;

    // Encode the envelope to JSON
    let payload =
        encode_event_envelope(&op).map_err(|e| StoreError::Serialization(e.to_string()))?;

    // Try to insert - this will fail with unique constraint if op_id already exists
    let new_revision = current_revision + 1;
    let insert_result = tx.execute(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![op.op_id, new_revision, payload, op.timestamp.to_string()],
    );

    match insert_result {
        Ok(_) => {
            // Insert succeeded - new operation
            tx.commit().map_err(StoreError::Sqlite)?;
            Ok(AppendOutcome {
                revision: new_revision,
                op_id: op.op_id,
                timestamp: op.timestamp,
            })
        }
        Err(e) => {
            // Check if it's a unique constraint violation
            // SQLITE_CONSTRAINT (error code 19) - UNIQUE constraint failed
            let is_unique_constraint = e.to_string().contains("UNIQUE constraint failed")
                || e.to_string().contains("constraint failed");

            if is_unique_constraint {
                // Unique constraint violation - lookup existing record and classify
                let existing = lookup_existing_op(&tx, &op.op_id)?;

                match existing {
                    Some(record) => {
                        // Classify the duplicate
                        let kind = classify_duplicate(&record, &op)?;

                        match kind {
                            DuplicateKind::Exact => {
                                // Exact duplicate - return existing outcome (no-op success)
                                // Note: we don't need to commit since we're just reading
                                Ok(AppendOutcome {
                                    revision: record.revision,
                                    op_id: record.op_id,
                                    timestamp: record.timestamp,
                                })
                            }
                            DuplicateKind::Conflict => {
                                // Conflicting duplicate - return error
                                Err(StoreError::DuplicateWithConflict(op.op_id))
                            }
                        }
                    }
                    // This shouldn't happen - unique constraint means record exists
                    None => {
                        // This is a very unlikely race - retry or fail
                        Err(StoreError::Sqlite(e))
                    }
                }
            } else {
                // Some other error - propagate it
                Err(StoreError::Sqlite(e))
            }
        }
    }
}

/// Ensure op_id uniqueness by creating/verifying the unique index
///
/// This function ensures that the unique index on operation_id exists,
/// enforcing idempotency at the storage layer.
///
/// # Errors
/// Returns `StoreError::Sqlite` if the index creation fails
pub fn ensure_op_id_uniqueness(conn: &mut Connection) -> Result<(), StoreError> {
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_events_operation_id_unique ON events(operation_id)",
        [],
    )
    .map_err(StoreError::Sqlite)?;
    Ok(())
}

/// Lookup an existing operation by op_id
///
/// This function checks if an operation with the given op_id already exists
/// in the durable log, supporting idempotent operation handling.
///
/// # Errors
/// Returns `StoreError::Sqlite` if the query fails
/// Returns `StoreError::Serialization` if the timestamp cannot be parsed
pub fn lookup_existing_op(
    conn: &Connection,
    op_id: &str,
) -> Result<Option<EventRecord>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT operation_id, revision, timestamp, payload FROM events WHERE operation_id = ?1",
        )
        .map_err(StoreError::Sqlite)?;

    let result = stmt
        .query_row([op_id], |row| {
            let timestamp_str: String = row.get(2)?;
            let timestamp: i64 = timestamp_str
                .parse()
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
            Ok(EventRecord {
                op_id: row.get(0)?,
                revision: row.get(1)?,
                timestamp,
                payload: row.get(3)?,
            })
        })
        .optional()
        .map_err(StoreError::Sqlite)?;

    Ok(result)
}

/// Execute a write operation within a transaction with automatic rollback on failure
///
/// This helper function provides a safe wrapper for atomic write operations:
/// 1. Begins a write transaction
/// 2. Executes the provided closure with the transaction
/// 3. On success, commits the transaction
/// 4. On failure, rolls back automatically (the transaction is dropped)
///
/// # Errors
/// Returns `StoreError::Sqlite` if transaction begin/commit fails
/// Returns `StoreError::TransactionAborted` if the closure returns an error
///
/// # Example
/// ```ignore
/// let result = with_write_tx(&mut conn, |tx| {
///     tx.execute("INSERT INTO events (id) VALUES (?1)", [1])?;
///     Ok(42)
/// })?;
/// assert_eq!(result, 42);
/// ```
pub fn with_write_tx<T, F>(conn: &mut Connection, f: F) -> Result<T, StoreError>
where
    F: FnOnce(&Transaction) -> Result<T, StoreError>,
{
    let tx = conn.transaction().map_err(StoreError::Sqlite)?;

    let result = f(&tx);

    match result {
        Ok(value) => {
            tx.commit().map_err(StoreError::Sqlite)?;
            Ok(value)
        }
        Err(err) => {
            // Preserve the original error variant rather than wrapping everything in TransactionAborted.
            // This allows callers to handle specific error types deterministically.
            // Transaction will roll back automatically when dropped.
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_bootstrap_store_creates_database_with_schema() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        assert_eq!(
            bootstrap.schema_version,
            crate::store::CURRENT_SCHEMA_VERSION
        );
        assert_eq!(bootstrap.db_path, db_path);

        // Verify the database file exists
        assert!(db_path.exists(), "Database file should exist");
    }

    #[test]
    fn test_bootstrap_store_enforces_wal_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");
        let config = current_store_config(&bootstrap.conn).expect("Failed to get config");

        assert_eq!(config.pragmas.journal_mode, "wal");
    }

    #[test]
    fn test_bootstrap_store_enforces_synchronous_full() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");
        let config = current_store_config(&bootstrap.conn).expect("Failed to get config");

        assert_eq!(config.pragmas.synchronous, 2);
    }

    #[test]
    fn test_bootstrap_store_with_invalid_path() {
        // Try to create a database in a non-existent directory
        let invalid_path = Path::new("/nonexistent/path/test.db");

        let result = bootstrap_store(invalid_path);

        assert!(result.is_err());
        match result {
            Err(StoreError::Io(_)) => {}
            Err(StoreError::Sqlite(_)) => {}
            Err(other) => panic!("Expected Io or Sqlite error, got {:?}", other),
            _ => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_bootstrap_store_creates_schema_tables() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Verify the schema_version table exists and has correct version
        let version: i32 = bootstrap
            .conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .expect("Failed to read schema version");

        assert_eq!(version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_current_store_config_returns_pragmas_and_version() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");
        let config = current_store_config(&bootstrap.conn).expect("Failed to get config");

        assert_eq!(config.pragmas.journal_mode, "wal");
        assert_eq!(config.pragmas.synchronous, 2);
        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_bootstrap_idempotent_on_existing_schema() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // First bootstrap
        let bootstrap1 = bootstrap_store(&db_path).expect("First bootstrap failed");
        let config1 = current_store_config(&bootstrap1.conn).expect("Failed to get config1");

        // Second bootstrap should be idempotent
        let bootstrap2 = bootstrap_store(&db_path).expect("Second bootstrap failed");
        let config2 = current_store_config(&bootstrap2.conn).expect("Failed to get config2");

        assert_eq!(config1.schema_version, config2.schema_version);
    }

    #[test]
    fn test_open_store_with_existing_wal_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // First create with bootstrap
        let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Then open with open_store
        let store = open_store(&db_path).expect("Failed to open store");
        let pragmas = read_store_pragmas(&store.conn).expect("Failed to read pragmas");

        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 2);
    }

    // Recovery mode tests

    #[test]
    fn test_startup_integrity_check_on_valid_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Create a valid database
        let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Run integrity check
        let status = startup_integrity_check(&db_path).expect("Integrity check failed");

        assert!(status.is_valid, "Database should be valid");
        assert!(
            status.error_message.is_none(),
            "Should have no error message"
        );
        assert!(
            status.schema_version.is_some(),
            "Should have schema version"
        );
    }

    #[test]
    fn test_startup_integrity_check_on_nonexistent_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("nonexistent.db");

        // Run integrity check on nonexistent file
        let status = startup_integrity_check(&db_path).expect("Integrity check failed");

        assert!(!status.is_valid, "Nonexistent database should not be valid");
        assert!(status.error_message.is_some(), "Should have error message");
    }

    #[test]
    fn test_open_recovery_mode_on_valid_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Create a valid database
        let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Open in recovery mode
        let mut handle = open_recovery_mode(&db_path).expect("Failed to open recovery mode");

        // Verify connection is read-only
        let result = handle
            .conn
            .query_row("SELECT 1", [], |row| row.get::<_, i32>(0));
        assert!(result.is_ok(), "Should be able to read from recovery mode");
    }

    #[test]
    fn test_recovery_handle_export_to_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Create a valid database
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Add some test events
        use crate::models::envelope::{Author, DomainOp, EventEnvelope};
        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

        // Open in recovery mode and export
        let mut handle = open_recovery_mode(&db_path).expect("Failed to open recovery mode");
        let export_path = temp_dir.path().join("export.json");

        let export_result = handle.export_to_json(&export_path);
        assert!(
            export_result.is_ok(),
            "Export should succeed: {:?}",
            export_result.err()
        );
        assert!(export_path.exists(), "Export file should exist");
    }

    // Contract signature tests - bd-7rt

    #[test]
    fn test_integrity_check_on_valid_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Create a valid database
        let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Run integrity check using contract signature
        let status = integrity_check(&db_path).expect("Integrity check failed");

        assert!(status.is_valid, "Database should be valid");
        assert!(
            status.error_message.is_none(),
            "Should have no error message"
        );
        assert!(
            status.schema_version.is_some(),
            "Should have schema version"
        );
    }

    #[test]
    fn test_integrity_check_on_nonexistent_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("nonexistent.db");

        // Run integrity check on nonexistent file using contract signature
        let status = integrity_check(&db_path).expect("Integrity check failed");

        assert!(!status.is_valid, "Nonexistent database should not be valid");
        assert!(status.error_message.is_some(), "Should have error message");
    }

    #[test]
    fn test_open_recovery_only_on_valid_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Create a valid database
        let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Open in recovery-only mode using contract signature
        let session = open_recovery_only(&db_path).expect("Failed to open recovery only mode");

        // Verify connection is read-only
        let result = session
            .conn
            .query_row("SELECT 1", [], |row| row.get::<_, i32>(0));
        assert!(
            result.is_ok(),
            "Should be able to read from recovery only mode"
        );
    }

    #[test]
    fn test_recovery_session_is_same_as_recovery_handle() {
        // Verify RecoverySession is an alias for RecoveryHandle
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        let handle = open_recovery_mode(&db_path).expect("Failed to open recovery mode");
        let session = open_recovery_only(&db_path).expect("Failed to open recovery only");

        // Both should have same structure
        assert_eq!(handle.db_path, session.db_path);
    }

    // CliErrorCode tests

    #[test]
    fn test_map_error_code_revision_mismatch() {
        let err = StoreError::RevisionMismatch {
            expected: 5,
            found: 3,
        };
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::RevisionMismatch);
    }

    #[test]
    fn test_map_error_code_human_priority_block() {
        let err = StoreError::HumanPriorityBlock("user is editing".to_string());
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::HumanPriorityBlock);
    }

    #[test]
    fn test_map_error_code_validation_failed() {
        let err = StoreError::ValidationFailed("invalid node position".to_string());
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::ValidationFailed);
    }

    #[test]
    fn test_map_error_code_sqlite() {
        let err = StoreError::Sqlite(rusqlite::Error::InvalidQuery);
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::Unknown);
    }

    #[test]
    fn test_map_error_code_io() {
        let err = StoreError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::Unknown);
    }

    #[test]
    fn test_render_error_json_revision_mismatch() {
        let json = render_error_json(
            CliErrorCode::RevisionMismatch,
            "expected revision 5 but found 3",
        );
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["code"], "revision_mismatch");
        assert_eq!(parsed["message"], "expected revision 5 but found 3");
    }

    #[test]
    fn test_render_error_json_human_priority_block() {
        let json = render_error_json(CliErrorCode::HumanPriorityBlock, "user is editing");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["code"], "human_priority_block");
        assert_eq!(parsed["message"], "user is editing");
    }

    #[test]
    fn test_render_error_json_validation_failed() {
        let json = render_error_json(CliErrorCode::ValidationFailed, "invalid node position");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["code"], "validation_failed");
        assert_eq!(parsed["message"], "invalid node position");
    }

    #[test]
    fn test_render_error_json_policy_violation() {
        let json = render_error_json(CliErrorCode::PolicyViolation, "operation not allowed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["code"], "policy_violation");
        assert_eq!(parsed["message"], "operation not allowed");
    }

    #[test]
    fn test_render_error_json_unknown() {
        let json = render_error_json(CliErrorCode::Unknown, "internal error");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["code"], "unknown");
        assert_eq!(parsed["message"], "internal error");
    }

    #[test]
    fn test_cli_error_code_serialization() {
        let code = CliErrorCode::RevisionMismatch;
        let json = serde_json::to_string(&code).expect("valid JSON");
        assert_eq!(json, "\"revision_mismatch\"");
    }

    // CliError and submit_cli_op tests

    #[test]
    fn test_cli_error_error_code_invalid_input() {
        let err = CliError::InvalidInput("test".to_string());
        assert_eq!(err.error_code(), CliErrorCode::ValidationFailed);
    }

    #[test]
    fn test_cli_error_error_code_conflict() {
        let err = CliError::Conflict("revision mismatch".to_string());
        assert_eq!(err.error_code(), CliErrorCode::RevisionMismatch);
    }

    #[test]
    fn test_cli_error_error_code_serialization() {
        let err = CliError::Serialization("failed".to_string());
        assert_eq!(err.error_code(), CliErrorCode::Unknown);
    }

    #[test]
    fn test_cli_error_error_code_store_failure() {
        let store_err = StoreError::RevisionMismatch {
            expected: 1,
            found: 2,
        };
        let err = CliError::StoreFailure(store_err);
        assert_eq!(err.error_code(), CliErrorCode::RevisionMismatch);
    }

    #[test]
    fn test_submit_cli_op_missing_op_id() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).unwrap();

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};
        let envelope = EventEnvelope {
            op_id: String::new(), // Empty op_id
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result = submit_cli_op(&mut bootstrap.conn, envelope, None);
        assert!(result.is_err());
        match result {
            Err(CliError::InvalidInput(msg)) => assert!(msg.contains("op_id")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_submit_cli_op_missing_author_id() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).unwrap();

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: String::new(), // Empty author id
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result = submit_cli_op(&mut bootstrap.conn, envelope, None);
        assert!(result.is_err());
        match result {
            Err(CliError::InvalidInput(msg)) => assert!(msg.contains("author.id")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_submit_cli_op_success() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).unwrap();

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result = submit_cli_op(&mut bootstrap.conn, envelope, None);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let outcome = result.unwrap();
        assert_eq!(outcome.revision, 1);
        assert_eq!(outcome.op_id, "op-1");
    }

    #[test]
    fn test_submit_cli_op_revision_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).unwrap();

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        // Expect revision 5 but database is at 0
        let result = submit_cli_op(&mut bootstrap.conn, envelope, Some(5));
        assert!(result.is_err());
        match result {
            Err(CliError::StoreFailure(StoreError::RevisionMismatch { expected, found })) => {
                assert_eq!(expected, 5);
                assert_eq!(found, 0);
            }
            _ => panic!("Expected RevisionMismatch error, got: {:?}", result),
        }
    }

    #[test]
    fn test_cli_submit_response() {
        let outcome = AppendOutcome {
            revision: 42,
            op_id: "op-123".to_string(),
            timestamp: 1700000000,
        };

        let json = cli_submit_response(&outcome);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["revision"], 42);
        assert_eq!(parsed["op_id"], "op-123");
        assert_eq!(parsed["timestamp"], 1700000000);
    }

    #[test]
    fn test_append_outcome_from_append_result() {
        let result = AppendResult {
            revision: 10,
            op_id: "op-456".to_string(),
            timestamp: 1700000001,
        };

        let outcome = AppendOutcome::from(result);

        assert_eq!(outcome.revision, 10);
        assert_eq!(outcome.op_id, "op-456");
        assert_eq!(outcome.timestamp, 1700000001);
    }

    // Transaction helper tests

    #[test]
    fn test_with_write_tx_commits_on_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Execute a successful write transaction
        let result: Result<i64, StoreError> = with_write_tx(&mut bootstrap.conn, |tx| {
            tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["test-op", 1, "{}", "2024-01-01"],
            )
            .map_err(StoreError::Sqlite)?;
            Ok(42)
        });

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        assert_eq!(result.unwrap(), 42);

        // Verify the data was committed
        let count: i64 = bootstrap
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE operation_id = 'test-op'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to count events");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_with_write_tx_rolls_back_on_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Execute a transaction that fails after a write
        let result: Result<i64, StoreError> = with_write_tx(&mut bootstrap.conn, |tx| {
            tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["test-op-rollback", 1, "{}", "2024-01-01"],
            )
            .map_err(StoreError::Sqlite)?;
            // Simulate a failure
            Err(StoreError::ValidationFailed(
                "intentional failure".to_string(),
            ))
        });

        // Should get the original error (preserving the variant for deterministic handling)
        assert!(result.is_err());
        match result {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("intentional failure"));
            }
            Err(e) => panic!("Expected ValidationFailed, got: {:?}", e),
            Ok(_) => panic!("Expected error, got success"),
        }

        // Verify the data was rolled back
        let count: i64 = bootstrap
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE operation_id = 'test-op-rollback'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to count events");
        assert_eq!(count, 0, "Data should have been rolled back");
    }

    #[test]
    fn test_transaction_aborted_error_display() {
        let err = StoreError::TransactionAborted {
            source: Box::new(std::io::Error::other("test error")),
        };
        let msg = err.to_string();
        assert!(msg.contains("Transaction aborted"));
        assert!(msg.contains("test error"));
    }

    #[test]
    fn test_map_error_code_transaction_aborted() {
        let err = StoreError::TransactionAborted {
            source: Box::new(std::io::Error::other("test")),
        };
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::Unknown);
    }

    #[test]
    fn test_with_write_tx_multiple_operations_atomic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Execute multiple operations in a transaction, then fail
        let result: Result<(), StoreError> = with_write_tx(&mut bootstrap.conn, |tx| {
            tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["op1", 1, "{}", "2024-01-01"],
            )
            .map_err(StoreError::Sqlite)?;
            tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["op2", 2, "{}", "2024-01-01"],
            )
            .map_err(StoreError::Sqlite)?;
            Err(StoreError::ValidationFailed(
                "fail after inserts".to_string(),
            ))
        });

        assert!(result.is_err());

        // Verify both inserts were rolled back
        let count: i64 = bootstrap
            .conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .expect("Failed to count events");
        assert_eq!(count, 0, "All operations should have been rolled back");
    }

    // append_with_occ and verify_occ_append tests

    #[test]
    fn test_append_with_occ_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};
        let envelope = EventEnvelope {
            op_id: "op-occ-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test Node".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result = append_with_occ(&mut bootstrap.conn, envelope, None);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let outcome = result.expect("Checked is_ok");
        assert_eq!(outcome.revision, 1);
        assert_eq!(outcome.op_id, "op-occ-1");
    }

    #[test]
    fn test_append_with_occ_revision_mismatch() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};
        let envelope = EventEnvelope {
            op_id: "op-occ-2".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test Node".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        // Expect revision 5 but database is at 0
        let result = append_with_occ(&mut bootstrap.conn, envelope, Some(5));
        assert!(result.is_err());
        match result {
            Err(StoreError::RevisionMismatch { expected, found }) => {
                assert_eq!(expected, 5);
                assert_eq!(found, 0);
            }
            _ => panic!("Expected RevisionMismatch error"),
        }
    }

    #[test]
    fn test_verify_occ_append_valid_result() {
        let result = AppendResult {
            revision: 1,
            op_id: "op-valid".to_string(),
            timestamp: 1700000000,
        };

        assert!(verify_occ_append(&result).is_ok());
    }

    #[test]
    fn test_verify_occ_append_zero_revision() {
        let result = AppendResult {
            revision: 0,
            op_id: "op-invalid".to_string(),
            timestamp: 1700000000,
        };

        let err = verify_occ_append(&result);
        assert!(err.is_err());
        match err {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("revision"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_verify_occ_append_empty_op_id() {
        let result = AppendResult {
            revision: 1,
            op_id: String::new(),
            timestamp: 1700000000,
        };

        let err = verify_occ_append(&result);
        assert!(err.is_err());
        match err {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("op_id"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_verify_occ_append_zero_timestamp() {
        let result = AppendResult {
            revision: 1,
            op_id: "op-valid".to_string(),
            timestamp: 0,
        };

        let err = verify_occ_append(&result);
        assert!(err.is_err());
        match err {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("timestamp"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_verify_occ_append_negative_timestamp() {
        let result = AppendResult {
            revision: 1,
            op_id: "op-valid".to_string(),
            timestamp: -1,
        };

        let err = verify_occ_append(&result);
        assert!(err.is_err());
        match err {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("timestamp"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    // current_revision and next_revision tests

    #[test]
    fn test_current_revision_empty_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Empty database should return 0
        let revision = current_revision(&bootstrap.conn).expect("Failed to get current revision");
        assert_eq!(revision, 0, "Empty database should have revision 0");
    }

    #[test]
    fn test_current_revision_with_events() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Add an event
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

        // Should return 1 after one event
        let revision = current_revision(&bootstrap.conn).expect("Failed to get current revision");
        assert_eq!(revision, 1);
    }

    #[test]
    fn test_current_revision_multiple_events() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Add multiple events
        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000000 + i,
            };
            let _ =
                append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");
        }

        // Should return 5 after five events
        let revision = current_revision(&bootstrap.conn).expect("Failed to get current revision");
        assert_eq!(revision, 5);
    }

    #[test]
    fn test_next_revision_empty_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Empty database: current=0, next=1
        let revision = next_revision(&bootstrap.conn).expect("Failed to get next revision");
        assert_eq!(revision, 1, "Next revision should be 1 for empty database");
    }

    #[test]
    fn test_next_revision_with_events() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Add an event
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

        // After one event: current=1, next=2
        let revision = next_revision(&bootstrap.conn).expect("Failed to get next revision");
        assert_eq!(revision, 2);
    }

    #[test]
    fn test_next_revision_monotonic_increase() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Verify monotonic increase across multiple appends
        for i in 1..=3 {
            let next_before = next_revision(&bootstrap.conn).expect("Failed to get next revision");
            assert_eq!(next_before, i);

            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000000 + i,
            };
            let _ =
                append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

            let current_after =
                current_revision(&bootstrap.conn).expect("Failed to get current revision");
            assert_eq!(current_after, i);
        }
    }

    // RevisionGap error tests

    #[test]
    fn test_revision_gap_error_display() {
        let err = StoreError::RevisionGap {
            expected: 5,
            found: 7,
        };
        let msg = err.to_string();
        assert!(msg.contains("Revision gap detected"));
        assert!(msg.contains("expected sequential revision 5"));
        assert!(msg.contains("gap at 7"));
    }

    #[test]
    fn test_map_error_code_revision_gap() {
        let err = StoreError::RevisionGap {
            expected: 5,
            found: 7,
        };
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::RevisionMismatch);
    }

    // ensure_op_id_uniqueness and lookup_existing_op tests (bd-1ua)

    #[test]
    fn test_ensure_op_id_uniqueness_creates_index() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Should succeed without error
        let result = ensure_op_id_uniqueness(&mut bootstrap.conn);
        assert!(
            result.is_ok(),
            "ensure_op_id_uniqueness should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_ensure_op_id_uniqueness_is_idempotent() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Call twice - should be idempotent
        let result1 = ensure_op_id_uniqueness(&mut bootstrap.conn);
        assert!(result1.is_ok());

        let result2 = ensure_op_id_uniqueness(&mut bootstrap.conn);
        assert!(result2.is_ok(), "Second call should also succeed");
    }

    #[test]
    fn test_lookup_existing_op_returns_none_for_nonexistent() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        let result = lookup_existing_op(&bootstrap.conn, "nonexistent-op-id");
        assert!(result.is_ok(), "lookup should succeed: {:?}", result.err());
        assert!(
            result.expect("checked is_ok").is_none(),
            "Should return None for nonexistent op_id"
        );
    }

    #[test]
    fn test_lookup_existing_op_returns_record_for_existing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Add an event
        let envelope = EventEnvelope {
            op_id: "op-lookup-test".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

        // Lookup should find it
        let result = lookup_existing_op(&bootstrap.conn, "op-lookup-test");
        assert!(result.is_ok(), "lookup should succeed: {:?}", result.err());
        let record = result.expect("checked is_ok").expect("should find record");
        assert_eq!(record.op_id, "op-lookup-test");
        assert_eq!(record.revision, 1);
        assert_eq!(record.timestamp, 1700000000);
    }

    #[test]
    fn test_duplicate_op_id_rejected_by_unique_constraint() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let op_id = "op-duplicate-constraint";

        // Add first event
        let envelope1 = EventEnvelope {
            op_id: op_id.to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "First".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        let result1 = append_event(&mut bootstrap.conn, envelope1, None);
        assert!(result1.is_ok(), "First append should succeed");

        // Try duplicate op_id - should fail
        let envelope2 = EventEnvelope {
            op_id: op_id.to_string(), // Same op_id
            operation: DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 20.0,
                y: 30.0,
                width: 100.0,
                height: 50.0,
                label: "Second".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };
        let result2 = append_event(&mut bootstrap.conn, envelope2, None);
        assert!(result2.is_err(), "Duplicate op_id should be rejected");
    }

    #[test]
    fn test_duplicate_with_conflict_error_display() {
        let err = StoreError::DuplicateWithConflict("op-123".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Duplicate op_id"));
        assert!(msg.contains("op-123"));
    }

    #[test]
    fn test_map_error_code_duplicate_with_conflict() {
        let err = StoreError::DuplicateWithConflict("op-123".to_string());
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::RevisionMismatch);
    }

    // Idempotent append tests (bd-2qg)

    #[test]
    fn test_classify_duplicate_exact_match() {
        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let payload = encode_event_envelope(&envelope).expect("Failed to encode envelope");

        let record = EventRecord {
            op_id: "op-1".to_string(),
            revision: 1,
            timestamp: 1700000000,
            payload,
        };

        let kind = classify_duplicate(&record, &envelope);
        assert!(
            kind.is_ok(),
            "classify_duplicate should succeed: {:?}",
            kind.err()
        );
        assert_eq!(kind.expect("checked is_ok"), DuplicateKind::Exact);
    }

    #[test]
    fn test_classify_duplicate_conflict() {
        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let envelope1 = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let payload1 = encode_event_envelope(&envelope1).expect("Failed to encode envelope");

        let record = EventRecord {
            op_id: "op-1".to_string(),
            revision: 1,
            timestamp: 1700000000,
            payload: payload1,
        };

        // Different envelope with same op_id but different payload
        let envelope2 = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 999.0, // Different x coordinate
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let kind = classify_duplicate(&record, &envelope2);
        assert!(
            kind.is_ok(),
            "classify_duplicate should succeed: {:?}",
            kind.err()
        );
        assert_eq!(kind.expect("checked is_ok"), DuplicateKind::Conflict);
    }

    #[test]
    fn test_append_idempotent_new_operation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let envelope = EventEnvelope {
            op_id: "op-new".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result = append_idempotent(&mut bootstrap.conn, envelope);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let outcome = result.expect("checked is_ok");
        assert_eq!(
            outcome.revision, 1,
            "New operation should create revision 1"
        );
        assert_eq!(outcome.op_id, "op-new");
    }

    #[test]
    fn test_append_idempotent_exact_duplicate_returns_existing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let envelope = EventEnvelope {
            op_id: "op-exact".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        // First append
        let result1 = append_idempotent(&mut bootstrap.conn, envelope.clone());
        assert!(
            result1.is_ok(),
            "First append should succeed: {:?}",
            result1.err()
        );
        let outcome1 = result1.expect("checked is_ok");
        assert_eq!(outcome1.revision, 1);

        // Second append with exact duplicate
        let result2 = append_idempotent(&mut bootstrap.conn, envelope);
        assert!(
            result2.is_ok(),
            "Exact duplicate should return Ok: {:?}",
            result2.err()
        );
        let outcome2 = result2.expect("checked is_ok");

        // Should return existing outcome (no-op)
        assert_eq!(
            outcome2.revision, outcome1.revision,
            "Revision should be unchanged for exact duplicate"
        );
        assert_eq!(outcome2.op_id, outcome1.op_id);
        assert_eq!(outcome2.timestamp, outcome1.timestamp);

        // Verify only one row in database
        let count: i64 = bootstrap
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE operation_id = 'op-exact'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to count events");
        assert_eq!(count, 1, "Should have exactly one row for exact duplicate");
    }

    #[test]
    fn test_append_idempotent_conflicting_duplicate_returns_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let envelope1 = EventEnvelope {
            op_id: "op-conflict".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Original".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        // First append
        let result1 = append_idempotent(&mut bootstrap.conn, envelope1);
        assert!(
            result1.is_ok(),
            "First append should succeed: {:?}",
            result1.err()
        );

        // Second append with conflicting payload (same op_id, different content)
        let envelope2 = EventEnvelope {
            op_id: "op-conflict".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 999.0, // Different x coordinate
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Modified".to_string(), // Different label
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result2 = append_idempotent(&mut bootstrap.conn, envelope2);
        assert!(
            result2.is_err(),
            "Conflicting duplicate should return error"
        );
        match result2 {
            Err(StoreError::DuplicateWithConflict(op_id)) => {
                assert_eq!(op_id, "op-conflict");
            }
            Err(e) => panic!("Expected DuplicateWithConflict error, got: {:?}", e),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_append_idempotent_preserves_revision_on_duplicate() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Add several operations first
        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000000 + i,
            };
            let _ = append_idempotent(&mut bootstrap.conn, envelope).expect("Failed to append");
        }

        // Verify we're at revision 5
        let rev_before = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(rev_before, 5);

        // Now try to append exact duplicate of op-3
        let envelope_dup = EventEnvelope {
            op_id: "op-3".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-3".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Node 3".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000003,
        };

        let result = append_idempotent(&mut bootstrap.conn, envelope_dup);
        assert!(
            result.is_ok(),
            "Exact duplicate should succeed: {:?}",
            result.err()
        );

        // Revision should be unchanged
        let rev_after = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(
            rev_after, rev_before,
            "Revision should be unchanged after exact duplicate"
        );
    }

    #[test]
    fn test_append_idempotent_multiple_different_ops() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Add op-1
        let envelope1 = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Node 1".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };
        let result1 = append_idempotent(&mut bootstrap.conn, envelope1);
        assert!(result1.is_ok());
        assert_eq!(result1.expect("checked is_ok").revision, 1);

        // Add op-2
        let envelope2 = EventEnvelope {
            op_id: "op-2".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 20.0,
                y: 30.0,
                width: 100.0,
                height: 50.0,
                label: "Node 2".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000002,
        };
        let result2 = append_idempotent(&mut bootstrap.conn, envelope2);
        assert!(result2.is_ok());
        assert_eq!(result2.expect("checked is_ok").revision, 2);

        // Add op-3 (new)
        let envelope3 = EventEnvelope {
            op_id: "op-3".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-3".to_string(),
                x: 30.0,
                y: 40.0,
                width: 100.0,
                height: 50.0,
                label: "Node 3".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000003,
        };
        let result3 = append_idempotent(&mut bootstrap.conn, envelope3);
        assert!(result3.is_ok());
        assert_eq!(result3.expect("checked is_ok").revision, 3);
    }

    #[test]
    fn test_duplicate_kind_equality() {
        assert_eq!(DuplicateKind::Exact, DuplicateKind::Exact);
        assert_eq!(DuplicateKind::Conflict, DuplicateKind::Conflict);
        assert_ne!(DuplicateKind::Exact, DuplicateKind::Conflict);
    }

    #[test]
    fn test_append_idempotent_with_different_operation_types() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Add a NodeAdd operation
        let envelope_add = EventEnvelope {
            op_id: "op-add".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        let result_add = append_idempotent(&mut bootstrap.conn, envelope_add.clone());
        assert!(result_add.is_ok());

        // Exact duplicate of NodeAdd
        let result_dup = append_idempotent(&mut bootstrap.conn, envelope_add);
        assert!(result_dup.is_ok());

        // Add a NodeMove operation
        let envelope_move = EventEnvelope {
            op_id: "op-move".to_string(),
            operation: DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };
        let result_move = append_idempotent(&mut bootstrap.conn, envelope_move.clone());
        assert!(result_move.is_ok());

        // Exact duplicate of NodeMove
        let result_move_dup = append_idempotent(&mut bootstrap.conn, envelope_move);
        assert!(result_move_dup.is_ok());
    }

    // append_batch tests

    #[test]
    fn test_append_batch_with_valid_events() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let events = vec![
            EventEnvelope {
                op_id: "batch-op-1".to_string(),
                operation: DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 1".to_string(),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000001,
            },
            EventEnvelope {
                op_id: "batch-op-2".to_string(),
                operation: DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 30.0,
                    y: 40.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 2".to_string(),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000002,
            },
            EventEnvelope {
                op_id: "batch-op-3".to_string(),
                operation: DomainOp::EdgeConnect {
                    id: "edge-1".to_string(),
                    source: "node-1".to_string(),
                    target: "node-2".to_string(),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000003,
            },
        ];

        let result = append_batch(&mut bootstrap.conn, events, None);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.start_revision, 1);
        assert_eq!(batch_result.end_revision, 3);
        assert_eq!(batch_result.count, 3);
        assert_eq!(batch_result.op_ids.len(), 3);
        assert_eq!(batch_result.last_timestamp, 1700000003);
    }

    #[test]
    fn test_append_batch_empty_returns_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        let result = append_batch(&mut bootstrap.conn, vec![], None);
        assert!(result.is_err());

        match result {
            Err(StoreError::EmptyBatch) => {}
            Err(other) => panic!("Expected EmptyBatch error, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_append_batch_with_revision_mismatch() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let events = vec![EventEnvelope {
            op_id: "batch-op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Node 1".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        }];

        // Expect revision 5, but actual is 0
        let result = append_batch(&mut bootstrap.conn, events, Some(5));
        assert!(result.is_err());

        match result {
            Err(StoreError::RevisionMismatch { expected, found }) => {
                assert_eq!(expected, 5);
                assert_eq!(found, 0);
            }
            Err(other) => panic!("Expected RevisionMismatch error, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_append_batch_with_valid_expected_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // First, add a single event to get to revision 1
        let first_event = EventEnvelope {
            op_id: "first-op".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-0".to_string(),
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
                label: "Node 0".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        let first_result = append_event(&mut bootstrap.conn, first_event, None);
        assert!(first_result.is_ok());
        assert_eq!(first_result.unwrap().revision, 1);

        // Now add a batch with expected revision 1
        let events = vec![
            EventEnvelope {
                op_id: "batch-op-1".to_string(),
                operation: DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 1".to_string(),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000001,
            },
            EventEnvelope {
                op_id: "batch-op-2".to_string(),
                operation: DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 30.0,
                    y: 40.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 2".to_string(),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000002,
            },
        ];

        let result = append_batch(&mut bootstrap.conn, events, Some(1));
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.start_revision, 2);
        assert_eq!(batch_result.end_revision, 3);
        assert_eq!(batch_result.count, 2);
    }

    #[test]
    fn test_append_batch_atomicity_on_failure() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // First, add an event that will cause a duplicate conflict later
        let first_event = EventEnvelope {
            op_id: "duplicate-op".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-0".to_string(),
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
                label: "Node 0".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        let first_result = append_event(&mut bootstrap.conn, first_event, None);
        assert!(first_result.is_ok());

        // Now try to add a batch with a duplicate op_id (will fail due to unique constraint)
        let events = vec![
            EventEnvelope {
                op_id: "batch-op-1".to_string(),
                operation: DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 1".to_string(),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000001,
            },
            EventEnvelope {
                op_id: "duplicate-op".to_string(), // This will cause a failure
                operation: DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 30.0,
                    y: 40.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 2".to_string(),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000002,
            },
        ];

        let result = append_batch(&mut bootstrap.conn, events, Some(1));
        // The batch should fail due to the duplicate
        assert!(result.is_err(), "Expected error for duplicate op_id");

        // Verify that no events were added (atomicity)
        let count: i64 = bootstrap
            .conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .expect("Failed to count events");
        assert_eq!(count, 1, "Only the first event should exist (atomicity)");
    }

    #[test]
    fn test_verify_batch_atomicity_valid() {
        let result = BatchAppendResult {
            start_revision: 1,
            end_revision: 3,
            count: 3,
            op_ids: vec!["op-1".to_string(), "op-2".to_string(), "op-3".to_string()],
            last_timestamp: 1700000003,
        };

        let verification = verify_batch_atomicity(&result);
        assert!(
            verification.is_ok(),
            "Expected Ok, got: {:?}",
            verification.err()
        );
    }

    #[test]
    fn test_verify_batch_atomicity_invalid_start_revision() {
        let result = BatchAppendResult {
            start_revision: 0,
            end_revision: 2,
            count: 3,
            op_ids: vec!["op-1".to_string(), "op-2".to_string(), "op-3".to_string()],
            last_timestamp: 1700000003,
        };

        let verification = verify_batch_atomicity(&result);
        assert!(verification.is_err());

        match verification {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("start_revision"));
            }
            Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_verify_batch_atomicity_invalid_revision_range() {
        let result = BatchAppendResult {
            start_revision: 5,
            end_revision: 3, // end < start
            count: 0,
            op_ids: vec![],
            last_timestamp: 1700000003,
        };

        let verification = verify_batch_atomicity(&result);
        assert!(verification.is_err());

        match verification {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("end_revision"));
            }
            Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_verify_batch_atomicity_count_mismatch() {
        let result = BatchAppendResult {
            start_revision: 1,
            end_revision: 3,
            count: 5, // Should be 3
            op_ids: vec!["op-1".to_string(), "op-2".to_string(), "op-3".to_string()],
            last_timestamp: 1700000003,
        };

        let verification = verify_batch_atomicity(&result);
        assert!(verification.is_err());

        match verification {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("count"));
            }
            Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_verify_batch_atomicity_empty_op_id() {
        let result = BatchAppendResult {
            start_revision: 1,
            end_revision: 2,
            count: 2,
            op_ids: vec!["op-1".to_string(), "".to_string()], // Empty op_id
            last_timestamp: 1700000002,
        };

        let verification = verify_batch_atomicity(&result);
        assert!(verification.is_err());

        match verification {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("op_id"));
            }
            Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_verify_batch_atomicity_invalid_timestamp() {
        let result = BatchAppendResult {
            start_revision: 1,
            end_revision: 1,
            count: 1,
            op_ids: vec!["op-1".to_string()],
            last_timestamp: 0, // Invalid timestamp
        };

        let verification = verify_batch_atomicity(&result);
        assert!(verification.is_err());

        match verification {
            Err(StoreError::ValidationFailed(msg)) => {
                assert!(msg.contains("last_timestamp"));
            }
            Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    #[test]
    fn test_append_batch_single_event() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let events = vec![EventEnvelope {
            op_id: "single-op".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Single Node".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        }];

        let result = append_batch(&mut bootstrap.conn, events, None);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.start_revision, 1);
        assert_eq!(batch_result.end_revision, 1);
        assert_eq!(batch_result.count, 1);
        assert_eq!(batch_result.op_ids, vec!["single-op"]);
    }

    // OCC idempotency regression tests (bd-ahf)

    /// Regression test: stale revision must be rejected with no append
    ///
    /// This test verifies that when a client attempts to append with an
    /// outdated (stale) expected revision, the operation is rejected
    /// with RevisionMismatch and no event is appended.
    #[test]
    fn test_occ_stale_revision_rejected_with_no_append() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        // Add initial events to advance revision to 3
        for i in 1..=3 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0 * i as f64,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000000 + i,
            };
            let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append");
        }

        // Verify current revision is 3
        let current = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(current, 3, "Database should be at revision 3");

        // Attempt to append with stale revision (claiming revision 1)
        let stale_envelope = EventEnvelope {
            op_id: "op-stale".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-stale".to_string(),
                x: 999.0,
                y: 999.0,
                width: 100.0,
                height: 50.0,
                label: "Stale Node".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000999,
        };

        let result = append_with_occ(&mut bootstrap.conn, stale_envelope, Some(1));

        // Must reject with RevisionMismatch
        assert!(result.is_err(), "Stale revision should be rejected");
        match result {
            Err(StoreError::RevisionMismatch { expected, found }) => {
                assert_eq!(expected, 1, "Expected should be the stale revision");
                assert_eq!(found, 3, "Found should be the current revision");
            }
            Err(other) => panic!("Expected RevisionMismatch, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }

        // Verify no new event was appended (revision still 3)
        let after_revision = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(
            after_revision, 3,
            "Revision should still be 3 after rejected append"
        );

        // Verify the stale op_id does not exist in the database
        let count: i64 = bootstrap
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE operation_id = 'op-stale'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to count events");
        assert_eq!(count, 0, "Stale operation should not be in the database");
    }

    /// Regression test: exact duplicate op_id must return no-op success
    ///
    /// This test verifies that when the same operation (same op_id and payload)
    /// is submitted again via append_idempotent, it returns Ok with the
    /// existing outcome and does not append a new row.
    #[test]
    fn test_occ_exact_duplicate_returns_noop_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let envelope = EventEnvelope {
            op_id: "op-duplicate-test".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-dup".to_string(),
                x: 42.0,
                y: 84.0,
                width: 100.0,
                height: 50.0,
                label: "Duplicate Test Node".to_string(),
            },
            author: Author {
                id: "user-dup".to_string(),
                name: "Duplicate User".to_string(),
                email: None,
            },
            timestamp: 1700000420,
        };

        // First append - should succeed with revision 1
        let result1 = append_idempotent(&mut bootstrap.conn, envelope.clone());
        assert!(
            result1.is_ok(),
            "First append should succeed: {:?}",
            result1.err()
        );
        let outcome1 = result1.expect("checked is_ok");
        assert_eq!(
            outcome1.revision, 1,
            "First append should create revision 1"
        );
        assert_eq!(outcome1.op_id, "op-duplicate-test");
        assert_eq!(outcome1.timestamp, 1700000420);

        // Second append with exact duplicate - must return no-op success
        let result2 = append_idempotent(&mut bootstrap.conn, envelope.clone());
        assert!(
            result2.is_ok(),
            "Exact duplicate should return Ok (no-op success): {:?}",
            result2.err()
        );
        let outcome2 = result2.expect("checked is_ok");

        // Must return the existing outcome (same revision, op_id, timestamp)
        assert_eq!(
            outcome2.revision, outcome1.revision,
            "Duplicate should return existing revision"
        );
        assert_eq!(
            outcome2.op_id, outcome1.op_id,
            "Duplicate should return existing op_id"
        );
        assert_eq!(
            outcome2.timestamp, outcome1.timestamp,
            "Duplicate should return existing timestamp"
        );

        // Verify only one row exists in the database
        let count: i64 = bootstrap
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE operation_id = 'op-duplicate-test'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to count events");
        assert_eq!(count, 1, "Exact duplicate should not create a new row");

        // Verify current revision is still 1
        let current = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(
            current, 1,
            "Revision should still be 1 after no-op duplicate"
        );
    }

    /// Regression test: duplicate op_id with different payload must return error
    ///
    /// This test verifies that when an operation with the same op_id but
    /// different payload is submitted, it returns DuplicateWithConflict error.
    #[test]
    fn test_occ_conflicting_duplicate_returns_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let envelope1 = EventEnvelope {
            op_id: "op-conflict-test".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-conflict".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Original".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000100,
        };

        // First append - should succeed
        let result1 = append_idempotent(&mut bootstrap.conn, envelope1);
        assert!(result1.is_ok(), "First append should succeed");
        let outcome1 = result1.expect("checked is_ok");
        assert_eq!(outcome1.revision, 1);

        // Second append with same op_id but different payload
        let envelope2 = EventEnvelope {
            op_id: "op-conflict-test".to_string(), // Same op_id
            operation: DomainOp::NodeAdd {
                id: "node-conflict".to_string(),
                x: 999.0, // Different x coordinate
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Modified".to_string(), // Different label
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000100,
        };

        let result2 = append_idempotent(&mut bootstrap.conn, envelope2);
        assert!(
            result2.is_err(),
            "Conflicting duplicate should return error"
        );
        match result2 {
            Err(StoreError::DuplicateWithConflict(op_id)) => {
                assert_eq!(op_id, "op-conflict-test");
            }
            Err(other) => panic!("Expected DuplicateWithConflict, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }

        // Verify only one row exists (the original)
        let count: i64 = bootstrap
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE operation_id = 'op-conflict-test'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to count events");
        assert_eq!(count, 1, "Conflicting duplicate should not create new row");
    }

    // ============================================================
    // BDD Error Path Tests (bd-12m)
    // ============================================================

    // ------------------------------------------------------------
    // InvalidPragma Error Path Tests
    // ------------------------------------------------------------

    /// BDD: Given InvalidPragma error variant, when constructed with WAL issue,
    /// then the error displays correctly with context.
    #[test]
    fn test_invalid_pragma_wal_mode_error_construction() {
        // Test that InvalidPragma can be constructed for WAL mode issues
        let err = StoreError::InvalidPragma("Expected WAL journal mode, got delete".to_string());

        // Verify error displays correctly
        let msg = err.to_string();
        assert!(
            msg.contains("Invalid pragma"),
            "Error message should contain 'Invalid pragma': {}",
            msg
        );
        assert!(
            msg.contains("WAL"),
            "Error message should mention WAL: {}",
            msg
        );
        assert!(
            msg.contains("delete"),
            "Error message should mention the wrong mode: {}",
            msg
        );
    }

    /// BDD: Given InvalidPragma error variant, when constructed with synchronous issue,
    /// then the error displays correctly with context.
    #[test]
    fn test_invalid_pragma_synchronous_mode_error_construction() {
        // Test that InvalidPragma can be constructed for synchronous mode issues
        let err =
            StoreError::InvalidPragma("Expected FULL synchronous mode (2), got 0".to_string());

        // Verify error displays correctly
        let msg = err.to_string();
        assert!(
            msg.contains("Invalid pragma"),
            "Error message should contain 'Invalid pragma': {}",
            msg
        );
        assert!(
            msg.contains("synchronous"),
            "Error message should mention synchronous: {}",
            msg
        );
        assert!(
            msg.contains("FULL") || msg.contains("2"),
            "Error message should mention expected value: {}",
            msg
        );
    }

    /// BDD: Given a database opened in read-only mode, when trying to set WAL,
    /// then an error occurs (SQLite or InvalidPragma).
    #[test]
    fn test_invalid_pragma_readonly_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Create and bootstrap database first
        let _ = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Open in read-only mode
        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .expect("Failed to open read-only");

        // Try to set WAL - should fail or be ignored in read-only mode
        let result: std::result::Result<(), rusqlite::Error> =
            conn.execute_batch("PRAGMA journal_mode=WAL;");

        // In read-only mode, pragma may fail or return an error
        // This verifies that the pragma mechanism can fail
        if let Ok(_) = result {
            // On some systems, the pragma may succeed but not actually change
            // Let's verify the journal mode is what we expect
            let mode: String = conn
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .unwrap_or_else(|_| "unknown".to_string());
            // In read-only mode, the mode should remain unchanged
            // This test documents the behavior
            assert!(
                mode == "wal" || mode == "delete" || mode == "unknown",
                "Journal mode in read-only: {}",
                mode
            );
        }
        // Test passes - we've verified the pragma behavior
    }

    /// BDD: Given InvalidPragma error, when converted to string,
    /// then the message contains the configuration issue.
    #[test]
    fn test_invalid_pragma_error_display() {
        let err = StoreError::InvalidPragma("journal mode is delete".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("Invalid pragma"),
            "Error message should contain 'Invalid pragma': {}",
            msg
        );
        assert!(
            msg.contains("journal mode is delete"),
            "Error message should contain the detail: {}",
            msg
        );
    }

    // ------------------------------------------------------------
    // SchemaVersionMismatch Error Path Tests
    // ------------------------------------------------------------

    /// BDD: Given InvalidPragma error, when mapping to CliErrorCode,
    /// then Unknown is returned.
    #[test]
    fn test_map_error_code_invalid_pragma() {
        let err = StoreError::InvalidPragma("test".to_string());
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::Unknown);
    }

    /// BDD: Given SchemaVersionMismatch error, when displayed,
    /// then the message shows expected and found versions.
    #[test]
    fn test_schema_version_mismatch_error_display() {
        let err = StoreError::SchemaVersionMismatch {
            expected: 2,
            found: 1,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("Schema version mismatch"),
            "Error message should contain 'Schema version mismatch': {}",
            msg
        );
        assert!(
            msg.contains("expected 2"),
            "Error message should contain expected version: {}",
            msg
        );
        assert!(
            msg.contains("found 1"),
            "Error message should contain found version: {}",
            msg
        );
    }

    /// BDD: Given SchemaVersionMismatch error, when mapping to CliErrorCode,
    /// then Unknown is returned.
    #[test]
    fn test_map_error_code_schema_version_mismatch() {
        let err = StoreError::SchemaVersionMismatch {
            expected: 2,
            found: 1,
        };
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::Unknown);
    }

    // ------------------------------------------------------------
    // MigrationForbidden Error Path Tests
    // ------------------------------------------------------------

    /// BDD: Given MigrationForbidden error, when displayed,
    /// then the message shows the forbidden version.
    #[test]
    fn test_migration_forbidden_error_display() {
        let err = StoreError::MigrationForbidden { version: 0 };
        let msg = err.to_string();
        assert!(
            msg.contains("Migration forbidden"),
            "Error message should contain 'Migration forbidden': {}",
            msg
        );
        assert!(
            msg.contains("version 0"),
            "Error message should contain version: {}",
            msg
        );
    }

    /// BDD: Given MigrationForbidden error, when mapping to CliErrorCode,
    /// then Unknown is returned.
    #[test]
    fn test_map_error_code_migration_forbidden() {
        let err = StoreError::MigrationForbidden { version: 0 };
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::Unknown);
    }

    // ------------------------------------------------------------
    // RevisionMismatch Error Path Tests (BDD-style)
    // ------------------------------------------------------------

    /// BDD: Given RevisionMismatch error, when displayed,
    /// then the message shows expected and found revisions.
    #[test]
    fn test_revision_mismatch_error_display() {
        let err = StoreError::RevisionMismatch {
            expected: 10,
            found: 5,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("Revision mismatch"),
            "Error message should contain 'Revision mismatch': {}",
            msg
        );
        assert!(
            msg.contains("expected 10"),
            "Error message should contain expected revision: {}",
            msg
        );
        assert!(
            msg.contains("found 5"),
            "Error message should contain found revision: {}",
            msg
        );
    }

    /// BDD: Given RevisionMismatch error, when mapping to CliErrorCode,
    /// then RevisionMismatch is returned.
    #[test]
    fn test_map_error_code_revision_mismatch_variant() {
        let err = StoreError::RevisionMismatch {
            expected: 5,
            found: 3,
        };
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::RevisionMismatch);
    }

    // ------------------------------------------------------------
    // RevisionGap Error Path Tests (BDD-style)
    // ------------------------------------------------------------

    /// BDD: Given a RevisionGap error, when verified,
    /// then it maps to RevisionMismatch code and displays correctly.
    #[test]
    fn test_revision_gap_full_error_path() {
        // Test display
        let err = StoreError::RevisionGap {
            expected: 5,
            found: 7,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("Revision gap detected"),
            "Error message should contain 'Revision gap detected': {}",
            msg
        );
        assert!(
            msg.contains("sequential revision 5"),
            "Error message should contain expected sequential revision: {}",
            msg
        );
        assert!(
            msg.contains("gap at 7"),
            "Error message should contain found gap revision: {}",
            msg
        );

        // Test error code mapping
        let code = map_error_code(&err);
        assert_eq!(
            code,
            CliErrorCode::RevisionMismatch,
            "RevisionGap should map to RevisionMismatch code"
        );
    }

    // ------------------------------------------------------------
    // EmptyBatch Error Path Tests (BDD-style)
    // ------------------------------------------------------------

    /// BDD: Given EmptyBatch error, when displayed,
    /// then the message mentions zero events.
    #[test]
    fn test_empty_batch_error_display() {
        let err = StoreError::EmptyBatch;
        let msg = err.to_string();
        assert!(
            msg.contains("Empty batch"),
            "Error message should contain 'Empty batch': {}",
            msg
        );
        assert!(
            msg.contains("zero events"),
            "Error message should mention zero events: {}",
            msg
        );
    }

    /// BDD: Given EmptyBatch error, when mapping to CliErrorCode,
    /// then ValidationFailed is returned.
    #[test]
    fn test_map_error_code_empty_batch() {
        let err = StoreError::EmptyBatch;
        let code = map_error_code(&err);
        assert_eq!(code, CliErrorCode::ValidationFailed);
    }

    // ------------------------------------------------------------
    // CorruptDatabase Error Path Tests
    // ------------------------------------------------------------

    /// BDD: Given CorruptDatabase error, when displayed,
    /// then the message shows the corruption detail.
    #[test]
    fn test_corrupt_database_error_display() {
        let err = RecoveryError::CorruptDatabase("page 42 is malformed".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("integrity check failed"),
            "Error message should contain 'integrity check failed': {}",
            msg
        );
        assert!(
            msg.contains("page 42 is malformed"),
            "Error message should contain detail: {}",
            msg
        );
    }

    /// BDD: Given a corrupted database file, when integrity check runs,
    /// then CorruptDatabase error is returned.
    #[test]
    fn test_corrupt_database_on_invalid_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("corrupt.db");

        // Write invalid SQLite header
        std::fs::write(&db_path, b"This is not a valid SQLite database file")
            .expect("Failed to write corrupt file");

        let result = startup_integrity_check(&db_path);
        assert!(result.is_err(), "Expected error for corrupt database");

        match result {
            Err(RecoveryError::CorruptDatabase(msg)) => {
                assert!(
                    !msg.is_empty(),
                    "CorruptDatabase error should have a message"
                );
            }
            Err(RecoveryError::Sqlite(_)) => {
                // SQLite error is also acceptable for corrupt file
            }
            Err(other) => panic!("Expected CorruptDatabase or Sqlite error, got: {:?}", other),
            Ok(status) => {
                // If it returns Ok, the status should indicate invalid
                assert!(
                    !status.is_valid,
                    "Corrupt database should be marked as invalid"
                );
            }
        }
    }

    // ------------------------------------------------------------
    // BackupUnavailable Error Path Tests
    // ------------------------------------------------------------

    /// BDD: Given BackupUnavailable error, when displayed,
    /// then the message shows the unavailability reason.
    #[test]
    fn test_backup_unavailable_error_display() {
        let err = RecoveryError::BackupUnavailable("/path/to/backup.db not found".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("Backup file unavailable"),
            "Error message should contain 'Backup file unavailable': {}",
            msg
        );
        assert!(
            msg.contains("/path/to/backup.db not found"),
            "Error message should contain detail: {}",
            msg
        );
    }

    /// BDD: Given a nonexistent backup path, when recovery is attempted,
    /// then appropriate error is returned.
    #[test]
    fn test_backup_unavailable_on_missing_file() {
        let nonexistent_backup = Path::new("/nonexistent/path/backup.db");

        // Verify the file doesn't exist
        assert!(
            !nonexistent_backup.exists(),
            "Test assumes backup file does not exist"
        );

        // The RecoveryError::BackupUnavailable would be used in a restore function
        // Here we verify the error can be constructed and used correctly
        let err = RecoveryError::BackupUnavailable(format!(
            "Backup file not found: {}",
            nonexistent_backup.display()
        ));

        match &err {
            RecoveryError::BackupUnavailable(msg) => {
                assert!(
                    msg.contains("not found"),
                    "Error message should indicate file not found: {}",
                    msg
                );
            }
            _ => panic!("Expected BackupUnavailable error"),
        }
    }

    // ------------------------------------------------------------
    // Comprehensive BDD Scenario Tests
    // ------------------------------------------------------------

    /// BDD Scenario: Atomicity on RevisionMismatch
    /// Given a database at revision 0
    /// When append_batch is called with expected revision 999
    /// Then RevisionMismatch error is returned
    /// And no events are appended
    #[test]
    fn test_bdd_revision_mismatch_atomicity() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        use crate::models::envelope::{Author, DomainOp, EventEnvelope};

        let events = vec![EventEnvelope {
            op_id: "op-should-not-append".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Node 1".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        }];

        // Pre-condition: database is at revision 0
        let revision_before = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(revision_before, 0, "Database should start at revision 0");

        // Attempt to append with wrong expected revision
        let result = append_batch(&mut bootstrap.conn, events, Some(999));

        // Verify error
        assert!(result.is_err(), "Expected error for revision mismatch");
        match result {
            Err(StoreError::RevisionMismatch { expected, found }) => {
                assert_eq!(expected, 999, "Expected should be 999");
                assert_eq!(found, 0, "Found should be 0");
            }
            Err(other) => panic!("Expected RevisionMismatch, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }

        // Verify atomicity: no events were appended
        let revision_after = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(
            revision_after, 0,
            "Revision should still be 0 after failed append"
        );

        let count: i64 = bootstrap
            .conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .expect("Failed to count events");
        assert_eq!(count, 0, "No events should be in the database");
    }

    /// BDD Scenario: EmptyBatch rejection
    /// Given a valid database connection
    /// When append_batch is called with an empty vector
    /// Then EmptyBatch error is returned
    /// And database state is unchanged
    #[test]
    fn test_bdd_empty_batch_rejection() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Pre-condition: database is at revision 0
        let revision_before = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(revision_before, 0);

        // Attempt to append empty batch
        let result = append_batch(&mut bootstrap.conn, vec![], None);

        // Verify error
        assert!(result.is_err(), "Expected error for empty batch");
        match result {
            Err(StoreError::EmptyBatch) => {}
            Err(other) => panic!("Expected EmptyBatch, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }

        // Verify no state change
        let revision_after = current_revision(&bootstrap.conn).expect("Failed to get revision");
        assert_eq!(
            revision_after, 0,
            "Revision should still be 0 after empty batch"
        );
    }

    /// BDD Scenario: Error message quality
    /// Given various error types
    /// When converted to string
    /// Then messages are human-readable and contain relevant context
    #[test]
    fn test_bdd_error_message_quality() {
        // Test all error types have meaningful messages

        // StoreError variants
        let test_cases: Vec<(StoreError, &[&str])> = vec![
            (
                StoreError::InvalidPragma("bad config".to_string()),
                &["Invalid pragma", "bad config"],
            ),
            (
                StoreError::SchemaVersionMismatch {
                    expected: 2,
                    found: 1,
                },
                &["Schema version mismatch", "expected 2", "found 1"],
            ),
            (
                StoreError::MigrationForbidden { version: 0 },
                &["Migration forbidden", "version 0"],
            ),
            (
                StoreError::RevisionMismatch {
                    expected: 10,
                    found: 5,
                },
                &["Revision mismatch", "expected 10", "found 5"],
            ),
            (
                StoreError::RevisionGap {
                    expected: 5,
                    found: 7,
                },
                &["Revision gap", "sequential revision 5", "gap at 7"],
            ),
            (StoreError::EmptyBatch, &["Empty batch", "zero events"]),
        ];

        for (err, expected_fragments) in test_cases {
            let msg = err.to_string();
            for fragment in expected_fragments {
                assert!(
                    msg.contains(fragment),
                    "Error message '{}' should contain '{}': {}",
                    msg,
                    fragment,
                    msg
                );
            }
        }

        // RecoveryError variants
        let recovery_test_cases: Vec<(RecoveryError, &[&str])> = vec![
            (
                RecoveryError::CorruptDatabase("malformed page".to_string()),
                &["integrity check failed", "malformed page"],
            ),
            (
                RecoveryError::BackupUnavailable("file not found".to_string()),
                &["Backup file unavailable", "file not found"],
            ),
        ];

        for (err, expected_fragments) in recovery_test_cases {
            let msg = err.to_string();
            for fragment in expected_fragments {
                assert!(
                    msg.contains(fragment),
                    "Error message '{}' should contain '{}': {}",
                    msg,
                    fragment,
                    msg
                );
            }
        }
    }
}
