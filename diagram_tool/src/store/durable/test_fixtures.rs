use sqlx::SqlitePool;
use tempfile::TempDir;

use crate::store::durable::bootstrap::{bootstrap_durable_store, DurableConfig};
use crate::store::durable::error::DurableError;

/// Creates a test database pool with migrations run
pub async fn create_test_pool() -> Result<(SqlitePool, TempDir), DurableError> {
    let temp_dir = TempDir::new().map_err(|e| {
        DurableError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_durable_store(&db_path, DurableConfig::default()).await?;

    Ok((bootstrap.pool, temp_dir))
}

/// Creates a test pool and ensures the events table exists for cursor tests
pub async fn create_test_pool_with_events() -> Result<(SqlitePool, TempDir), DurableError> {
    let (pool, temp_dir) = create_test_pool().await?;

    // Create events table if it doesn't exist (for cursor tests)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT NOT NULL,
            revision INTEGER NOT NULL,
            timestamp TEXT NOT NULL,
            payload TEXT NOT NULL,
            PRIMARY KEY (operation_id, revision)
        )",
    )
    .execute(&pool)
    .await
    .map_err(DurableError::Sqlx)?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)")
        .execute(&pool)
        .await
        .map_err(DurableError::Sqlx)?;

    Ok((pool, temp_dir))
}
