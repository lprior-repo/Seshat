//! Bootstrap and pool creation for the async store.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

use super::error::AsyncStoreError;
use super::types::{AsyncStoreBootstrap, AsyncStorePragmas};

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
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
    )
    .fetch_one(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

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
            .map_err(AsyncStoreError::Sqlx)?;

    if events_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id TEXT NOT NULL UNIQUE,
                revision INTEGER NOT NULL,
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
    .map_err(AsyncStoreError::Sqlx)?;

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

/// Reads store pragmas asynchronously.
///
/// # Errors
/// Returns an error if any pragma query fails.
pub async fn read_store_pragmas_async(
    pool: &SqlitePool,
) -> Result<AsyncStorePragmas, AsyncStoreError> {
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
