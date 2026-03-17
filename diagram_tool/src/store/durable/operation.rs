use sqlx::SqlitePool;

use crate::store::durable::error::DurableError;
use crate::store::types::{OperationRecord, OperationState};

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
