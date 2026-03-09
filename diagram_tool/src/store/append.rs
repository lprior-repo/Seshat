use super::errors::StoreError;
use super::types::{AppendResult, OpId, StoreRevision, Timestamp};
use crate::models::envelope::{encode_event_envelope, EventEnvelope};
use rusqlite::{Connection, Transaction};

pub fn append_event(
    conn: &mut Connection,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendResult, StoreError> {
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

    let new_revision = current_revision + 1;
    let payload =
        encode_event_envelope(&envelope).map_err(|e| StoreError::Serialization(e.to_string()))?;

    tx.execute(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            envelope.op_id,
            new_revision,
            payload,
            envelope.timestamp.to_string()
        ],
    )
    .map_err(StoreError::Sqlite)?;

    tx.commit().map_err(StoreError::Sqlite)?;

    Ok(AppendResult {
        revision: StoreRevision::new(new_revision)?,
        op_id: OpId::new(envelope.op_id)?,
        timestamp: Timestamp::new(envelope.timestamp)?,
    })
}

pub fn append_with_occ(
    conn: &mut Connection,
    op: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendResult, StoreError> {
    append_event(conn, op, expected_revision)
}

pub fn verify_occ_append(_result: &AppendResult) -> Result<(), StoreError> {
    Ok(())
}

pub fn with_write_tx<T, F>(conn: &mut Connection, f: F) -> Result<T, StoreError>
where
    F: FnOnce(&Transaction) -> Result<T, StoreError>,
{
    let tx = conn.transaction().map_err(StoreError::Sqlite)?;
    let result = f(&tx);

    match result {
        Ok(value) => {
            tx.commit().map_err(StoreError::Sqlite)?;
            Ok(value)
        }
        Err(err) => Err(StoreError::TransactionAborted(err.to_string())),
    }
}
