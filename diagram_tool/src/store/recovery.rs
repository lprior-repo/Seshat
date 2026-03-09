use rusqlite::Connection;
use std::path::Path;

use crate::store::error::RecoveryError;
use crate::store::session::{IntegrityStatus, RecoveryHandle, RecoverySession};

struct RawIntegrityStats {
    integrity_result: String,
    page_count: u32,
    free_pages: u32,
    schema_version: Option<i32>,
    event_count: u64,
    latest_revision: Option<i64>,
}

pub fn startup_integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError> {
    if !db_path.exists() {
        return Ok(missing_database_status());
    }

    let conn = Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(RecoveryError::Sqlite)?;

    let raw_stats = fetch_raw_integrity_stats(&conn)?;
    Ok(calculate_integrity_status(raw_stats))
}

fn missing_database_status() -> IntegrityStatus {
    IntegrityStatus {
        is_valid: false,
        page_count: 0,
        free_pages: 0,
        corrupted_pages: 0,
        schema_version: None,
        event_count: 0,
        latest_revision: None,
        error_message: Some("Database file does not exist".to_string()),
    }
}

fn fetch_raw_integrity_stats(conn: &Connection) -> Result<RawIntegrityStats, RecoveryError> {
    Ok(RawIntegrityStats {
        integrity_result: conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(RecoveryError::Sqlite)?,
        page_count: conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))
            .map_err(RecoveryError::Sqlite)?,
        free_pages: conn
            .query_row("PRAGMA freelist_count", [], |row| row.get(0))
            .map_err(RecoveryError::Sqlite)?,
        schema_version: conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .ok(),
        event_count: conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap_or(0),
        latest_revision: conn
            .query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |r| {
                r.get(0)
            })
            .ok()
            .filter(|&rev| rev > 0),
    })
}

fn calculate_integrity_status(stats: RawIntegrityStats) -> IntegrityStatus {
    let is_valid = stats.integrity_result == "ok";
    let corrupted_pages = u32::from(!is_valid && stats.integrity_result.contains("corrupt"));
    let error_message = match (is_valid, corrupted_pages) {
        (false, 0) => Some(stats.integrity_result),
        (false, c) if c > 0 => Some(format!("{c} corrupted pages found")),
        _ => None,
    };

    IntegrityStatus {
        is_valid,
        page_count: stats.page_count,
        free_pages: stats.free_pages,
        corrupted_pages,
        schema_version: stats.schema_version,
        event_count: stats.event_count,
        latest_revision: stats.latest_revision,
        error_message,
    }
}

pub fn open_recovery_mode(db_path: &Path) -> Result<RecoveryHandle, RecoveryError> {
    let conn = Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(RecoveryError::Sqlite)?;

    let _: i32 = conn
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .map_err(|e| RecoveryError::CorruptDatabase(e.to_string()))?;

    Ok(RecoveryHandle {
        conn,
        db_path: db_path.to_path_buf(),
        export_path: None,
    })
}

pub fn integrity_check(db_path: &Path) -> Result<IntegrityStatus, RecoveryError> {
    startup_integrity_check(db_path)
}

pub fn open_recovery_only(db_path: &Path) -> Result<RecoverySession, RecoveryError> {
    open_recovery_mode(db_path)
}

impl RecoveryHandle {
    pub fn export_to_json(&mut self, output_path: &Path) -> Result<(), RecoveryError> {
        let mut stmt = self.conn.prepare("SELECT id, operation_id, revision, payload, timestamp FROM events ORDER BY revision").map_err(RecoveryError::Sqlite)?;
        let events: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let operation_id: String = row.get(1)?;
                let revision: i64 = row.get(2)?;
                let payload: String = row.get(3)?;
                let timestamp: String = row.get(4)?;
                Ok(serde_json::json!({
                    "id": id,
                    "operation_id": operation_id,
                    "revision": revision,
                    "payload": payload,
                    "timestamp": timestamp
                }))
            })
            .map_err(RecoveryError::Sqlite)?
            .filter_map(Result::ok)
            .collect();
        let json_content = serde_json::to_string_pretty(&events)
            .map_err(|e| RecoveryError::Io(std::io::Error::other(e.to_string())))?;
        std::fs::write(output_path, json_content).map_err(RecoveryError::Io)?;
        self.export_path = Some(output_path.to_path_buf());
        Ok(())
    }
}
