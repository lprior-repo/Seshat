use super::config::{
    JournalMode, StoreBootstrap, StoreConfig, StorePragmas, SynchronousMode, WalAutoCheckpoint,
};
use super::errors::StoreError;
use super::types::StoreConnection;
use crate::config::DatabaseConfig;
use rusqlite::Connection;
use std::path::Path;

pub fn open_store(db_path: &Path) -> Result<StoreConnection, StoreError> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=FULL;
         PRAGMA wal_autocheckpoint=1000;",
    )?;

    let pragmas = read_store_pragmas(&conn)?;
    if pragmas.journal_mode != JournalMode::Wal {
        return Err(StoreError::InvalidPragma(format!(
            "Expected WAL journal mode, got {}",
            pragmas.journal_mode
        )));
    }
    if pragmas.synchronous != SynchronousMode::Full {
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
        journal_mode: JournalMode::from(journal_mode),
        synchronous: SynchronousMode::try_from(synchronous).map_err(StoreError::InvalidPragma)?,
        wal_autocheckpoint: WalAutoCheckpoint(wal_autocheckpoint.try_into().unwrap_or(0)),
    })
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

    let pragmas = read_store_pragmas(&conn)?;
    if pragmas.journal_mode.to_string().to_lowercase() != config.journal_mode.to_lowercase() {
        return Err(StoreError::InvalidPragma(format!(
            "Expected {} journal mode, got {}",
            config.journal_mode, pragmas.journal_mode
        )));
    }
    if pragmas.synchronous as i32 != config.synchronous {
        return Err(StoreError::InvalidPragma(format!(
            "Expected synchronous mode ({}), got {}",
            config.synchronous, pragmas.synchronous
        )));
    }

    run_schema_migration(&conn)?;
    let schema_version = conn
        .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
        .unwrap_or(0);

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
    let table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sqlite)?;
    if table_exists == 0 {
        conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL DEFAULT 1); INSERT OR IGNORE INTO schema_version (version) VALUES (1);")?;
    }

    let events_table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sqlite)?;
    if events_table_exists == 0 {
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
