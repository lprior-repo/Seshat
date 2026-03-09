use super::errors::StoreError;
use super::types::BatchAppendResult;
use crate::models::envelope::{encode_event_envelope, EventEnvelope};
use rusqlite::{Connection, OptionalExtension};

pub fn append_batch(
    conn: &mut Connection,
    ops: Vec<EventEnvelope>,
    expected_revision: Option<i64>,
) -> Result<BatchAppendResult, StoreError> {
    if ops.is_empty() {
        return Err(StoreError::EmptyBatch);
    }

    let tx = conn.transaction().map_err(StoreError::Sqlite)?;

    let current_revision: i64 = tx
        .query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |row| {
            row.get(0)
        })
        .map_err(StoreError::Sqlite)?;

    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(StoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    let mut existing_count = 0;
    for op in &ops {
        let mut stmt = tx
            .prepare("SELECT payload FROM events WHERE operation_id = ?1")
            .map_err(StoreError::Sqlite)?;
        let existing_payload: Option<String> = stmt
            .query_row([&op.op_id], |row| row.get(0))
            .optional()
            .map_err(StoreError::Sqlite)?;

        if let Some(payload) = existing_payload {
            let incoming_payload =
                encode_event_envelope(op).map_err(|e| StoreError::Serialization(e.to_string()))?;
            if payload == incoming_payload {
                existing_count += 1;
            } else {
                return Err(StoreError::DuplicateWithConflict(op.op_id.clone()));
            }
        }
    }

    if existing_count > 0 {
        if existing_count == ops.len() {
            let last_timestamp = ops.last().map(|op| op.timestamp).unwrap_or(0);
            return Ok(BatchAppendResult {
                start_revision: current_revision,
                end_revision: current_revision,
                count: 0,
                op_ids: ops.into_iter().map(|op| op.op_id).collect(),
                last_timestamp,
            });
        } else {
            return Err(StoreError::ValidationFailed(
                "Partial batch duplicate detected".to_string(),
            ));
        }
    }

    let batch_size = ops.len();
    let start_revision = current_revision + 1;
    let end_revision = current_revision + batch_size as i64;
    let mut op_ids = Vec::with_capacity(batch_size);
    let mut last_timestamp = 0i64;

    for (idx, envelope) in ops.into_iter().enumerate() {
        let new_revision = current_revision + 1 + idx as i64;
        let payload = encode_event_envelope(&envelope)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        tx.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![envelope.op_id, new_revision, payload, envelope.timestamp.to_string()],
        ).map_err(StoreError::Sqlite)?;

        op_ids.push(envelope.op_id);
        last_timestamp = envelope.timestamp;
    }

    tx.commit().map_err(StoreError::Sqlite)?;

    Ok(BatchAppendResult {
        start_revision,
        end_revision,
        count: batch_size,
        op_ids,
        last_timestamp,
    })
}

pub fn verify_batch_atomicity(result: &BatchAppendResult) -> Result<(), StoreError> {
    if result.count == 0 {
        if result.end_revision != result.start_revision {
            return Err(StoreError::ValidationFailed(
                "end_revision must equal start_revision for empty count".to_string(),
            ));
        }
    } else {
        if result.start_revision < 1 {
            return Err(StoreError::ValidationFailed(
                "start_revision must be at least 1".to_string(),
            ));
        }
        if result.end_revision < result.start_revision {
            return Err(StoreError::ValidationFailed(
                "end_revision must be >= start_revision".to_string(),
            ));
        }
        let expected_count = (result.end_revision - result.start_revision + 1) as usize;
        if result.count != expected_count {
            return Err(StoreError::ValidationFailed(format!(
                "count {} does not match revision range",
                result.count
            )));
        }
    }

    if result.count > 0 && result.op_ids.len() != result.count {
        return Err(StoreError::ValidationFailed(
            "op_ids length must match count".to_string(),
        ));
    }

    for (idx, op_id) in result.op_ids.iter().enumerate() {
        if op_id.is_empty() {
            return Err(StoreError::ValidationFailed(format!(
                "op_id at index {} must not be empty",
                idx
            )));
        }
    }
    if result.last_timestamp <= 0 {
        return Err(StoreError::ValidationFailed(
            "last_timestamp must be positive".to_string(),
        ));
    }
    Ok(())
}
