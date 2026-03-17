use sqlx::SqlitePool;

use crate::store::durable::error::DurableError;
use crate::store::durable::operation::get_operation;
use crate::store::durable::step_journal::get_pending_steps;
use crate::store::types::{OperationState, StepRecord};

/// Gets the current timestamp as i64 (Unix epoch seconds)
#[allow(dead_code)]
pub fn current_timestamp() -> Result<i64, DurableError> {
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
