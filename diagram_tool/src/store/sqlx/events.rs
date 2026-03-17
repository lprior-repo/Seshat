use crate::store::sqlx::error::*;
use crate::store::sqlx::models::*;
use sqlx::SqlitePool;
use diagram_models::envelope::{encode_event_envelope, parse_event_envelope, EventEnvelope};

///
/// # Errors
///
/// Returns a `StoreError` if the batch is empty or the append fails.
#[allow(clippy::cast_possible_wrap)]
pub async fn append_batch(
    pool: &SqlitePool,
    ops: Vec<EventEnvelope>,
    expected_revision: Option<i64>,
) -> Result<BatchAppendResult, StoreError> {
    if ops.is_empty() {
        return Err(StoreError::EmptyBatch);
    }

    let mut tx = pool.begin().await.map_err(StoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(StoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    let batch_size = ops.len();
    // Cast usize to i64 - batch_size is small (bounded by MAX_OPS), safe for revision numbers
    let start_revision = current_revision + 1;
    let end_revision = current_revision + batch_size as i64;
    let mut op_ids = Vec::with_capacity(batch_size);
    let mut last_timestamp = 0i64;

    for (idx, envelope) in ops.into_iter().enumerate() {
        // Cast usize to i64 - idx is bounded by batch_size, safe for revision numbers
        let new_revision = current_revision + 1 + idx as i64;

        let payload = encode_event_envelope(&envelope)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        let insert_result = sqlx::query(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(&envelope.op_id)
        .bind(new_revision)
        .bind(&payload)
        .bind(envelope.timestamp.to_string())
        .execute(&mut *tx)
        .await;

        match insert_result {
            Ok(_) => {}
            Err(e) => {
                if e.to_string().contains("events.revision") {
                    return Err(StoreError::RevisionMismatch {
                        expected: current_revision,
                        found: current_revision + 1,
                    });
                }
                return Err(StoreError::Sqlx(e));
            }
        }

        op_ids.push(envelope.op_id);
        last_timestamp = envelope.timestamp;
    }

    tx.commit().await.map_err(StoreError::Sqlx)?;

    Ok(BatchAppendResult {
        start_revision,
        end_revision,
        count: batch_size,
        op_ids,
        last_timestamp,
    })
}
/// Looks up an existing operation by ID
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn lookup_existing_op(
    pool: &SqlitePool,
    op_id: &str,
) -> Result<Option<EventRecord>, StoreError> {
    let result = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events WHERE operation_id = ?1",
    )
    .bind(op_id)
    .fetch_optional(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    match result {
        Some((op_id, revision, timestamp_str, payload)) => {
            let timestamp: i64 = timestamp_str
                .parse()
                .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;
            Ok(Some(EventRecord {
                op_id,
                revision,
                timestamp,
                payload,
            }))
        }
        None => Ok(None),
    }
}

/// Classifies a duplicate as exact or conflicting
///
/// # Errors
///
/// Returns a `StoreError` if serialization fails.
#[allow(clippy::unused_async)]
pub async fn classify_duplicate(
    existing: &EventRecord,
    incoming: &EventEnvelope,
) -> Result<DuplicateKind, StoreError> {
    let incoming_payload =
        encode_event_envelope(incoming).map_err(|e| StoreError::Serialization(e.to_string()))?;

    if existing.payload == incoming_payload {
        Ok(DuplicateKind::Exact)
    } else {
        Ok(DuplicateKind::Conflict)
    }
}

/// Appends an event idempotently (handles duplicates gracefully)
///
/// # Errors
///
/// Returns a `StoreError` if the append fails.
pub async fn append_idempotent(
    pool: &SqlitePool,
    envelope: EventEnvelope,
) -> Result<AppendResult, StoreError> {
    let mut tx = pool.begin().await.map_err(StoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    let payload =
        encode_event_envelope(&envelope).map_err(|e| StoreError::Serialization(e.to_string()))?;

    let new_revision = current_revision + 1;
    let insert_result = sqlx::query(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&envelope.op_id)
    .bind(new_revision)
    .bind(&payload)
    .bind(envelope.timestamp.to_string())
    .execute(&mut *tx)
    .await;

    match insert_result {
        Ok(_) => {
            tx.commit().await.map_err(StoreError::Sqlx)?;
            Ok(AppendResult {
                revision: new_revision,
                op_id: envelope.op_id,
                timestamp: envelope.timestamp,
            })
        }
        Err(e) => {
            let is_unique_constraint = e.to_string().contains("UNIQUE constraint failed")
                || e.to_string().contains("constraint failed")
                || e.to_string().contains("constraint");

            if is_unique_constraint {
                // Rollback this transaction and look up the existing record
                drop(tx);
                let existing = lookup_existing_op(pool, &envelope.op_id).await?;

                match existing {
                    Some(record) => {
                        let kind = classify_duplicate(&record, &envelope).await?;

                        match kind {
                            DuplicateKind::Exact => Ok(AppendResult {
                                revision: record.revision,
                                op_id: record.op_id,
                                timestamp: record.timestamp,
                            }),
                            DuplicateKind::Conflict => {
                                Err(StoreError::DuplicateWithConflict(envelope.op_id))
                            }
                        }
                    }
                    None => Err(StoreError::Sqlx(e)),
                }
            } else {
                Err(StoreError::Sqlx(e))
            }
        }
    }
}

