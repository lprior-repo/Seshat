//! Revision management for the async store.

use sqlx::SqlitePool;

use super::error::AsyncStoreError;

/// Fetches the latest revision.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn fetch_latest_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    let revision: Option<i64> = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_optional(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    Ok(revision.unwrap_or(0))
}

/// Gets current revision.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn current_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    fetch_latest_revision(pool).await
}

/// Gets next revision.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn next_revision(pool: &SqlitePool) -> Result<i64, AsyncStoreError> {
    let current = current_revision(pool).await?;
    Ok(current + 1)
}
