//! Sync operations
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::models::envelope::parse_event_envelope;
use crate::models::projection::EventRecord;

use super::{ApplySummary, SyncError, SyncMessage, WatcherHandle};

#[cfg(not(target_arch = "wasm32"))]
pub fn start_store_watcher(path: PathBuf) -> Result<WatcherHandle, SyncError> {
    if !path.exists() { return Err(SyncError::Io(format!("database file does not exist: {}", path.display()))); }
    let active = Arc::new(AtomicBool::new(true));
    let active_clone = active.clone();
    let config = Config::default().with_poll_interval(Duration::from_millis(100)).with_compare_contents(false);
    let watcher = RecommendedWatcher::new(move |res: Result<Event, notify::Error>| {
        if !active_clone.load(Ordering::SeqCst) { return; }
        if let Ok(event) = res { if matches!(event.kind, EventKind::Modify(_)) { let _is_db_change = event.paths.iter().any(|p| p.to_string_lossy().ends_with(".db") || p.to_string_lossy().ends_with("-wal")); } }
    }, config).map_err(|_| SyncError::WatchInit)?;
    let mut watcher = watcher;
    let watch_path = path.parent().ok_or_else(|| SyncError::Io("cannot determine parent directory".to_string()))?.to_path_buf();
    watcher.watch(&watch_path, RecursiveMode::NonRecursive).map_err(|_| SyncError::WatchInit)?;
    Ok(WatcherHandle { watcher, active, watch_path })
}

#[cfg(target_arch = "wasm32")]
pub fn start_store_watcher(_path: PathBuf) -> Result<WatcherHandle, SyncError> { Ok(WatcherHandle { active: Arc::new(AtomicBool::new(false)) }) }

#[cfg(not(target_arch = "wasm32"))]
pub fn stop_store_watcher(mut handle: WatcherHandle) -> Result<(), SyncError> { handle.active.store(false, Ordering::SeqCst); handle.watcher.unwatch(&handle.watch_path).map_err(|_| SyncError::WatchRuntime); Ok(()) }
#[cfg(target_arch = "wasm32")]
pub fn stop_store_watcher(handle: WatcherHandle) -> Result<(), SyncError> { handle.active.store(false, Ordering::SeqCst); Ok(()) }

#[cfg(not(target_arch = "wasm32"))]
pub fn start_event_tail_watcher(db_path: PathBuf, tx: Sender<SyncMessage>) -> Result<WatcherHandle, SyncError> {
    if !db_path.exists() { return Err(SyncError::Io(format!("database file does not exist: {}", db_path.display()))); }
    let active = Arc::new(AtomicBool::new(true));
    let active_clone = active.clone();
    let config = Config::default().with_poll_interval(Duration::from_millis(100)).with_compare_contents(false);
    let tx_clone = tx.clone();
    let tx_clone_for_timer = tx.clone();
    let active_for_timer = active.clone();
    std::thread::spawn(move || { while active_for_timer.load(Ordering::SeqCst) { std::thread::sleep(Duration::from_secs(5)); if active_for_timer.load(Ordering::SeqCst) { let _ = tx_clone_for_timer.send(SyncMessage::EventsUpdated(vec![])); } } });
    let mut watcher = RecommendedWatcher::new(move |res: Result<Event, notify::Error>| {
        if !active_clone.load(Ordering::SeqCst) { return; }
        if let Ok(event) = res { if matches!(event.kind, EventKind::Modify(_)) { let is_db_change = event.paths.iter().any(|p| p.to_string_lossy().ends_with(".db") || p.to_string_lossy().ends_with("-wal")); if is_db_change { let _ = tx_clone.send(SyncMessage::EventsUpdated(vec![])); } } }
    }, config).map_err(|_| SyncError::WatchInit)?;
    let watch_path = db_path.parent().ok_or_else(|| SyncError::Io("cannot determine parent directory".to_string()))?.to_path_buf();
    watcher.watch(&watch_path, RecursiveMode::NonRecursive).map_err(|_| SyncError::WatchInit)?;
    Ok(WatcherHandle { watcher, active, watch_path })
}

#[cfg(target_arch = "wasm32")]
pub fn start_event_tail_watcher(_db_path: PathBuf, _tx: Sender<SyncMessage>) -> Result<WatcherHandle, SyncError> { Ok(WatcherHandle { active: Arc::new(AtomicBool::new(false)) }) }

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_new_events(pool: &sqlx::SqlitePool, after_revision: i64) -> Result<Vec<EventRecord>, SyncError> {
    let rows = sqlx::query_as::<sqlx::Sqlite, (String, i64, String, String)>("SELECT operation_id, revision, payload, timestamp FROM events WHERE revision > $1 ORDER BY revision ASC").bind(after_revision).fetch_all(pool).await.map_err(|e| SyncError::Sqlite(e.to_string()))?;
    let mut events = Vec::with_capacity(rows.len());
    let mut expected_revision = after_revision + 1;
    for (operation_id, revision, payload, timestamp) in rows {
        if revision != expected_revision { return Err(SyncError::Decode(format!("revision gap"))); }
        let envelope = parse_event_envelope(&payload).map_err(|e| SyncError::Decode(format!("envelope parse error")))?;
        let timestamp = timestamp.parse::<i64>().map_err(|e| SyncError::Decode(format!("timestamp parse error")))?;
        events.push(EventRecord { op_id: envelope.op_id, revision: revision as u64, operation: envelope.operation, author: envelope.author, timestamp });
        expected_revision += 1;
    }
    Ok(events)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_latest_revision(pool: &sqlx::SqlitePool) -> Result<i64, SyncError> { sqlx::query_scalar::<sqlx::Sqlite, i64>("SELECT COALESCE(MAX(revision), 0) FROM events").fetch_one(pool).await.map_err(|e| SyncError::Sqlite(e.to_string())) }

pub fn apply_tail_batch(projection: &mut crate::models::projection::DiagramProjection, events: Vec<EventRecord>) -> Result<ApplySummary, SyncError> {
    use crate::models::projection::{replay_events_from, ReplayError};
    if events.is_empty() { return Ok(ApplySummary { events_applied: 0, from_revision: projection.revision, to_revision: projection.revision, affected_entities: Vec::new() }); }
    let from_revision = projection.revision;
    let affected_entities: Vec<String> = events.iter().flat_map(|e| match &e.operation { crate::models::envelope::DomainOp::NodeAdd { id, .. } | crate::models::envelope::DomainOp::NodeMove { id, .. } | crate::models::envelope::DomainOp::NodeDelete { id } | crate::models::envelope::DomainOp::NodeRestore { id } => vec![format!("node:{}", id)], _ => vec![] }).collect();
    let updated_projection = replay_events_from(projection.clone(), &events).map_err(|e| SyncError::Decode(e.to_string()))?;
    let to_revision = updated_projection.revision;
    *projection = updated_projection;
    Ok(ApplySummary { events_applied: events.len(), from_revision, to_revision, affected_entities })
}

pub fn schedule_ui_update(summary: ApplySummary) -> Result<(), SyncError> { if summary.events_applied == 0 { Ok(()) } else { Ok(()) } }
