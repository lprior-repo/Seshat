use crate::store::sqlx::error::*;
use crate::store::sqlx::models::*;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::{Path, PathBuf};

/// Creates an async `SQLite` connection pool.
///
/// # Errors
///
/// Returns a `StoreError` if the connection fails.
pub async fn create_pool(db_path: &Path) -> Result<SqlitePool, StoreError> {
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    // Configure pragmas for optimal concurrent performance
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA synchronous=NORMAL")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA wal_autocheckpoint=1000")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;

    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(&pool)
        .await?;

    Ok(pool)
}

/// Bootstraps a new async store, creating the database and running migrations
///
/// # Errors
///
/// Returns a `StoreError` if the store cannot be created or migrated.
pub async fn bootstrap_store(db_path: &Path) -> Result<StoreBootstrap, StoreError> {
    let pool = create_pool(db_path).await?;

    run_schema_migration(&pool).await?;

    let schema_version = sqlx::query_scalar::<_, i32>("SELECT version FROM schema_version")
        .fetch_one(&pool)
        .await
        .map_err(|_| StoreError::SchemaVersionMissing)?;

    Ok(StoreBootstrap {
        pool,
        db_path: db_path.to_path_buf(),
        schema_version,
    })
}

/// Runs schema migrations for the async store
async fn run_schema_migration(pool: &SqlitePool) -> Result<(), StoreError> {
    let table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
    )
    .fetch_one(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    if table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL DEFAULT 1
            )",
        )
        .execute(pool)
        .await?;

        sqlx::query("INSERT OR IGNORE INTO schema_version (version) VALUES (1)")
            .execute(pool)
            .await?;
    }

    let events_table_exists: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'")
            .fetch_one(pool)
            .await
            .map_err(StoreError::Sqlx)?;

    if events_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id TEXT NOT NULL UNIQUE,
                revision INTEGER NOT NULL UNIQUE,
                payload TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
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
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='snapshots'",
    )
    .fetch_one(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    if snapshot_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER NOT NULL PRIMARY KEY,
                revision INTEGER NOT NULL UNIQUE,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC)",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
/// Reads store configuration pragmas
///
/// # Errors
///
/// Returns a `StoreError` if the pragma query fails.
pub async fn read_store_pragmas(pool: &SqlitePool) -> Result<StorePragmas, StoreError> {
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    let synchronous: i32 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    let wal_autocheckpoint: i32 = sqlx::query_scalar("PRAGMA wal_autocheckpoint")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    let foreign_keys: i32 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    let busy_timeout: i32 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    Ok(StorePragmas {
        journal_mode,
        synchronous,
        wal_autocheckpoint,
        foreign_keys: foreign_keys != 0,
        busy_timeout,
    })
}
/// Gets the current store configuration (async version)
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn current_store_config(pool: &SqlitePool) -> Result<StoreConfig, StoreError> {
    let pragmas = read_store_pragmas(pool).await?;

    let schema_version: Option<i32> = sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_optional(pool)
        .await
        .map_err(StoreError::Sqlx)?;

    Ok(StoreConfig {
        pragmas,
        schema_version: schema_version.ok_or(StoreError::SchemaVersionMissing)?,
    })
}
