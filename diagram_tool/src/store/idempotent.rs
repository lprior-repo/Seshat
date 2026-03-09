use super::append::append_event;
use super::errors::StoreError;
use super::types::{AppendOutcome, DuplicateKind, EventRecord, OpId, StoreRevision, Timestamp};
use crate::models::envelope::{encode_event_envelope, EventEnvelope};
use rusqlite::{Connection, OptionalExtension};

pub fn classify_duplicate(
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

pub fn append_idempotent(
    conn: &mut Connection,
    op: EventEnvelope,
) -> Result<AppendOutcome, StoreError> {
    let existing = lookup_existing_op(conn, &op.op_id)?;
    match existing {
        None => {
            let result = append_event(conn, op, None)?;
            Ok(AppendOutcome::from(result))
        }
        Some(record) => {
            let kind = classify_duplicate(&record, &op)?;
            match kind {
                DuplicateKind::Exact => Ok(AppendOutcome {
                    revision: record.revision,
                    op_id: record.op_id,
                    timestamp: record.timestamp,
                }),
                DuplicateKind::Conflict => Err(StoreError::DuplicateWithConflict(op.op_id)),
            }
        }
    }
}

pub fn ensure_op_id_uniqueness(conn: &mut Connection) -> Result<(), StoreError> {
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_events_operation_id_unique ON events(operation_id)",
        [],
    )
    .map_err(StoreError::Sqlite)?;
    Ok(())
}

pub fn lookup_existing_op(
    conn: &Connection,
    op_id: &str,
) -> Result<Option<EventRecord>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT operation_id, revision, timestamp, payload FROM events WHERE operation_id = ?1",
        )
        .map_err(StoreError::Sqlite)?;

    let result = stmt
        .query_row([op_id], |row| {
            let timestamp_str: String = row.get(2)?;
            let timestamp: i64 = timestamp_str
                .parse()
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
            Ok(EventRecord {
                op_id: OpId::new(row.get(0)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                revision: StoreRevision::new(row.get(1)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                timestamp: Timestamp::new(timestamp).map_err(|_| rusqlite::Error::InvalidQuery)?,
                payload: row.get(3)?,
            })
        })
        .optional()
        .map_err(StoreError::Sqlite)?;
    Ok(result)
}
