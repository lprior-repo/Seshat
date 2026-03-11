//! Durable Workflow Store - Restate-like durable execution for diagram operations
//!
//! This module provides:
//! - Operation tracking for multi-step AI workflows
//! - Step journal for retry/resume capability
//! - Outbox for reliable side-effect delivery
//! - Conflict diff on conditional append rejection
//! - Cursor-based pagination for incremental sync

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use sqlx::SqlitePool;
use std::path::Path;
use thiserror::Error;

use crate::store::types::{
    ConflictDiff, DiffDomainOp, EventCursor, EventPage, EventRecord, OperationRecord,
    OperationState, OutboxRecord, OutboxStatus, SideEffectType, StepRecord, StepStatus,
};

use crate::store_async::{
    create_async_pool, fetch_events_since, fetch_latest_revision, AsyncStoreError as StoreError,
};

// =============================================================================
// Error Types
// =============================================================================

#[derive(Debug, Error)]
pub enum DurableError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Operation not found: {0}")]
    OperationNotFound(String),
    #[error("Operation in invalid state: expected {expected:?}, found {found:?}")]
    OperationStateInvalid {
        expected: OperationState,
        found: OperationState,
    },
    #[error("Step not found: operation {operation_id}, step {step_index}")]
    StepNotFound {
        operation_id: String,
        step_index: u32,
    },
    #[error("Step already completed: operation {operation_id}, step {step_index}")]
    StepAlreadyCompleted {
        operation_id: String,
        step_index: u32,
    },
    #[error("Outbox entry not found: {0}")]
    OutboxNotFound(String),
    #[error("Outbox max retries exceeded: {0}")]
    OutboxMaxRetriesExceeded(String),
    #[error("Cursor parse error: {0}")]
    CursorParseError(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<StoreError> for DurableError {
    fn from(err: StoreError) -> Self {
        match err {
            StoreError::Io(e) => Self::Io(e),
            StoreError::Sqlx(e) => Self::Sqlx(e),
            StoreError::ValidationFailed(s) => Self::ValidationFailed(s),
            StoreError::Serialization(s) => Self::Serialization(s),
            other => Self::ValidationFailed(other.to_string()),
        }
    }
}

// =============================================================================
// Durable Store Bootstrap
// =============================================================================

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

// =============================================================================
// Schema Migration for Durable Tables
// =============================================================================

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

// =============================================================================
// Operation Tracking
// =============================================================================

/// Starts a new durable operation
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn start_operation(
    pool: &SqlitePool,
    operation_id: String,
    total_steps: u32,
    author_id: String,
    description: String,
    timestamp: i64,
) -> Result<OperationRecord, DurableError> {
    sqlx::query(
        "INSERT INTO operations (operation_id, state, current_step, total_steps, started_at, author_id, description)
         VALUES (?1, 'started', 0, ?2, ?3, ?4, ?5)",
    )
    .bind(&operation_id)
    .bind(i64::from(total_steps))
    .bind(timestamp)
    .bind(&author_id)
    .bind(&description)
    .execute(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    Ok(OperationRecord {
        operation_id,
        state: OperationState::Started,
        current_step: 0,
        total_steps,
        started_at: timestamp,
        completed_at: None,
        final_revision: None,
        error_message: None,
        author_id,
        description,
    })
}

/// Gets an operation by ID
///
/// # Errors
/// Returns an error if database query fails or operation not found.
pub async fn get_operation(
    pool: &SqlitePool,
    operation_id: &str,
) -> Result<OperationRecord, DurableError> {
    let result = sqlx::query_as::<_, (
        String,
        String,
        i64,
        i64,
        i64,
        Option<i64>,
        Option<i64>,
        Option<String>,
        String,
        String,
    )>(
        "SELECT operation_id, state, current_step, total_steps, started_at, completed_at, final_revision, error_message, author_id, description
         FROM operations WHERE operation_id = ?1",
    )
    .bind(operation_id)
    .fetch_optional(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    match result {
        Some((
            op_id,
            state_str,
            current_step,
            total_steps,
            started_at,
            completed_at,
            final_revision,
            error_message,
            author_id,
            description,
        )) => {
            let state = OperationState::from_str(&state_str).ok_or_else(|| {
                DurableError::ValidationFailed(format!("Invalid state: {state_str}"))
            })?;

            let current_step_u32 = u32::try_from(current_step)
                .map_err(|_| DurableError::ValidationFailed("current_step overflow".to_string()))?;
            let total_steps_u32 = u32::try_from(total_steps)
                .map_err(|_| DurableError::ValidationFailed("total_steps overflow".to_string()))?;

            Ok(OperationRecord {
                operation_id: op_id,
                state,
                current_step: current_step_u32,
                total_steps: total_steps_u32,
                started_at,
                completed_at,
                final_revision,
                error_message,
                author_id,
                description,
            })
        }
        None => Err(DurableError::OperationNotFound(operation_id.to_string())),
    }
}

/// Updates an operation's state
///
/// # Errors
/// Returns an error if database update fails, operation not found, or state transition is invalid.
pub async fn update_operation_state(
    pool: &SqlitePool,
    operation_id: &str,
    new_state: OperationState,
    current_step: Option<u32>,
    final_revision: Option<i64>,
    error_message: Option<String>,
) -> Result<OperationRecord, DurableError> {
    let mut tx = pool.begin().await.map_err(DurableError::Sqlx)?;

    // Get current state
    let current: (String,) = sqlx::query_as("SELECT state FROM operations WHERE operation_id = ?1")
        .bind(operation_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(DurableError::Sqlx)?
        .ok_or_else(|| DurableError::OperationNotFound(operation_id.to_string()))?;

    let current_state_str = &current.0;
    let current_state = OperationState::from_str(current_state_str).ok_or_else(|| {
        DurableError::ValidationFailed(format!("Invalid state: {current_state_str}"))
    })?;

    // Validate state transition
    validate_state_transition(current_state, new_state)?;

    let completed_at: Option<i64> = match new_state {
        OperationState::Completed | OperationState::Failed => Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| DurableError::ValidationFailed(e.to_string()))?
                .as_secs()
                .cast_signed(),
        ),
        _ => None,
    };

    sqlx::query(
        "UPDATE operations SET state = ?1, current_step = COALESCE(?2, current_step),
         final_revision = ?3, error_message = ?4, completed_at = ?5
         WHERE operation_id = ?6",
    )
    .bind(new_state.as_str())
    .bind(current_step.map(i64::from))
    .bind(final_revision)
    .bind(&error_message)
    .bind(completed_at)
    .bind(operation_id)
    .execute(&mut *tx)
    .await
    .map_err(DurableError::Sqlx)?;

    tx.commit().await.map_err(DurableError::Sqlx)?;

    get_operation(pool, operation_id).await
}

/// Validates state transitions
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::match_same_arms)]
fn validate_state_transition(from: OperationState, to: OperationState) -> Result<(), DurableError> {
    match (from, to) {
        // Valid transitions
        (OperationState::Started, OperationState::InProgress) => Ok(()),
        (OperationState::Started, OperationState::Failed) => Ok(()),
        (OperationState::InProgress, OperationState::InProgress) => Ok(()),
        (OperationState::InProgress, OperationState::Completed) => Ok(()),
        (OperationState::InProgress, OperationState::Failed) => Ok(()),
        (OperationState::Completed, OperationState::Completed) => Ok(()),
        (OperationState::Failed, OperationState::Failed) => Ok(()),
        // Invalid transitions
        _ => Err(DurableError::OperationStateInvalid {
            expected: to,
            found: from,
        }),
    }
}

