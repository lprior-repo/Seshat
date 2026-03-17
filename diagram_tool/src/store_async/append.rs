//! Append operations for the async store.

use sqlx::SqlitePool;

use crate::store::types::{BoundedBatch, Revision, ValidEvent};
use diagram_models::envelope::EventEnvelope;

use super::error::{AsyncStoreError, DuplicateKind};
use super::types::{AsyncAppendResult, AsyncBatchAppendResult};

/// Appends an event asynchronously.
///
/// # Errors
/// Returns an error on serialization or database failure.
pub async fn append_event_async(
    pool: &SqlitePool,
    event: ValidEvent,
    expected_revision: Option<Revision>,
) -> Result<AsyncAppendResult, AsyncStoreError> {
    let mut tx = pool.begin().await.map_err(AsyncStoreError::Sqlx)?;

    let current_revision_i64: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let current_revision = u64::try_from(current_revision_i64).map_err(|_| {
        AsyncStoreError::ValidationFailed("Invalid revision from database".to_string())
    })?;

    if let Some(expected) = expected_revision {
        if current_revision != expected.get() {
            return Err(AsyncStoreError::RevisionMismatch {
                expected: i64::try_from(expected.get()).unwrap_or(i64::MAX),
                found: current_revision_i64,
            });
        }
    }

    let new_revision = current_revision + 1;
    let new_revision_i64 = i64::try_from(new_revision).map_err(|_| {
        AsyncStoreError::ValidationFailed("Revision overflow".to_string())
    })?;

    // The payload in ValidEvent is stored as a JSON string that represents the envelope
    let payload = event.payload.as_str();

    sqlx::query(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(event.op_id.as_str())
    .bind(new_revision_i64)
    .bind(payload)
    .bind(event.timestamp.get().to_string())
    .execute(&mut *tx)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    tx.commit().await.map_err(AsyncStoreError::Sqlx)?;

    Ok(AsyncAppendResult {
        revision: new_revision_i64,
        op_id: event.op_id.as_str().to_string(),
        timestamp: event.timestamp.get().cast_signed(),
    })
}

/// Appends a batch of events asynchronously.
///
/// # Errors
/// Returns an error if any append fails.
pub async fn append_batch_async<const MIN: usize, const MAX: usize>(
    pool: &SqlitePool,
    batch: BoundedBatch<MIN, MAX>,
    expected_revision: Option<Revision>,
) -> Result<AsyncBatchAppendResult, AsyncStoreError> {
    let events = batch.into_inner();

    let mut tx = pool.begin().await.map_err(AsyncStoreError::Sqlx)?;

    let current_revision_i64: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let current_revision = u64::try_from(current_revision_i64).map_err(|_| {
        AsyncStoreError::ValidationFailed("Invalid revision from database".to_string())
    })?;

    if let Some(expected) = expected_revision {
        if current_revision != expected.get() {
            return Err(AsyncStoreError::RevisionMismatch {
                expected: i64::try_from(expected.get()).unwrap_or(i64::MAX),
                found: current_revision_i64,
            });
        }
    }

    let batch_size = events.len();
    let start_revision = current_revision + 1;
    let end_revision = current_revision
        + u64::try_from(batch_size).map_err(|_| {
            AsyncStoreError::ValidationFailed(
                "Batch too large for revision calculation".to_string(),
            )
        })?;
    let start_revision_i64 = i64::try_from(start_revision).map_err(|_| {
        AsyncStoreError::ValidationFailed("Revision overflow".to_string())
    })?;
    let end_revision_i64 = i64::try_from(end_revision).map_err(|_| {
        AsyncStoreError::ValidationFailed("Revision overflow".to_string())
    })?;
    let mut op_ids = Vec::with_capacity(batch_size);
    let mut last_timestamp: u64 = 0;

    for (idx, event) in events.into_iter().enumerate() {
        let new_revision = current_revision
            + 1
            + u64::try_from(idx).map_err(|_| {
                AsyncStoreError::ValidationFailed("Index overflow in batch".to_string())
            })?;
        let new_revision_i64 = i64::try_from(new_revision).map_err(|_| {
            AsyncStoreError::ValidationFailed("Revision overflow".to_string())
        })?;

        // The payload in ValidEvent is stored as a JSON string
        let payload = event.payload.as_str();

        sqlx::query(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(event.op_id.as_str())
        .bind(new_revision_i64)
        .bind(payload)
        .bind(event.timestamp.get().to_string())
        .execute(&mut *tx)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

        op_ids.push(event.op_id.as_str().to_string());
        last_timestamp = event.timestamp.get();
    }

    tx.commit().await.map_err(AsyncStoreError::Sqlx)?;

    Ok(AsyncBatchAppendResult {
        start_revision: start_revision_i64,
        end_revision: end_revision_i64,
        count: batch_size,
        op_ids,
        last_timestamp: last_timestamp.cast_signed(),
    })
}

/// Looks up an existing operation by ID.
///
/// # Errors
/// Returns an error if the query fails or timestamp parsing fails.
pub async fn lookup_existing_op_async(
    pool: &SqlitePool,
    op_id: &str,
) -> Result<Option<super::types::EventRecord>, AsyncStoreError> {
    let result = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events WHERE operation_id = ?1",
    )
    .bind(op_id)
    .fetch_optional(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    match result {
        Some((op_id, revision, timestamp_str, payload)) => {
            let timestamp: i64 = timestamp_str.parse().map_err(|_| {
                AsyncStoreError::Serialization("Invalid timestamp format".to_string())
            })?;
            Ok(Some(super::types::EventRecord {
                op_id,
                revision,
                timestamp,
                payload,
            }))
        }
        None => Ok(None),
    }
}

/// Classifies a duplicate.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn classify_duplicate_async(
    existing: &super::types::EventRecord,
    incoming: &EventEnvelope,
) -> Result<DuplicateKind, AsyncStoreError> {
    let incoming_payload = diagram_models::envelope::encode_event_envelope(incoming).map_err(
        |e: diagram_models::envelope::ContractError| AsyncStoreError::Serialization(e.to_string()),
    )?;

    if existing.payload == incoming_payload {
        Ok(DuplicateKind::Exact)
    } else {
        Ok(DuplicateKind::Conflict)
    }
}

/// Appends an event idempotently.
///
/// # Errors
/// Returns an error if serialization or database execution fails.
pub async fn append_idempotent_async(
    pool: &SqlitePool,
    envelope: EventEnvelope,
) -> Result<AsyncAppendResult, AsyncStoreError> {
    let mut tx = pool.begin().await.map_err(AsyncStoreError::Sqlx)?;

    let current_revision_i64: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    let current_revision = u64::try_from(current_revision_i64).map_err(|_| {
        AsyncStoreError::ValidationFailed("Invalid revision from database".to_string())
    })?;

    let new_revision = current_revision + 1;
    let new_revision_i64 = i64::try_from(new_revision).map_err(|_| {
        AsyncStoreError::ValidationFailed("Revision overflow".to_string())
    })?;

    let payload = diagram_models::envelope::encode_event_envelope(&envelope).map_err(
        |e: diagram_models::envelope::ContractError| AsyncStoreError::Serialization(e.to_string()),
    )?;

    let insert_result = sqlx::query(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&envelope.op_id)
    .bind(new_revision_i64)
    .bind(&payload)
    .bind(envelope.timestamp.to_string())
    .execute(&mut *tx)
    .await;

    match insert_result {
        Ok(_) => {
            tx.commit().await.map_err(AsyncStoreError::Sqlx)?;
            Ok(AsyncAppendResult {
                revision: new_revision_i64,
                op_id: envelope.op_id,
                timestamp: envelope.timestamp,
            })
        }
        Err(e) => {
            let is_unique_constraint = e.to_string().contains("UNIQUE constraint failed")
                || e.to_string().contains("constraint failed")
                || e.to_string().contains("constraint");

            if is_unique_constraint {
                let existing = lookup_existing_op_async(pool, &envelope.op_id).await?;

                match existing {
                    Some(record) => {
                        let kind = classify_duplicate_async(&record, &envelope)?;

                        match kind {
                            DuplicateKind::Exact => Ok(AsyncAppendResult {
                                revision: record.revision,
                                op_id: record.op_id,
                                timestamp: record.timestamp,
                            }),
                            DuplicateKind::Conflict => {
                                Err(AsyncStoreError::DuplicateWithConflict(envelope.op_id))
                            }
                        }
                    }
                    None => Err(AsyncStoreError::Sqlx(e)),
                }
            } else {
                Err(AsyncStoreError::Sqlx(e))
            }
        }
    }
}
