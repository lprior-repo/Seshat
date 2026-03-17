use sqlx::SqlitePool;
use std::path::Path;

use crate::store::durable::error::DurableError;
use crate::store_async::create_async_pool;

/// Configuration for the durable store
#[derive(Debug, Clone)]
pub struct DurableConfig {
    pub max_retries: u32,
    pub batch_size: usize,
}

impl Default for DurableConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            batch_size: 100,
        }
    }
}

/// Bootstrap result for durable store
pub struct DurableStoreBootstrap {
    pub pool: SqlitePool,
    pub config: DurableConfig,
}

/// Runs schema migration for durable workflow tables
///
/// # Errors
/// Returns an error if database migration fails.
#[allow(clippy::too_many_lines)]
pub async fn run_durable_migration(pool: &SqlitePool) -> Result<(), DurableError> {
    // Operations table - tracks multi-step AI operations
    let operations_table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='operations'",
    )
    .fetch_one(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    if operations_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS operations (
                operation_id TEXT NOT NULL PRIMARY KEY,
                state TEXT NOT NULL DEFAULT 'started',
                current_step INTEGER NOT NULL DEFAULT 0,
                total_steps INTEGER NOT NULL DEFAULT 1,
                started_at INTEGER NOT NULL,
                completed_at INTEGER,
                final_revision INTEGER,
                error_message TEXT,
                author_id TEXT NOT NULL,
                description TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await
        .map_err(DurableError::Sqlx)?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_operations_state ON operations(state)")
            .execute(pool)
            .await
            .map_err(DurableError::Sqlx)?;
    }

    // Step journal table - tracks individual steps in operations
    let step_journal_table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='step_journal'",
    )
    .fetch_one(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    if step_journal_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS step_journal (
                operation_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                step_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                event_revision INTEGER,
                created_at INTEGER NOT NULL,
                started_at INTEGER,
                completed_at INTEGER,
                error_message TEXT,
                PRIMARY KEY (operation_id, step_index)
            )",
        )
        .execute(pool)
        .await
        .map_err(DurableError::Sqlx)?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_step_journal_status ON step_journal(status)")
            .execute(pool)
            .await
            .map_err(DurableError::Sqlx)?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_step_journal_operation ON step_journal(operation_id)",
        )
        .execute(pool)
        .await
        .map_err(DurableError::Sqlx)?;
    }

    // Outbox table - reliable side-effect delivery
    let outbox_table_exists: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='outbox'")
            .fetch_one(pool)
            .await
            .map_err(DurableError::Sqlx)?;

    if outbox_table_exists.0 == 0 {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS outbox (
                id TEXT NOT NULL PRIMARY KEY,
                side_effect_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                event_revision INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL DEFAULT 3,
                created_at INTEGER NOT NULL,
                dispatched_at INTEGER,
                acknowledged_at INTEGER,
                last_error TEXT
            )",
        )
        .execute(pool)
        .await
        .map_err(DurableError::Sqlx)?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_outbox_status ON outbox(status)")
            .execute(pool)
            .await
            .map_err(DurableError::Sqlx)?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_outbox_event_revision ON outbox(event_revision)",
        )
        .execute(pool)
        .await
        .map_err(DurableError::Sqlx)?;
    }

    Ok(())
}

/// Bootstraps the durable store
///
/// # Errors
/// Returns an error if database connection or migration fails.
pub async fn bootstrap_durable_store(
    db_path: &Path,
    config: DurableConfig,
) -> Result<DurableStoreBootstrap, DurableError> {
    let pool = create_async_pool(db_path).await?;
    run_durable_migration(&pool).await?;
    Ok(DurableStoreBootstrap { pool, config })
}