/// Gets all operations in a specific state
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_operations_by_state(
    pool: &SqlitePool,
    state: OperationState,
) -> Result<Vec<OperationRecord>, DurableError> {
    let rows = sqlx::query_as::<_, (
        String,
        String,
        i64,
        i64,
        i64,
        Option<i64>,
        Option<i64>,
        Option<String>,
        String,
        String,
    )>(
        "SELECT operation_id, state, current_step, total_steps, started_at, completed_at, final_revision, error_message, author_id, description
         FROM operations WHERE state = ?1 ORDER BY started_at ASC",
    )
    .bind(state.as_str())
    .fetch_all(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    let mut operations = Vec::with_capacity(rows.len());
    for row in rows {
        let state_str = &row.1;
        let state = OperationState::from_str(state_str)
            .ok_or_else(|| DurableError::ValidationFailed(format!("Invalid state: {state_str}")))?;

        let current_step_u32 = u32::try_from(row.2)
            .map_err(|_| DurableError::ValidationFailed("current_step overflow".to_string()))?;
        let total_steps_u32 = u32::try_from(row.3)
            .map_err(|_| DurableError::ValidationFailed("total_steps overflow".to_string()))?;

        operations.push(OperationRecord {
            operation_id: row.0,
            state,
            current_step: current_step_u32,
            total_steps: total_steps_u32,
            started_at: row.4,
            completed_at: row.5,
            final_revision: row.6,
            error_message: row.7,
            author_id: row.8,
            description: row.9,
        });
    }

    Ok(operations)
}

// =============================================================================
// Step Journal
// =============================================================================

/// Records a step in the step journal
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn record_step(
    pool: &SqlitePool,
    operation_id: String,
    step_index: u32,
    step_name: String,
    timestamp: i64,
) -> Result<StepRecord, DurableError> {
    sqlx::query(
        "INSERT INTO step_journal (operation_id, step_index, step_name, status, created_at)
         VALUES (?1, ?2, ?3, 'pending', ?4)",
    )
    .bind(&operation_id)
    .bind(i64::from(step_index))
    .bind(&step_name)
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    Ok(StepRecord {
        operation_id,
        step_index,
        step_name,
        status: StepStatus::Pending,
        event_revision: None,
        created_at: timestamp,
        started_at: None,
        completed_at: None,
        error_message: None,
    })
}

/// Gets a step from the journal
///
/// # Errors
/// Returns an error if database query fails or step not found.
pub async fn get_step(
    pool: &SqlitePool,
    operation_id: &str,
    step_index: u32,
) -> Result<StepRecord, DurableError> {
    let result = sqlx::query_as::<_, (
        String,
        i64,
        String,
        String,
        Option<i64>,
        i64,
        Option<i64>,
        Option<i64>,
        Option<String>,
    )>(
        "SELECT operation_id, step_index, step_name, status, event_revision, created_at, started_at, completed_at, error_message
         FROM step_journal WHERE operation_id = ?1 AND step_index = ?2",
    )
    .bind(operation_id)
    .bind(i64::from(step_index))
    .fetch_optional(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    match result {
        Some((
            op_id,
            idx,
            name,
            status_str,
            event_revision,
            created_at,
            started_at,
            completed_at,
            error_message,
        )) => {
            let status = StepStatus::from_str(&status_str).ok_or_else(|| {
                DurableError::ValidationFailed(format!("Invalid status: {status_str}"))
            })?;

            let step_index_u32 = u32::try_from(idx)
                .map_err(|_| DurableError::ValidationFailed("step_index overflow".to_string()))?;

            Ok(StepRecord {
                operation_id: op_id,
                step_index: step_index_u32,
                step_name: name,
                status,
                event_revision,
                created_at,
                started_at,
                completed_at,
                error_message,
            })
        }
        None => Err(DurableError::StepNotFound {
            operation_id: operation_id.to_string(),
            step_index,
        }),
    }
}

/// Updates step status and optionally marks it complete
///
/// # Errors
/// Returns an error if database update fails, step not found, or step already completed.
pub async fn update_step_status(
    pool: &SqlitePool,
    operation_id: &str,
    step_index: u32,
    new_status: StepStatus,
    event_revision: Option<i64>,
    error_message: Option<String>,
) -> Result<StepRecord, DurableError> {
    let mut tx = pool.begin().await.map_err(DurableError::Sqlx)?;

    // Get current status
    let current: (String,) = sqlx::query_as(
        "SELECT status FROM step_journal WHERE operation_id = ?1 AND step_index = ?2",
    )
    .bind(operation_id)
    .bind(i64::from(step_index))
    .fetch_optional(&mut *tx)
    .await
    .map_err(DurableError::Sqlx)?
    .ok_or_else(|| DurableError::StepNotFound {
        operation_id: operation_id.to_string(),
        step_index,
    })?;

    let current_status_str = &current.0;
    let current_status = StepStatus::from_str(current_status_str).ok_or_else(|| {
        DurableError::ValidationFailed(format!("Invalid status: {current_status_str}"))
    })?;

    // Cannot update a completed or skipped step (unless re-running)
    if (current_status == StepStatus::Completed || current_status == StepStatus::Skipped)
        && new_status != StepStatus::Running
    {
        return Err(DurableError::StepAlreadyCompleted {
            operation_id: operation_id.to_string(),
            step_index,
        });
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| DurableError::ValidationFailed(e.to_string()))?
        .as_secs()
        .cast_signed();

    let (started_at, completed_at) = match new_status {
        StepStatus::Running => (Some(timestamp), None),
        StepStatus::Completed | StepStatus::Skipped | StepStatus::Failed => (None, Some(timestamp)),
        StepStatus::Pending => (None, None),
    };

    sqlx::query(
        "UPDATE step_journal SET status = ?1, event_revision = COALESCE(?2, event_revision),
         started_at = COALESCE(?3, started_at), completed_at = ?4, error_message = ?5
         WHERE operation_id = ?6 AND step_index = ?7",
    )
    .bind(new_status.as_str())
    .bind(event_revision)
    .bind(started_at)
    .bind(completed_at)
    .bind(&error_message)
    .bind(operation_id)
    .bind(i64::from(step_index))
    .execute(&mut *tx)
    .await
    .map_err(DurableError::Sqlx)?;

    tx.commit().await.map_err(DurableError::Sqlx)?;

    get_step(pool, operation_id, step_index).await
}

/// Gets all pending steps for an operation (for retry/resume)
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_pending_steps(
    pool: &SqlitePool,
    operation_id: &str,
) -> Result<Vec<StepRecord>, DurableError> {
    let rows = sqlx::query_as::<_, (
        String,
        i64,
        String,
        String,
        Option<i64>,
        i64,
        Option<i64>,
        Option<i64>,
        Option<String>,
    )>(
        "SELECT operation_id, step_index, step_name, status, event_revision, created_at, started_at, completed_at, error_message
         FROM step_journal WHERE operation_id = ?1 AND status IN ('pending', 'running', 'failed')
         ORDER BY step_index ASC",
    )
    .bind(operation_id)
    .fetch_all(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    let mut steps = Vec::with_capacity(rows.len());
    for row in rows {
        let status_str = &row.3;
        let status = StepStatus::from_str(status_str).ok_or_else(|| {
            DurableError::ValidationFailed(format!("Invalid status: {status_str}"))
        })?;

        let step_index_u32 = u32::try_from(row.1)
            .map_err(|_| DurableError::ValidationFailed("step_index overflow".to_string()))?;

        steps.push(StepRecord {
            operation_id: row.0,
            step_index: step_index_u32,
            step_name: row.2,
            status,
            event_revision: row.4,
            created_at: row.5,
            started_at: row.6,
            completed_at: row.7,
            error_message: row.8,
        });
    }

    Ok(steps)
}

/// Marks a step as skipped (for retry scenario - step already completed)
///
/// # Errors
/// Returns an error if database update fails, step not found, or step already completed.
pub async fn skip_step(
    pool: &SqlitePool,
    operation_id: &str,
    step_index: u32,
) -> Result<StepRecord, DurableError> {
    update_step_status(
        pool,
        operation_id,
        step_index,
        StepStatus::Skipped,
        None,
        None,
    )
    .await
}

// =============================================================================
// Outbox (Side Effects)
// =============================================================================

/// Adds an entry to the outbox for reliable side-effect delivery
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn add_outbox_entry(
    pool: &SqlitePool,
    id: String,
    side_effect_type: SideEffectType,
    payload: String,
    event_revision: i64,
    max_retries: u32,
    timestamp: i64,
) -> Result<OutboxRecord, DurableError> {
    sqlx::query(
        "INSERT INTO outbox (id, side_effect_type, payload, event_revision, status, retry_count, max_retries, created_at)
         VALUES (?1, ?2, ?3, ?4, 'pending', 0, ?5, ?6)",
    )
    .bind(&id)
    .bind(side_effect_type.as_str())
    .bind(&payload)
    .bind(event_revision)
    .bind(i64::from(max_retries))
    .bind(timestamp)
    .execute(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    Ok(OutboxRecord {
        id,
        side_effect_type,
        payload,
        event_revision,
        status: OutboxStatus::Pending,
        retry_count: 0,
        max_retries,
        created_at: timestamp,
        dispatched_at: None,
        acknowledged_at: None,
        last_error: None,
    })
}

/// Gets an outbox entry by ID
///
/// # Errors
/// Returns an error if database query fails or outbox entry not found.
pub async fn get_outbox_entry(pool: &SqlitePool, id: &str) -> Result<OutboxRecord, DurableError> {
    let result = sqlx::query_as::<_, (
        String,
        String,
        String,
        i64,
        String,
        i64,
        i64,
        i64,
        Option<i64>,
        Option<i64>,
        Option<String>,
    )>(
        "SELECT id, side_effect_type, payload, event_revision, status, retry_count, max_retries, created_at, dispatched_at, acknowledged_at, last_error
         FROM outbox WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    match result {
        Some((
            id,
            type_str,
            payload,
            event_revision,
            status_str,
            retry_count,
            max_retries,
            created_at,
            dispatched_at,
            acknowledged_at,
            last_error,
        )) => {
            let side_effect_type = SideEffectType::from_str(&type_str).ok_or_else(|| {
                DurableError::ValidationFailed(format!("Invalid type: {type_str}"))
            })?;
            let status = OutboxStatus::from_str(&status_str).ok_or_else(|| {
                DurableError::ValidationFailed(format!("Invalid status: {status_str}"))
            })?;

            let retry_count_u32 = u32::try_from(retry_count)
                .map_err(|_| DurableError::ValidationFailed("retry_count overflow".to_string()))?;
            let max_retries_u32 = u32::try_from(max_retries)
                .map_err(|_| DurableError::ValidationFailed("max_retries overflow".to_string()))?;

            Ok(OutboxRecord {
                id,
                side_effect_type,
                payload,
                event_revision,
                status,
                retry_count: retry_count_u32,
                max_retries: max_retries_u32,
                created_at,
                dispatched_at,
                acknowledged_at,
                last_error,
            })
        }
        None => Err(DurableError::OutboxNotFound(id.to_string())),
    }
}

/// Marks an outbox entry as dispatched
///
/// # Errors
/// Returns an error if database update fails or outbox entry not found.
pub async fn mark_outbox_dispatched(
    pool: &SqlitePool,
    id: &str,
) -> Result<OutboxRecord, DurableError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| DurableError::ValidationFailed(e.to_string()))?
        .as_secs()
        .cast_signed();

    sqlx::query("UPDATE outbox SET status = 'dispatched', dispatched_at = ?1 WHERE id = ?2")
        .bind(timestamp)
        .bind(id)
        .execute(pool)
        .await
        .map_err(DurableError::Sqlx)?;

    get_outbox_entry(pool, id).await
}

/// Acknowledges an outbox entry (external system confirmed processing)
///
/// # Errors
/// Returns an error if database update fails or outbox entry not found.
pub async fn acknowledge_outbox(pool: &SqlitePool, id: &str) -> Result<OutboxRecord, DurableError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| DurableError::ValidationFailed(e.to_string()))?
        .as_secs()
        .cast_signed();

    sqlx::query("UPDATE outbox SET status = 'acknowledged', acknowledged_at = ?1 WHERE id = ?2")
        .bind(timestamp)
        .bind(id)
        .execute(pool)
        .await
        .map_err(DurableError::Sqlx)?;

    get_outbox_entry(pool, id).await
}

/// Marks an outbox entry as failed and increments retry count
///
/// # Errors
/// Returns an error if database update fails, outbox entry not found, or max retries exceeded.
pub async fn mark_outbox_failed(
    pool: &SqlitePool,
    id: &str,
    error_message: String,
) -> Result<OutboxRecord, DurableError> {
    let entry = get_outbox_entry(pool, id).await?;

    if entry.retry_count >= entry.max_retries {
        return Err(DurableError::OutboxMaxRetriesExceeded(id.to_string()));
    }

    sqlx::query(
        "UPDATE outbox SET status = 'failed', retry_count = retry_count + 1, last_error = ?1 WHERE id = ?2",
    )
    .bind(&error_message)
    .bind(id)
    .execute(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    get_outbox_entry(pool, id).await
}

/// Gets pending outbox entries (for processing)
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_pending_outbox(
    pool: &SqlitePool,
    limit: u32,
) -> Result<Vec<OutboxRecord>, DurableError> {
    let rows = sqlx::query_as::<_, (
        String,
        String,
        String,
        i64,
        String,
        i64,
        i64,
        i64,
        Option<i64>,
        Option<i64>,
        Option<String>,
    )>(
        "SELECT id, side_effect_type, payload, event_revision, status, retry_count, max_retries, created_at, dispatched_at, acknowledged_at, last_error
         FROM outbox WHERE status IN ('pending', 'failed') ORDER BY created_at ASC LIMIT ?1",
    )
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let side_effect_type_str = &row.1;
        let side_effect_type = SideEffectType::from_str(side_effect_type_str).ok_or_else(|| {
            DurableError::ValidationFailed(format!("Invalid type: {side_effect_type_str}"))
        })?;
        let status_str = &row.4;
        let status = OutboxStatus::from_str(status_str).ok_or_else(|| {
            DurableError::ValidationFailed(format!("Invalid status: {status_str}"))
        })?;

        let retry_count_u32 = u32::try_from(row.5)
            .map_err(|_| DurableError::ValidationFailed("retry_count overflow".to_string()))?;
        let max_retries_u32 = u32::try_from(row.6)
            .map_err(|_| DurableError::ValidationFailed("max_retries overflow".to_string()))?;

        entries.push(OutboxRecord {
            id: row.0,
            side_effect_type,
            payload: row.2,
            event_revision: row.3,
            status,
            retry_count: retry_count_u32,
            max_retries: max_retries_u32,
            created_at: row.7,
            dispatched_at: row.8,
            acknowledged_at: row.9,
            last_error: row.10,
        });
    }

    Ok(entries)
}

// =============================================================================
// Conflict Diff
// =============================================================================

/// Generates a rich diff when a conditional append is rejected
///
/// # Errors
/// Returns an error if database query fails.
pub async fn generate_conflict_diff(
    pool: &SqlitePool,
    assumed_revision: i64,
) -> Result<ConflictDiff, DurableError> {
    let current_revision = fetch_latest_revision(pool).await?;

    if current_revision <= assumed_revision {
        return Ok(ConflictDiff {
            assumed_revision,
            actual_revision: current_revision,
            changes: vec![],
            first_change_timestamp: 0,
            first_change_author: String::new(),
        });
    }

    let events = fetch_events_since(pool, assumed_revision).await?;

    let mut changes = Vec::with_capacity(events.len());
    let mut first_timestamp: i64 = 0;
    let mut first_author = String::new();

    for event in &events {
        if first_timestamp == 0 {
            first_timestamp = event.timestamp;
            first_author = "unknown".to_string();
        }
        changes.push(DiffDomainOp::Other);
    }

    Ok(ConflictDiff {
        assumed_revision,
        actual_revision: current_revision,
        changes,
        first_change_timestamp: first_timestamp,
        first_change_author: first_author,
    })
}

// =============================================================================
// Cursor-based Pagination
// =============================================================================

/// Fetches events using cursor-based pagination
///
/// # Errors
/// Returns an error if database query fails or timestamp parsing fails.
pub async fn fetch_events_cursor(
    pool: &SqlitePool,
    cursor: EventCursor,
) -> Result<EventPage, DurableError> {
    let limit = cursor.limit.min(1000);

    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events
         WHERE revision > ?1 ORDER BY revision ASC LIMIT ?2",
    )
    .bind(cursor.revision)
    .bind(i64::from(limit + 1))
    .fetch_all(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    let has_more = rows.len() > limit as usize;
    let events: Vec<_> = rows.into_iter().take(limit as usize).collect();

    let mut event_records = Vec::with_capacity(events.len());
    let mut last_revision = cursor.revision;

    for (op_id, revision, timestamp_str, payload) in events {
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| DurableError::Serialization("Invalid timestamp format".to_string()))?;

        last_revision = revision;
        event_records.push(EventRecord {
            op_id,
            revision,
            timestamp,
            payload,
        });
    }

    let next_cursor = if has_more {
        Some(EventCursor::new(last_revision, cursor.limit))
    } else {
        None
    };

    Ok(EventPage {
        events: event_records,
        next_cursor,
        has_more,
    })
}

