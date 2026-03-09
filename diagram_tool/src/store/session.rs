use rusqlite::Connection;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::config::DatabaseConfig;
use crate::store::error::StoreError;

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone)]
pub struct StorePragmas {
    pub journal_mode: String,
    pub synchronous: i32,
    pub wal_autocheckpoint: i32,
}

#[derive(Debug)]
pub struct StoreBootstrap {
    pub conn: Connection,
    pub db_path: PathBuf,
    pub schema_version: i32,
}

#[derive(Debug)]
pub struct StoreConfig {
    pub pragmas: StorePragmas,
    pub schema_version: i32,
}

#[derive(Debug, Clone, Serialize)]
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

pub struct StoreConnection {
    pub conn: Connection,
}

pub struct ReadWriteSession {
    pub conn: Connection,
    pub db_path: PathBuf,
}

pub struct ReadOnlySession {
    pub conn: Connection,
    pub db_path: PathBuf,
    pub export_path: Option<PathBuf>,
}

pub type RecoveryHandle = ReadOnlySession;
pub type RecoverySession = ReadOnlySession;

impl ReadWriteSession {
    pub fn open(db_path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;
             PRAGMA wal_autocheckpoint=1000;",
        )?;

        let journal_mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        if journal_mode != "wal" {
            return Err(StoreError::InvalidPragma(format!(
                "Expected WAL journal mode, got {}",
                journal_mode
            )));
        }

        let synchronous: i32 = conn.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
        if synchronous != 2 {
            return Err(StoreError::InvalidPragma(format!(
                "Expected FULL synchronous mode (2), got {}",
                synchronous
            )));
        }

        let schema_version: i32 =
            match conn.query_row("SELECT version FROM schema_version", [], |row| row.get(0)) {
                Ok(v) => v,
                Err(_) => 0,
            };
        if schema_version != CURRENT_SCHEMA_VERSION && schema_version != 0 {
            return Err(StoreError::SchemaVersionMismatch {
                expected: CURRENT_SCHEMA_VERSION,
                found: schema_version,
            });
        }

        Ok(Self {
            conn,
            db_path: db_path.to_path_buf(),
        })
    }
}

pub fn open_store(db_path: &Path) -> Result<StoreConnection, StoreError> {
    let session = ReadWriteSession::open(db_path)?;
    Ok(StoreConnection { conn: session.conn })
}

pub fn bootstrap_store_with_config(
    db_path: &Path,
    config: &DatabaseConfig,
) -> Result<StoreBootstrap, StoreError> {
    let conn = Connection::open(db_path)?;

    let pragma_sql = format!(
        "PRAGMA journal_mode={};
         PRAGMA synchronous={};
         PRAGMA wal_autocheckpoint={};",
        config.journal_mode, config.synchronous, config.wal_autocheckpoint
    );
    conn.execute_batch(&pragma_sql)?;

    run_schema_migration(&conn)?;

    let schema_version =
        match conn.query_row("SELECT version FROM schema_version", [], |row| row.get(0)) {
            Ok(v) => v,
            Err(_) => 0,
        };

    Ok(StoreBootstrap {
        conn,
        db_path: db_path.to_path_buf(),
        schema_version,
    })
}

pub fn bootstrap_store(db_path: &Path) -> Result<StoreBootstrap, StoreError> {
    let default_config = DatabaseConfig::default();
    bootstrap_store_with_config(db_path, &default_config)
}

fn run_schema_migration(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL DEFAULT 1
        );
        INSERT OR IGNORE INTO schema_version (version) VALUES (1);
        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            operation_id TEXT NOT NULL UNIQUE,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision);
        CREATE INDEX IF NOT EXISTS idx_events_operation_id ON events(operation_id);
        CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER NOT NULL PRIMARY KEY,
            revision INTEGER NOT NULL UNIQUE,
            payload TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );
        CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC);",
    )?;
    Ok(())
}

pub fn current_store_config(conn: &Connection) -> Result<StoreConfig, StoreError> {
    let journal_mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    let synchronous: i32 = conn.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
    let wal_autocheckpoint: i32 =
        conn.query_row("PRAGMA wal_autocheckpoint", [], |row| row.get(0))?;
    let schema_version =
        match conn.query_row("SELECT version FROM schema_version", [], |row| row.get(0)) {
            Ok(v) => v,
            Err(_) => 0,
        };

    Ok(StoreConfig {
        pragmas: StorePragmas {
            journal_mode,
            synchronous,
            wal_autocheckpoint,
        },
        schema_version,
    })
}
