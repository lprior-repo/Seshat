use crate::store::error::StoreError;
use crate::store::types::{EventRecord, ValidOperationId};
use rusqlite::Connection;

pub fn fetch_latest_revision(conn: &Connection) -> Result<i64, StoreError> {
    conn.query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |row| {
        row.get(0)
    })
    .map_err(StoreError::Sqlite)
}

pub fn current_revision(conn: &Connection) -> Result<i64, StoreError> {
    fetch_latest_revision(conn)
}

pub fn next_revision(conn: &Connection) -> Result<i64, StoreError> {
    current_revision(conn).map(|r| r + 1)
}

pub fn lookup_existing_op(
    conn: &Connection,
    op_id: &ValidOperationId,
) -> Result<Option<EventRecord>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT operation_id, revision, timestamp, payload FROM events WHERE operation_id = ?1",
        )
        .map_err(StoreError::Sqlite)?;
    let result = stmt
        .query_row([op_id.as_str()], |row| {
            let timestamp_str: String = row.get(2)?;
            let timestamp: i64 = timestamp_str.parse().unwrap_or_default();
            Ok(EventRecord {
                op_id: row.get(0)?,
                revision: row.get(1)?,
                timestamp,
                payload: row.get(3)?,
            })
        })
        .ok();
    Ok(result)
}
