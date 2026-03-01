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
use std::path::{Path, PathBuf};
use thiserror::Error;

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

pub struct StoreConnection {
    pub conn: Connection,
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
}
