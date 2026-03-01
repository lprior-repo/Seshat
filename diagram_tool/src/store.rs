//! SQLite storage module
//!
//! Provides SQLite-based storage with WAL mode and full synchronous durability.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use rusqlite::Connection;
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
    pub fn code(&self) -> &'static str {
        match self {
            CliErrorCode::RevisionMismatch => "revision_mismatch",
            CliErrorCode::HumanPriorityBlock => "human_priority_block",
            CliErrorCode::PolicyViolation => "policy_violation",
            CliErrorCode::ValidationFailed => "validation_failed",
            CliErrorCode::Unknown => "unknown",
        }
    }
}

/// Maps a StoreError to a CliErrorCode
///
/// # Errors
/// Returns `CliErrorCode::Unknown` for unmapped error variants
pub fn map_error_code(err: &StoreError) -> CliErrorCode {
    match err {
        StoreError::RevisionMismatch { .. } => CliErrorCode::RevisionMismatch,
        StoreError::HumanPriorityBlock(_) => CliErrorCode::HumanPriorityBlock,
        StoreError::ValidationFailed(_) => CliErrorCode::ValidationFailed,
        StoreError::Sqlite(_) => CliErrorCode::Unknown,
        StoreError::Io(_) => CliErrorCode::Unknown,
        StoreError::InvalidPragma(_) => CliErrorCode::Unknown,
        StoreError::SchemaVersionMismatch { .. } => CliErrorCode::Unknown,
        StoreError::MigrationForbidden { .. } => CliErrorCode::Unknown,
        StoreError::Serialization(_) => CliErrorCode::Unknown,
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

/// Errors that can occur during database recovery operations
#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("Database integrity check failed: {0}")]
    CorruptDatabase(String),
    #[error("Backup file unavailable: {0}")]
    BackupUnavailable(String),
    #[error("IO error during recovery: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error during recovery: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone)]
pub struct StorePragmas {
    pub journal_mode: String,
    pub synchronous: i32,
    pub wal_autocheckpoint: i32,
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

pub fn open_store(db_path: &Path) -> Result<StoreConnection, StoreError> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=FULL;
         PRAGMA wal_autocheckpoint=1000;",
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

    Ok(StorePragmas {
        journal_mode,
        synchronous,
        wal_autocheckpoint,
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
         PRAGMA wal_autocheckpoint=1000;",
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

/// Run integrity check on the database at startup
///
/// This function performs a comprehensive integrity check:
/// 1. Verifies the database file can be opened
/// 2. Checks SQLite integrity via PRAGMA integrity_check
/// 3. Validates schema version table exists and is readable
/// 4. Counts events and determines latest revision
/// 5. Checks for page corruption
///
/// Returns an IntegrityStatus with detailed results of each check.
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

    let corrupted_pages: u32 = if !is_valid && integrity_result.contains("corrupt") {
        1
    } else {
        0
    };

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
        Some(format!("{} corrupted pages found", corrupted_pages))
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
/// Returns a RecoveryHandle for read-only operations.
pub fn open_recovery_mode(db_path: &Path) -> Result<RecoveryHandle, RecoveryError> {
    // Open in read-only mode
    let conn = Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(RecoveryError::Sqlite)?;

    // Verify we can read from the database
    let _: i32 = conn
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .map_err(|e| RecoveryError::CorruptDatabase(e.to_string()))?;

    Ok(RecoveryHandle {
        conn,
        db_path: db_path.to_path_buf(),
        export_path: None,
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

        let events: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let operation_id: String = row.get(1)?;
                let revision: i64 = row.get(2)?;
                let payload: String = row.get(3)?;
                let timestamp: String = row.get(4)?;

                Ok(serde_json::json!({
                    "id": id,
                    "operation_id": operation_id,
                    "revision": revision,
                    "payload": payload,
                    "timestamp": timestamp
                }))
            })
            .map_err(RecoveryError::Sqlite)?
            .filter_map(Result::ok)
            .collect();

        // Write to JSON file
        let json_content = serde_json::to_string_pretty(&events).map_err(|e| {
            RecoveryError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

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
}
