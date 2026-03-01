//! Events schema module - v1 schema for events snapshots metadata
//!
//! This module provides `SQLite` schema management for storing event snapshots
//! and their metadata. The schema tracks versions and rejects unknown versions
//! rather than attempting migration.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::store::StoreError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// Current schema version for events schema
const SCHEMA_VERSION: i32 = 1;

/// Name of the schema state table
const SCHEMA_TABLE: &str = "events_schema_version";

/// Schema state tracking the current version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaState {
    pub version: i32,
    pub created_at: i64,
}

/// Create v1 schema for events snapshots metadata
///
/// This function creates the necessary tables for storing event snapshots
/// and their metadata. It will fail if an unknown schema version already exists.
///
/// # Errors
/// Returns `StoreError::SchemaVersionMismatch` if an incompatible schema version exists
/// Returns `StoreError::MigrationForbidden` if migration is attempted
pub fn ensure_schema_v1(conn: &Connection) -> Result<SchemaState, StoreError> {
    let existing_state = read_schema_state(conn).ok();

    if let Some(state) = existing_state {
        // Schema exists - check version compatibility
        if state.version == SCHEMA_VERSION {
            // Already at v1, nothing to do
            return Ok(state);
        }
        // Unknown version - reject instead of migrating
        if state.version > SCHEMA_VERSION {
            return Err(StoreError::SchemaVersionMismatch {
                expected: SCHEMA_VERSION,
                found: state.version,
            });
        }
        // Version < SCHEMA_VERSION - migration forbidden per contract
        return Err(StoreError::MigrationForbidden {
            version: state.version,
        });
    }

    // No schema exists - create v1 schema in a transaction
    create_schema_v1(conn)
}

/// Read the current schema state from the database
///
/// # Errors
/// Returns an error if the schema table cannot be read
pub fn read_schema_state(conn: &Connection) -> Result<SchemaState, StoreError> {
    let query = format!("SELECT version, created_at FROM {SCHEMA_TABLE} LIMIT 1");

    conn.query_row(&query, [], |row| {
        Ok(SchemaState {
            version: row.get(0)?,
            created_at: row.get(1)?,
        })
    })
    .map_err(StoreError::Sqlite)
}

/// Create the v1 schema tables
fn create_schema_v1(conn: &Connection) -> Result<SchemaState, StoreError> {
    let tx = conn.unchecked_transaction()?;

    // Create schema version tracking table
    tx.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {SCHEMA_TABLE} (
                version INTEGER NOT NULL PRIMARY KEY,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )"
        ),
        [],
    )?;

    // Create events table for storing event snapshots
    tx.execute(
        "CREATE TABLE IF NOT EXISTS events (
            id TEXT NOT NULL PRIMARY KEY,
            revision INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            metadata TEXT NOT NULL DEFAULT '{}',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    // Create index on revision for efficient history queries
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)",
        [],
    )?;

    // Create index on event_type for filtering
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)",
        [],
    )?;

    // Create snapshot table for storing serialized projections
    tx.execute(
        "CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER NOT NULL PRIMARY KEY,
            revision INTEGER NOT NULL UNIQUE,
            payload TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    // Create index on snapshot revision for efficient lookups
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC)",
        [],
    )?;

    // Insert schema version record
    tx.execute(
        &format!("INSERT INTO {SCHEMA_TABLE} (version) VALUES (?)"),
        [SCHEMA_VERSION],
    )?;

    tx.commit()?;

    // Return the created schema state
    read_schema_state(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn given_fresh_database_when_ensuring_schema_then_schema_is_created() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut conn = Connection::open(&db_path).unwrap();

        // Ensure schema v1 on fresh database
        let result = ensure_schema_v1(&mut conn);

        assert!(result.is_ok(), "Schema creation failed: {:?}", result.err());
        let state = result.unwrap();
        assert_eq!(state.version, SCHEMA_VERSION);
    }

    #[test]
    fn given_database_with_v1_schema_when_reading_state_then_returns_v1() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut conn = Connection::open(&db_path).unwrap();

        // First ensure creates schema
        ensure_schema_v1(&mut conn).unwrap();

        // Read state separately
        let state = read_schema_state(&conn).unwrap();

        assert_eq!(state.version, 1);
    }

    #[test]
    fn given_database_with_v1_schema_when_ensuring_again_then_returns_existing() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut conn = Connection::open(&db_path).unwrap();

        // First ensure creates schema
        let first = ensure_schema_v1(&mut conn).unwrap();

        // Second ensure returns existing
        let second = ensure_schema_v1(&mut conn).unwrap();

        assert_eq!(first.version, second.version);
    }

    #[test]
    fn given_unknown_higher_schema_version_then_rejects_with_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut conn = Connection::open(&db_path).unwrap();

        // Manually insert a higher version
        conn.execute(
            &format!(
                "CREATE TABLE {} (version INTEGER NOT NULL, created_at INTEGER)",
                SCHEMA_TABLE
            ),
            [],
        )
        .unwrap();
        conn.execute(
            &format!(
                "INSERT INTO {} (version, created_at) VALUES (99, 0)",
                SCHEMA_TABLE
            ),
            [],
        )
        .unwrap();

        // Now try to ensure v1 - should fail
        let result = ensure_schema_v1(&mut conn);

        assert!(result.is_err());
        match result {
            Err(StoreError::SchemaVersionMismatch { expected, found }) => {
                assert_eq!(expected, 1);
                assert_eq!(found, 99);
            }
            _ => panic!("Expected SchemaVersionMismatch error"),
        }
    }

    #[test]
    fn given_lower_schema_version_then_rejects_with_migration_forbidden() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut conn = Connection::open(&db_path).unwrap();

        // Manually insert a lower version
        conn.execute(
            &format!(
                "CREATE TABLE {} (version INTEGER NOT NULL, created_at INTEGER)",
                SCHEMA_TABLE
            ),
            [],
        )
        .unwrap();
        conn.execute(
            &format!(
                "INSERT INTO {} (version, created_at) VALUES (0, 0)",
                SCHEMA_TABLE
            ),
            [],
        )
        .unwrap();

        // Now try to ensure v1 - should fail
        let result = ensure_schema_v1(&mut conn);

        assert!(result.is_err());
        match result {
            Err(StoreError::MigrationForbidden { version }) => {
                assert_eq!(version, 0);
            }
            _ => panic!("Expected MigrationForbidden error"),
        }
    }
}
