//! SQLite storage module
//!
//! Provides SQLite-based storage with WAL mode and full synchronous durability.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
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
}

#[derive(Debug, Clone)]
pub struct StorePragmas {
    pub journal_mode: String,
    pub synchronous: i32,
    pub wal_autocheckpoint: i32,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_open_store_creates_wal_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let store = open_store(&db_path).unwrap();
        let pragmas = read_store_pragmas(&store.conn).unwrap();

        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 2);
    }
}
