use sqlx::SqlitePool;

use crate::store::durable::error::DurableError;
use crate::store::types::{OutboxRecord, OutboxStatus, SideEffectType};

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
