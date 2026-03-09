use crate::models::envelope::{encode_event_envelope, EventEnvelope};
use crate::store::error::{CliError, StoreError};
use crate::store::read::{fetch_latest_revision, lookup_existing_op};
use crate::store::session::ReadWriteSession;
use crate::store::types::{
    AppendOutcome, BoundedBatch, EventRecord, Revision, ValidEvent, ValidOperationId, ValidPayload,
    ValidTimestamp,
};
use rusqlite::{Connection, Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    Exact,
    Conflict,
}

pub fn submit_cli_op(
    session: &mut ReadWriteSession,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendOutcome, CliError> {
    if envelope.op_id.is_empty() {
        return Err(CliError::InvalidInput("op_id is required".to_string()));
    }
    if envelope.author.id.is_empty() {
        return Err(CliError::InvalidInput("author.id is required".to_string()));
    }

    let op_id = ValidOperationId::new(envelope.op_id.clone()).map_err(CliError::StoreFailure)?;
    let timestamp =
        ValidTimestamp::new(envelope.timestamp as u64).map_err(CliError::StoreFailure)?;
    let payload_str =
        encode_event_envelope(&envelope).map_err(|e| CliError::Serialization(e.to_string()))?;
    let payload = ValidPayload::new(payload_str).map_err(CliError::StoreFailure)?;

    let event = ValidEvent {
        op_id,
        timestamp,
        payload,
    };
    let expected = expected_revision
        .map(Revision::new)
        .transpose()
        .map_err(CliError::StoreFailure)?;

    let rev = append_event(session, event, expected)?;
    Ok(AppendOutcome {
        revision: rev.get(),
        op_id: envelope.op_id,
        timestamp: envelope.timestamp,
    })
}

pub fn cli_submit_response(outcome: &AppendOutcome) -> String {
    serde_json::json!({
        "ok": true,
        "revision": outcome.revision,
        "op_id": outcome.op_id,
        "timestamp": outcome.timestamp
    })
    .to_string()
}

pub fn append_event(
    session: &mut ReadWriteSession,
    event: ValidEvent,
    expected_revision: Option<Revision>,
) -> Result<Revision, StoreError> {
    let current_revision = fetch_latest_revision(&session.conn)?;
    let (_, end_revision) = calculate_batch_revisions(current_revision, expected_revision, 1)?;

    with_write_tx(&mut session.conn, |tx| {
        insert_single_event(tx, end_revision, &event)
    })?;

    Revision::new(end_revision)
}

pub fn append_batch(
    session: &mut ReadWriteSession,
    ops: BoundedBatch<1, 1000>,
    expected_revision: Option<Revision>,
) -> Result<Revision, StoreError> {
    let current_revision = fetch_latest_revision(&session.conn)?;
    let events = ops.into_inner();
    let (_, end_revision) =
        calculate_batch_revisions(current_revision, expected_revision, events.len() as i64)?;

    with_write_tx(&mut session.conn, |tx| {
        insert_events_tx(tx, current_revision, &events)
    })?;

    Revision::new(end_revision)
}

fn calculate_batch_revisions(
    current_revision: i64,
    expected_revision: Option<Revision>,
    batch_size: i64,
) -> Result<(i64, i64), StoreError> {
    if let Some(expected) = expected_revision {
        if current_revision != expected.get() {
            return Err(StoreError::RevisionMismatch {
                expected: expected.get(),
                found: current_revision,
            });
        }
    }
    if current_revision > i64::MAX - batch_size {
        return Err(StoreError::RevisionOverflow);
    }
    Ok((current_revision + 1, current_revision + batch_size))
}

fn insert_events_tx(
    tx: &Transaction,
    current_revision: i64,
    events: &[ValidEvent],
) -> Result<(), StoreError> {
    events.iter().enumerate().try_for_each(|(idx, event)| {
        let new_revision = current_revision + 1 + idx as i64;
        insert_single_event(tx, new_revision, event)
    })
}

fn insert_single_event(
    tx: &Transaction,
    new_revision: i64,
    event: &ValidEvent,
) -> Result<(), StoreError> {
    tx.execute(
        "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            event.op_id.as_str(),
            new_revision,
            event.payload.as_str(),
            event.timestamp.get().to_string()
        ],
    )
    .map_err(|e| map_sqlite_err(e, event.op_id.as_str()))
    .map(|_| ())
}

fn map_sqlite_err(err: rusqlite::Error, op_id: &str) -> StoreError {
    let sqlite_err = StoreError::Sqlite(err);
    if sqlite_err
        .to_string()
        .contains("UNIQUE constraint failed: events.operation_id")
    {
        StoreError::DuplicateWithConflict(op_id.to_string())
    } else {
        sqlite_err
    }
}

pub fn classify_duplicate(
    existing: &EventRecord,
    incoming_payload: &ValidPayload,
) -> DuplicateKind {
    if existing.payload == incoming_payload.as_str() {
        DuplicateKind::Exact
    } else {
        DuplicateKind::Conflict
    }
}

pub fn append_idempotent(
    session: &mut ReadWriteSession,
    event: ValidEvent,
) -> Result<AppendOutcome, StoreError> {
    let existing = lookup_existing_op(&session.conn, &event.op_id)?;
    match existing {
        None => {
            let op_id = event.op_id.as_str().to_string();
            let timestamp = event.timestamp.get() as i64;
            let rev = append_event(session, event, None)?;
            Ok(AppendOutcome {
                revision: rev.get(),
                op_id,
                timestamp,
            })
        }
        Some(record) => {
            let kind = classify_duplicate(&record, &event.payload);
            match kind {
                DuplicateKind::Exact => Ok(AppendOutcome {
                    revision: record.revision,
                    op_id: record.op_id,
                    timestamp: record.timestamp,
                }),
                DuplicateKind::Conflict => Err(StoreError::DuplicateWithConflict(
                    event.op_id.as_str().to_string(),
                )),
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
