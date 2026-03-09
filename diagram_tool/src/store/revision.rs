use super::errors::StoreError;
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
    let current = current_revision(conn)?;
    Ok(current + 1)
}
