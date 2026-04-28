use sqlx::SqlitePool;

use crate::store::types::durable_types::{OperationRecord, OperationState, StepRecord, StepStatus};
use crate::store_async::error::AsyncStoreError;

pub async fn ensure_saga_tables(pool: &SqlitePool) -> Result<(), AsyncStoreError> {
    use diagram_models::schema_defs::{
        SCHEMA_SAGA_OPERATIONS_STATE_INDEX, SCHEMA_SAGA_OPERATIONS_TABLE,
        SCHEMA_STEP_JOURNAL_OPERATION_INDEX, SCHEMA_STEP_JOURNAL_TABLE,
    };

    let ops_exists: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='saga_operations'")
            .fetch_one(pool)
            .await
            .map_err(AsyncStoreError::Sqlx)?;

    if ops_exists.0 == 0 {
        sqlx::query(SCHEMA_SAGA_OPERATIONS_TABLE)
            .execute(pool)
            .await
            .map_err(AsyncStoreError::Sqlx)?;
        sqlx::query(SCHEMA_SAGA_OPERATIONS_STATE_INDEX)
            .execute(pool)
            .await
            .map_err(AsyncStoreError::Sqlx)?;
    }

    let journal_exists: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='step_journal'")
            .fetch_one(pool)
            .await
            .map_err(AsyncStoreError::Sqlx)?;

    if journal_exists.0 == 0 {
        sqlx::query(SCHEMA_STEP_JOURNAL_TABLE)
            .execute(pool)
            .await
            .map_err(AsyncStoreError::Sqlx)?;
        sqlx::query(SCHEMA_STEP_JOURNAL_OPERATION_INDEX)
            .execute(pool)
            .await
            .map_err(AsyncStoreError::Sqlx)?;
    }

    Ok(())
}

pub async fn create_saga_operation(
    pool: &SqlitePool,
    record: &OperationRecord,
) -> Result<(), AsyncStoreError> {
    sqlx::query(
        "INSERT INTO saga_operations (operation_id, state, current_step, total_steps, started_at, completed_at, final_revision, error_message, author_id, description)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
    )
    .bind(&record.operation_id)
    .bind(record.state.as_str())
    .bind(record.current_step)
    .bind(record.total_steps)
    .bind(record.started_at)
    .bind(record.completed_at)
    .bind(record.final_revision)
    .bind(&record.error_message)
    .bind(&record.author_id)
    .bind(&record.description)
    .execute(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    Ok(())
}

pub async fn update_operation_state(
    pool: &SqlitePool,
    operation_id: &str,
    state: OperationState,
    current_step: u32,
    error_message: Option<&str>,
    final_revision: Option<i64>,
) -> Result<(), AsyncStoreError> {
    let now = epoch_seconds();

    let completed_at = match state {
        OperationState::Completed | OperationState::Failed => Some(now),
        _ => None,
    };

    sqlx::query(
        "UPDATE saga_operations SET state = ?1, current_step = ?2, error_message = ?3, completed_at = COALESCE(?4, completed_at), final_revision = COALESCE(?5, final_revision) WHERE operation_id = ?6"
    )
    .bind(state.as_str())
    .bind(current_step)
    .bind(error_message)
    .bind(completed_at)
    .bind(final_revision)
    .bind(operation_id)
    .execute(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    Ok(())
}

pub async fn read_operation(
    pool: &SqlitePool,
    operation_id: &str,
) -> Result<Option<OperationRecord>, AsyncStoreError> {
    let result = sqlx::query_as::<_, (String, String, i32, i32, i64, Option<i64>, Option<i64>, Option<String>, String, String)>(
        "SELECT operation_id, state, current_step, total_steps, started_at, completed_at, final_revision, error_message, author_id, description FROM saga_operations WHERE operation_id = ?1"
    )
    .bind(operation_id)
    .fetch_optional(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    match result {
        Some((operation_id, state_str, current_step, total_steps, started_at, completed_at, final_revision, error_message, author_id, description)) => {
            let state = OperationState::from_str(&state_str)
                .ok_or_else(|| AsyncStoreError::Serialization(format!("Unknown operation state: {state_str}")))?;
            Ok(Some(OperationRecord {
                operation_id,
                state,
                current_step: current_step as u32,
                total_steps: total_steps as u32,
                started_at,
                completed_at,
                final_revision,
                error_message,
                author_id,
                description,
            }))
        }
        None => Ok(None),
    }
}

pub async fn record_step_start(
    pool: &SqlitePool,
    operation_id: &str,
    step_index: u32,
    step_name: &str,
) -> Result<(), AsyncStoreError> {
    let now = epoch_seconds();

    sqlx::query(
        "INSERT INTO step_journal (operation_id, step_index, step_name, status, created_at, started_at)
         VALUES (?1, ?2, ?3, 'running', ?4, ?5)
         ON CONFLICT(operation_id, step_index) DO UPDATE SET status = 'running', started_at = ?5"
    )
    .bind(operation_id)
    .bind(step_index)
    .bind(step_name)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    Ok(())
}

pub async fn record_step_complete(
    pool: &SqlitePool,
    operation_id: &str,
    step_index: u32,
    event_revision: Option<i64>,
) -> Result<(), AsyncStoreError> {
    let now = epoch_seconds();

    sqlx::query(
        "UPDATE step_journal SET status = 'completed', event_revision = ?1, completed_at = ?2 WHERE operation_id = ?3 AND step_index = ?4"
    )
    .bind(event_revision)
    .bind(now)
    .bind(operation_id)
    .bind(step_index)
    .execute(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    Ok(())
}

pub async fn record_step_failed(
    pool: &SqlitePool,
    operation_id: &str,
    step_index: u32,
    error_message: &str,
) -> Result<(), AsyncStoreError> {
    let now = epoch_seconds();

    sqlx::query(
        "UPDATE step_journal SET status = 'failed', error_message = ?1, completed_at = ?2 WHERE operation_id = ?3 AND step_index = ?4"
    )
    .bind(error_message)
    .bind(now)
    .bind(operation_id)
    .bind(step_index)
    .execute(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    Ok(())
}

pub async fn read_step_journal(
    pool: &SqlitePool,
    operation_id: &str,
) -> Result<Vec<StepRecord>, AsyncStoreError> {
    let rows = sqlx::query_as::<_, (String, i32, String, String, Option<i64>, i64, Option<i64>, Option<i64>, Option<String>)>(
        "SELECT operation_id, step_index, step_name, status, event_revision, created_at, started_at, completed_at, error_message FROM step_journal WHERE operation_id = ?1 ORDER BY step_index"
    )
    .bind(operation_id)
    .fetch_all(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    rows.into_iter()
        .map(|(operation_id, step_index, step_name, status_str, event_revision, created_at, started_at, completed_at, error_message)| {
            let status = StepStatus::from_str(&status_str)
                .ok_or_else(|| AsyncStoreError::Serialization(format!("Unknown step status: {status_str}")))?;
            Ok(StepRecord {
                operation_id,
                step_index: step_index as u32,
                step_name,
                status,
                event_revision,
                created_at,
                started_at,
                completed_at,
                error_message,
            })
        })
        .collect()
}

fn epoch_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_epoch_seconds_when_called_then_returns_positive_value() {
        let ts = epoch_seconds();
        assert!(ts > 1_700_000_000, "timestamp should be post-2023");
    }
}