/// Parses a cursor from a string
///
/// # Errors
/// Returns an error if cursor string format is invalid.
pub fn parse_cursor(cursor_str: &str) -> Result<EventCursor, DurableError> {
    let parts: Vec<&str> = cursor_str.split(':').collect();
    if parts.len() != 2 {
        return Err(DurableError::CursorParseError(
            "Expected format: revision:limit".to_string(),
        ));
    }

    let revision = parts[0]
        .parse()
        .map_err(|_| DurableError::CursorParseError("Invalid revision".to_string()))?;
    let limit = parts[1]
        .parse()
        .map_err(|_| DurableError::CursorParseError("Invalid limit".to_string()))?;

    Ok(EventCursor::new(revision, limit))
}

/// Serializes a cursor to a string
#[must_use]
pub fn serialize_cursor(cursor: &EventCursor) -> String {
    format!("{}:{}", cursor.revision, cursor.limit)
}

// =============================================================================
// High-level Workflow Helpers
// =============================================================================

/// Gets the current timestamp as i64 (Unix epoch seconds)
#[allow(dead_code)]
fn current_timestamp() -> Result<i64, DurableError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| DurableError::ValidationFailed(e.to_string()))
        .map(|d| d.as_secs().cast_signed())
}

/// Checks if an operation can be resumed (has pending/failed steps)
///
/// # Errors
/// Returns an error if database query fails or operation not found.
pub async fn can_resume_operation(
    pool: &SqlitePool,
    operation_id: &str,
) -> Result<bool, DurableError> {
    let operation = get_operation(pool, operation_id).await?;

    match operation.state {
        OperationState::InProgress | OperationState::Failed => {
            let pending = get_pending_steps(pool, operation_id).await?;
            Ok(!pending.is_empty())
        }
        _ => Ok(false),
    }
}

/// Gets the next step to execute in an operation (for resume)
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_next_step(
    pool: &SqlitePool,
    operation_id: &str,
) -> Result<Option<StepRecord>, DurableError> {
    let pending = get_pending_steps(pool, operation_id).await?;

    // Return the first pending/running/failed step
    Ok(pending.into_iter().next())
}
