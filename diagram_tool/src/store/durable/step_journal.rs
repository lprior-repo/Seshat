use sqlx::SqlitePool;

use crate::store::durable::error::DurableError;
use crate::store::types::{StepRecord, StepStatus};

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
