//! Fetch and integrity operations for the async store.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

use super::error::AsyncStoreError;
use super::types::EventRecord;

/// Fetches events since a given revision.
///
/// # Errors
/// Returns an error on query failure.
pub async fn fetch_events_since(
    pool: &SqlitePool,
    revision: i64,
) -> Result<Vec<EventRecord>, AsyncStoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events WHERE revision > ?1 ORDER BY revision ASC"
    )
    .bind(revision)
    .fetch_all(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    let mut events = Vec::with_capacity(rows.len());
    for (op_id, revision, timestamp_str, payload) in rows {
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| AsyncStoreError::Serialization("Invalid timestamp format".to_string()))?;
        events.push(EventRecord {
            op_id,
            revision,
            timestamp,
            payload,
        });
    }

    Ok(events)
}

/// Fetches all events.
///
/// # Errors
/// Returns an error on query failure.
pub async fn fetch_all_events(pool: &SqlitePool) -> Result<Vec<EventRecord>, AsyncStoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events ORDER BY revision ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    let mut events = Vec::with_capacity(rows.len());
    for (op_id, revision, timestamp_str, payload) in rows {
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| AsyncStoreError::Serialization("Invalid timestamp format".to_string()))?;
        events.push(EventRecord {
            op_id,
            revision,
            timestamp,
            payload,
        });
    }

    Ok(events)
}

/// Performs an integrity check on the database.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn integrity_check_async(db_path: &Path) -> Result<Vec<String>, AsyncStoreError> {
    let connection_string = format!("sqlite:{}?mode=ro", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let results: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_all(&pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    pool.close().await;

    Ok(results)
}

/// Resets the store by deleting all events.
///
/// This is used when opening a new document to clear any existing event history.
///
/// # Errors
/// Returns an error if the deletion fails.
pub async fn reset_store_async(pool: &SqlitePool) -> Result<(), AsyncStoreError> {
    sqlx::query("DELETE FROM events")
        .execute(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;
    Ok(())
}

/// Opens the database in recovery mode.
///
/// # Errors
/// Returns an error if the connection fails or pragmas cannot be set.
pub async fn open_recovery_mode_async(db_path: &Path) -> Result<SqlitePool, AsyncStoreError> {
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    sqlx::query("PRAGMA journal_mode=Delete")
        .execute(&pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    sqlx::query("PRAGMA synchronous=NORMAL")
        .execute(&pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    Ok(pool)
}
