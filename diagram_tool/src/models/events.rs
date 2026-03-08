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
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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
pub async fn ensure_schema_v1(pool: &SqlitePool) -> Result<SchemaState, StoreError> {
    let existing_state = read_schema_state(pool).await.ok();

    if let Some(state) = existing_state {
        if state.version == SCHEMA_VERSION {
            return Ok(state);
        }
        if state.version > SCHEMA_VERSION {
            return Err(StoreError::SchemaVersionMismatch {
                expected: SCHEMA_VERSION,
                found: state.version,
            });
        }
        return Err(StoreError::MigrationForbidden {
            version: state.version,
        });
    }

    create_schema_v1(pool).await
}

/// Read the current schema state from the database
///
/// # Errors
/// Returns an error if the schema table cannot be read
pub async fn read_schema_state(pool: &SqlitePool) -> Result<SchemaState, StoreError> {
    let query = format!("SELECT version, created_at FROM {SCHEMA_TABLE} LIMIT 1");

    let row = sqlx::query_as::<_, (i32, i64)>(&query)
        .fetch_optional(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    match row {
        Some((version, created_at)) => Ok(SchemaState {
            version,
            created_at,
        }),
        None => Err(StoreError::Sqlx(sqlx::Error::RowNotFound)),
    }
}

/// Create the v1 schema tables
async fn create_schema_v1(pool: &SqlitePool) -> Result<SchemaState, StoreError> {
    let mut tx = pool.begin().await.map_err(StoreError::Sqlx)?;

    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {SCHEMA_TABLE} (
            version INTEGER NOT NULL PRIMARY KEY,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )"
    ))
    .execute(&mut *tx)
    .await
    .map_err(StoreError::Sqlx)?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id TEXT NOT NULL PRIMARY KEY,
            revision INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            metadata TEXT NOT NULL DEFAULT '{}',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
    )
    .execute(&mut *tx)
    .await
    .map_err(StoreError::Sqlx)?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)")
        .execute(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)")
        .execute(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER NOT NULL PRIMARY KEY,
            revision INTEGER NOT NULL UNIQUE,
            payload TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
    )
    .execute(&mut *tx)
    .await
    .map_err(StoreError::Sqlx)?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC)")
        .execute(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    sqlx::query(&format!("INSERT INTO {SCHEMA_TABLE} (version) VALUES (?)"))
        .bind(SCHEMA_VERSION)
        .execute(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    tx.commit().await.map_err(StoreError::Sqlx)?;

    read_schema_state(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::create_pool;
    use tempfile::TempDir;

    #[tokio::test]
    async fn given_fresh_database_when_ensuring_schema_then_schema_is_created() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.expect("Failed to create pool");

        let result = ensure_schema_v1(&pool).await;

        assert!(result.is_ok(), "Schema creation failed: {:?}", result.err());
        let state = result.expect("Schema state");
        assert_eq!(state.version, SCHEMA_VERSION);

        pool.close().await;
    }

    #[tokio::test]
    async fn given_database_with_v1_schema_when_reading_state_then_returns_v1() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.expect("Failed to create pool");

        ensure_schema_v1(&pool)
            .await
            .expect("Schema creation failed");

        let state = read_schema_state(&pool).await.expect("Read state failed");

        assert_eq!(state.version, 1);

        pool.close().await;
    }

    #[tokio::test]
    async fn given_database_with_v1_schema_when_ensuring_again_then_returns_existing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.expect("Failed to create pool");

        let first = ensure_schema_v1(&pool).await.expect("First ensure failed");
        let second = ensure_schema_v1(&pool).await.expect("Second ensure failed");

        assert_eq!(first.version, second.version);

        pool.close().await;
    }

    #[tokio::test]
    async fn given_unknown_higher_schema_version_then_rejects_with_mismatch() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.expect("Failed to create pool");

        sqlx::query(&format!(
            "CREATE TABLE {} (version INTEGER NOT NULL, created_at INTEGER)",
            SCHEMA_TABLE
        ))
        .execute(&pool)
        .await
        .expect("Create table failed");

        sqlx::query(&format!(
            "INSERT INTO {} (version, created_at) VALUES (99, 0)",
            SCHEMA_TABLE
        ))
        .execute(&pool)
        .await
        .expect("Insert failed");

        let result = ensure_schema_v1(&pool).await;

        assert!(result.is_err());
        match result {
            Err(StoreError::SchemaVersionMismatch { expected, found }) => {
                assert_eq!(expected, 1);
                assert_eq!(found, 99);
            }
            _ => panic!("Expected SchemaVersionMismatch error"),
        }

        pool.close().await;
    }

    #[tokio::test]
    async fn given_lower_schema_version_then_rejects_with_migration_forbidden() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.expect("Failed to create pool");

        sqlx::query(&format!(
            "CREATE TABLE {} (version INTEGER NOT NULL, created_at INTEGER)",
            SCHEMA_TABLE
        ))
        .execute(&pool)
        .await
        .expect("Create table failed");

        sqlx::query(&format!(
            "INSERT INTO {} (version, created_at) VALUES (0, 0)",
            SCHEMA_TABLE
        ))
        .execute(&pool)
        .await
        .expect("Insert failed");

        let result = ensure_schema_v1(&pool).await;

        assert!(result.is_err());
        match result {
            Err(StoreError::MigrationForbidden { version }) => {
                assert_eq!(version, 0);
            }
            _ => panic!("Expected MigrationForbidden error"),
        }

        pool.close().await;
    }
}
