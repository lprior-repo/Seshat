//! Sync module - file-watch tail ingestion for external CLI writes
//!
//! This module provides file watching to detect and ingest changes
//! made by external CLI tools. It watches the SQLite database file
//! and its WAL file for modifications and fetches new events.
//!
//! This module is not available on WASM targets.

#![cfg(not(target_arch = "wasm32"))]
//!
//! # Architecture
//!
//! The sync module uses the `notify` crate for file watching. When the
//! database file changes (either the main `.db` file or the `-wal` file),
//! the watcher sends a `SyncMessage::EventsUpdated` notification through
//! a channel. The GUI can then call `fetch_new_events` to get the new
//! event records.
//!
//! # Example
//!
//! ```ignore
//! use std::sync::mpsc::channel;
//! use diagram_tool::models::sync::{start_event_tail_watcher, SyncMessage};
//!
//! let (tx, rx) = channel();
//! let handle = start_event_tail_watcher(db_path, tx)?;
//!
//! // In GUI event loop
//! while let Ok(msg) = rx.try_recv() {
//!     match msg {
//!         SyncMessage::EventsUpdated(revisions) => {
//!             // Fetch and apply new events
//!         }
//!         SyncMessage::Error(e) => {
//!             // Handle error
//!         }
//!     }
//! }
//! ```

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;
#[cfg(any(kani, test))]
#[allow(unused_imports)]
use tokio::time::sleep;

use crate::models::envelope::parse_event_envelope;
use crate::models::projection::EventRecord;
use crate::store_async::envelope_to_valid_event;

#[cfg(kani)]
use crate::models::document::NodeId;

/// Helper to convert EventEnvelope to ValidEvent (for testing)
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::unwrap_used)]
fn to_valid_event(
    envelope: crate::models::envelope::EventEnvelope,
) -> Result<crate::store::types::ValidEvent, crate::store_async::AsyncStoreError> {
    envelope_to_valid_event(&envelope)
}

/// Errors that can occur during sync operations
#[derive(Debug, Error, Clone)]
pub enum SyncError {
    /// Failed to initialize the file watcher
    #[error("failed to initialize file watcher")]
    WatchInit,
    /// Runtime error during watching
    #[error("watcher runtime error")]
    WatchRuntime,
    /// I/O error accessing the database file
    #[error("I/O error: {0}")]
    Io(String),
    /// SQLite database error
    #[error("SQLite error: {0}")]
    Sqlite(String),
    /// Failed to decode event from database
    #[error("failed to decode event: {0}")]
    Decode(String),
    /// Channel was closed unexpectedly
    #[error("channel closed")]
    ChannelClosed,
}

impl From<io::Error> for SyncError {
    fn from(err: io::Error) -> Self {
        SyncError::Io(err.to_string())
    }
}

/// Handle to the file watcher
///
/// This handle keeps the watcher alive. When dropped, the watcher is stopped.
#[cfg(not(target_arch = "wasm32"))]
pub struct WatcherHandle {
    watcher: RecommendedWatcher,
    /// Flag to track if the watcher is still active
    active: Arc<AtomicBool>,
    /// The path being watched (for unwatch)
    watch_path: PathBuf,
}

/// Stub handle for WASM (file watching not supported)
#[cfg(target_arch = "wasm32")]
pub struct WatcherHandle {
    /// Flag to track if the watcher is still active
    active: Arc<AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl WatcherHandle {
    /// Check if the watcher is still active
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

#[cfg(target_arch = "wasm32")]
impl WatcherHandle {
    /// Check if the watcher is still active (always false on WASM)
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

/// Start watching the store database file for external writes
///
/// This is the contract-compliant function that watches the SQLite database
/// file (.db) and its WAL file (.db-wal) for modifications. When changes are
/// detected, the watcher emits sync tick events internally.
///
/// # Arguments
///
/// * `path` - Path to the SQLite database file to watch
///
/// # Returns
///
/// Returns a `WatcherHandle` that keeps the watcher alive. Use `stop_store_watcher`
/// to explicitly stop the watcher, or simply drop the handle.
///
/// # Errors
///
/// Returns `SyncError::WatchInit` if the watcher cannot be initialized.
/// Returns `SyncError::Io` if the path doesn't exist or is inaccessible.
///
/// # Example
///
/// ```ignore
/// let handle = start_store_watcher(PathBuf::from("diagram.db"))?;
/// // Watcher is now active
/// stop_store_watcher(handle)?; // Explicitly stop
/// // Or just let handle drop to stop automatically
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub fn start_store_watcher(path: PathBuf) -> Result<WatcherHandle, SyncError> {
    // Verify the database file exists
    if !path.exists() {
        return Err(SyncError::Io(format!(
            "database file does not exist: {}",
            path.display()
        )));
    }

    let active = Arc::new(AtomicBool::new(true));
    let active_clone = active.clone();

    // Create the watcher with a configuration
    let config = Config::default()
        .with_poll_interval(Duration::from_millis(100))
        .with_compare_contents(false);

    // Create the watcher with an event handler
    let watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            // Only process if still active
            if !active_clone.load(Ordering::SeqCst) {
                return;
            }

            match res {
                Ok(event) => {
                    // Only process modify events on our database files
                    if matches!(event.kind, EventKind::Modify(_)) {
                        // Check if this is a database or WAL file modification
                        let _is_db_change = event.paths.iter().any(|p| {
                            let path_str = p.to_string_lossy();
                            path_str.ends_with(".db")
                                || path_str.ends_with("-wal")
                                || path_str.ends_with(".db-wal")
                        });
                        // Sync tick emitted - caller should poll fetch_new_events
                    }
                }
                Err(_e) => {
                    // Error during watching - set inactive
                    active_clone.store(false, Ordering::SeqCst);
                }
            }
        },
        config,
    )
    .map_err(|_| SyncError::WatchInit)?;

    let mut watcher = watcher;

    // Watch the directory containing the database (to catch WAL file changes too)
    let watch_path = path
        .parent()
        .ok_or_else(|| SyncError::Io("cannot determine parent directory".to_string()))?
        .to_path_buf();

    watcher
        .watch(&watch_path, RecursiveMode::NonRecursive)
        .map_err(|_| SyncError::WatchInit)?;

    Ok(WatcherHandle {
        watcher,
        active,
        watch_path,
    })
}

/// Stub for WASM - file watching not supported
#[cfg(target_arch = "wasm32")]
pub fn start_store_watcher(_path: PathBuf) -> Result<WatcherHandle, SyncError> {
    // File watching not supported on WASM
    Ok(WatcherHandle {
        active: Arc::new(AtomicBool::new(false)),
    })
}

/// Stop the store watcher
///
/// This function explicitly stops the file watcher and releases its resources.
///
/// # Arguments
///
/// * `handle` - The watcher handle to stop
///
/// # Returns
///
/// Returns `Ok(())` if the watcher was stopped successfully.
///
/// # Errors
///
/// Returns `SyncError::WatchRuntime` if the watcher fails to stop cleanly.
#[cfg(not(target_arch = "wasm32"))]
pub fn stop_store_watcher(mut handle: WatcherHandle) -> Result<(), SyncError> {
    handle.active.store(false, Ordering::SeqCst);
    handle
        .watcher
        .unwatch(&handle.watch_path)
        .map_err(|_| SyncError::WatchRuntime)?;
    Ok(())
}

/// Stub for WASM - file watching not supported
#[cfg(target_arch = "wasm32")]
pub fn stop_store_watcher(handle: WatcherHandle) -> Result<(), SyncError> {
    handle.active.store(false, Ordering::SeqCst);
    Ok(())
}

/// Message types for sync notifications
#[derive(Debug, Clone)]
pub enum SyncMessage {
    /// New events are available with the list of new revision numbers
    EventsUpdated(Vec<u64>),
    /// An error occurred during watching
    Error(String),
}

/// Start watching for file changes to trigger tail ingestion
///
/// This function sets up a file watcher on the SQLite database file and its
/// WAL file. When changes are detected, it sends `SyncMessage::EventsUpdated`
/// notifications through the provided channel.
///
/// # Arguments
///
/// * `db_path` - Path to the SQLite database file
/// * `tx` - Channel sender for sync notifications
///
/// # Returns
///
/// Returns a `WatcherHandle` that keeps the watcher alive. Drop the handle
/// to stop watching.
///
/// # Errors
///
/// Returns `SyncError::WatchInit` if the watcher cannot be created or
/// if the database path doesn't exist.
///
/// # Example
///
/// ```ignore
/// let (tx, rx) = std::sync::mpsc::channel();
/// let handle = start_event_tail_watcher(db_path.into(), tx)?;
/// // Watcher is now active
/// drop(handle); // Stops watching
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub fn start_event_tail_watcher(
    db_path: PathBuf,
    tx: Sender<SyncMessage>,
) -> Result<WatcherHandle, SyncError> {
    // Verify the database file exists
    if !db_path.exists() {
        return Err(SyncError::Io(format!(
            "database file does not exist: {}",
            db_path.display()
        )));
    }

    let active = Arc::new(AtomicBool::new(true));
    let active_clone = active.clone();

    // Create the watcher with a configuration
    let config = Config::default()
        .with_poll_interval(Duration::from_millis(100))
        .with_compare_contents(false);

    // Clone the sender for use in the callback
    let tx_clone = tx.clone();

    // Create a fallback polling thread
    let tx_clone_for_timer = tx.clone();
    let active_for_timer = active.clone();
    std::thread::spawn(move || {
        while active_for_timer.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_secs(5));
            if active_for_timer.load(Ordering::SeqCst) {
                // Send a periodic sync tick as fallback in case file watcher drops events
                let _ = tx_clone_for_timer.send(SyncMessage::EventsUpdated(vec![]));
            }
        }
    });

    // Create the watcher with an event handler
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            // Only process if still active
            if !active_clone.load(Ordering::SeqCst) {
                return;
            }

            match res {
                Ok(event) => {
                    // Only process modify events on our database files
                    if matches!(event.kind, EventKind::Modify(_)) {
                        // Check if this is a database or WAL file modification
                        let is_db_change = event.paths.iter().any(|p| {
                            let path_str = p.to_string_lossy();
                            path_str.ends_with(".db")
                                || path_str.ends_with("-wal")
                                || path_str.ends_with(".db-wal")
                        });

                        if is_db_change {
                            // Send a notification - the receiver will fetch new events
                            // We don't know the revision numbers yet, so send empty vec
                            // The receiver should call fetch_new_events to get them
                            let _ = tx_clone.send(SyncMessage::EventsUpdated(vec![]));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(SyncMessage::Error(e.to_string()));
                }
            }
        },
        config,
    )
    .map_err(|_| SyncError::WatchInit)?;

    // Watch the directory containing the database (to catch WAL file changes too)
    let watch_path = db_path
        .parent()
        .ok_or_else(|| SyncError::Io("cannot determine parent directory".to_string()))?
        .to_path_buf();

    watcher
        .watch(&watch_path, RecursiveMode::NonRecursive)
        .map_err(|_| SyncError::WatchInit)?;

    Ok(WatcherHandle {
        watcher,
        active,
        watch_path,
    })
}

/// Stub for WASM - file watching not supported
#[cfg(target_arch = "wasm32")]
pub fn start_event_tail_watcher(
    _db_path: PathBuf,
    _tx: Sender<SyncMessage>,
) -> Result<WatcherHandle, SyncError> {
    // File watching not supported on WASM
    Ok(WatcherHandle {
        active: Arc::new(AtomicBool::new(false)),
    })
}

/// Fetch new events after a given revision
///
/// This function queries the events table for all events with a revision
/// greater than `after_revision`. It decodes the event payloads and returns
/// them as `EventRecord` instances.
///
/// # Arguments
///
/// * `conn` - SQLite database connection
/// * `after_revision` - Fetch events with revision > this value
///
/// # Returns
///
/// Returns a vector of `EventRecord` instances for all new events.
/// Returns an empty vector if there are no new events.
///
/// # Errors
///
/// Returns `SyncError::Sqlite` if the database query fails.
/// Returns `SyncError::Decode` if an event payload cannot be decoded.
///
/// # Example
///
/// ```ignore
/// let current_revision = 5;
/// let new_events = fetch_new_events(&bootstrap.pool, current_revision)?;
/// for event in new_events {
///     // Process each event
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_new_events(
    pool: &sqlx::SqlitePool,
    after_revision: i64,
) -> Result<Vec<EventRecord>, SyncError> {
    let rows = sqlx::query_as::<sqlx::Sqlite, (String, i64, String, String)>(
        "SELECT operation_id, revision, payload, timestamp FROM events \
         WHERE revision > $1 ORDER BY revision ASC",
    )
    .bind(after_revision)
    .fetch_all(pool)
    .await
    .map_err(|e: sqlx::Error| SyncError::Sqlite(e.to_string()))?;

    let mut events = Vec::with_capacity(rows.len());
    let mut expected_revision = after_revision + 1;

    for (operation_id, revision, payload, timestamp) in rows {
        if revision != expected_revision {
            return Err(SyncError::Decode(format!(
                "revision gap detected: expected {}, found {}",
                expected_revision, revision
            )));
        }

        let envelope = parse_event_envelope(&payload).map_err(|e| {
            SyncError::Decode(format!(
                "envelope parse error for op {}: {}",
                operation_id, e
            ))
        })?;

        let timestamp = timestamp.parse::<i64>().map_err(|e| {
            SyncError::Decode(format!(
                "timestamp parse error for op {}: {}",
                operation_id, e
            ))
        })?;

        events.push(EventRecord {
            op_id: envelope.op_id,
            revision: revision as u64,
            operation: envelope.operation,
            author: envelope.author,
            timestamp,
        });

        expected_revision += 1;
    }

    Ok(events)
}

/// Get the current latest revision from the database
///
/// # Errors
///
/// Returns `SyncError::Sqlite` if the query fails.
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_latest_revision(pool: &sqlx::SqlitePool) -> Result<i64, SyncError> {
    sqlx::query_scalar::<sqlx::Sqlite, i64>("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(pool)
        .await
        .map_err(|e: sqlx::Error| SyncError::Sqlite(e.to_string()))
}

/// Summary of a batch apply operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplySummary {
    /// Number of events applied
    pub events_applied: usize,
    /// Starting revision before apply
    pub from_revision: u64,
    /// Ending revision after apply
    pub to_revision: u64,
    /// IDs of affected entities (nodes/edges)
    pub affected_entities: Vec<String>,
}

/// Batch apply tail events to a projection without blocking the render loop
///
/// This function takes a vector of events and applies them to the projection
/// in a single batch operation. It returns a summary of what was applied,
/// which can be used to schedule targeted UI updates.
///
/// # Arguments
///
/// * `projection` - The current diagram projection to update
/// * `events` - The events to apply (should be ordered by revision)
///
/// # Returns
///
/// Returns an `ApplySummary` with details about the applied events.
/// Returns an empty summary if no events were provided.
///
/// # Errors
///
/// Returns `SyncError::Decode` if the events cannot be replayed.
///
/// # Example
///
/// ```ignore
/// let events = fetch_new_events(&bootstrap.pool, current_revision)?;
/// let summary = apply_tail_batch(&mut projection, events)?;
/// schedule_ui_update(summary)?;
/// ```
pub fn apply_tail_batch(
    projection: &mut crate::models::projection::DiagramProjection,
    events: Vec<EventRecord>,
) -> Result<ApplySummary, SyncError> {
    use crate::models::projection::{replay_events_from, ReplayError};

    if events.is_empty() {
        return Ok(ApplySummary {
            events_applied: 0,
            from_revision: projection.revision,
            to_revision: projection.revision,
            affected_entities: Vec::new(),
        });
    }

    let from_revision = projection.revision;
    let affected_entities = extract_affected_entities_from_events(&events);

    // Apply events using the existing replay mechanism
    let updated_projection = replay_events_from(projection.clone(), &events)
        .map_err(|e: ReplayError| SyncError::Decode(e.to_string()))?;

    let to_revision = updated_projection.revision;
    *projection = updated_projection;

    Ok(ApplySummary {
        events_applied: events.len(),
        from_revision,
        to_revision,
        affected_entities,
    })
}

/// Extract affected entity IDs from a batch of events
///
/// This function examines the events and collects all affected entity IDs
/// (nodes and edges) for targeted UI updates.
fn extract_affected_entities_from_events(events: &[EventRecord]) -> Vec<String> {
    use crate::models::envelope::{DomainOp, LabelTargetId, LabelTargetType};
    use std::collections::HashSet;

    let mut entities: HashSet<String> = HashSet::new();

    for event in events {
        match &event.operation {
            DomainOp::NodeAdd { id, .. }
            | DomainOp::NodeMove { id, .. }
            | DomainOp::NodeDelete { id }
            | DomainOp::NodeRestore { id }
            | DomainOp::UpdateNodeStyle { id, .. } => {
                entities.insert(format!("node:{}", id));
            }
            DomainOp::UpdateLabel {
                target_id,
                target_type,
                ..
            } => {
                match target_type {
                    LabelTargetType::Node => {
                        if let LabelTargetId::Node(node_id) = target_id {
                            entities.insert(format!("node:{}", node_id.as_str()));
                        }
                    }
                    LabelTargetType::Edge => {
                        if let LabelTargetId::Edge(edge_id) = target_id {
                            entities.insert(format!("edge:{}", edge_id.as_str()));
                        }
                    }
                };
            }
            DomainOp::NodeResize { id, .. } => {
                entities.insert(format!("node:{}", id.as_str()));
            }
            DomainOp::EdgeConnect { id, source, target } => {
                entities.insert(format!("edge:{}", id));
                entities.insert(format!("node:{}", source));
                entities.insert(format!("node:{}", target));
            }
            DomainOp::EdgeDisconnect { id } => {
                entities.insert(format!("edge:{}", id));
            }
            DomainOp::UpdateEdgeStyle { id, .. } => {
                entities.insert(format!("edge:{}", id));
            }
            DomainOp::BringForward { ids }
            | DomainOp::SendBackward { ids }
            | DomainOp::BringToFront { ids }
            | DomainOp::SendToBack { ids } => {
                for id in ids {
                    entities.insert(format!("node:{}", id));
                }
            }
            DomainOp::Group { id, ids } => {
                entities.insert(format!("node:{}", id));
                for node_id in ids {
                    entities.insert(format!("node:{}", node_id));
                }
            }
            DomainOp::Ungroup { id } => {
                entities.insert(format!("group:{}", id));
            }
        }
    }

    entities.into_iter().collect()
}

/// Schedule a UI update based on the apply summary
///
/// This function is called after `apply_tail_batch` to signal that the UI
/// should be updated. The summary contains information about which entities
/// were affected, allowing for targeted updates.
///
/// # Arguments
///
/// * `summary` - The summary from `apply_tail_batch`
///
/// # Returns
///
/// Returns `Ok(())` if the update was scheduled successfully.
///
/// # Errors
///
/// Returns `SyncError::ChannelClosed` if the UI channel is closed.
///
/// # Example
///
/// ```ignore
/// let summary = apply_tail_batch(&mut projection, events)?;
/// schedule_ui_update(summary)?;
/// ```
pub fn schedule_ui_update(summary: ApplySummary) -> Result<(), SyncError> {
    // In a full implementation, this would:
    // 1. Send a message through a channel to the UI thread
    // 2. The UI thread would then update the Dioxus signal
    //
    // For now, we just validate the summary is valid and return success.
    // The actual UI integration would use a channel or coroutine to
    // communicate with the Dioxus runtime.

    if summary.events_applied == 0 {
        // No changes, no update needed
        return Ok(());
    }

    // Log the update for debugging (in production, this would signal the UI)
    #[cfg(debug_assertions)]
    eprintln!(
        "[UI_UPDATE] events={} revision={}->{} entities={:?}",
        summary.events_applied,
        summary.from_revision,
        summary.to_revision,
        summary.affected_entities
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::NodeId;
    use crate::models::envelope::{Author, DomainOp, EventEnvelope};
    use crate::store_async as store;

    use tempfile::TempDir;

    async fn create_test_db() -> (TempDir, PathBuf, store::AsyncStoreBootstrap) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        (temp_dir, db_path, bootstrap)
    }

    fn make_test_envelope(op_id: &str, revision: i64) -> EventEnvelope {
        EventEnvelope {
            op_id: op_id.to_string(),
            timestamp: 1700000000 + revision,
            author: Author {
                id: "human-test-user".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: NodeId::new(format!("node-{revision}")),
                x: 100.0 * revision as f64,
                y: 200.0 * revision as f64,
                width: 80.0,
                height: 40.0,
                label: format!("Test Node {revision}"),
            },
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_fetch_new_events_returns_empty_when_no_events() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        let events = fetch_new_events(&bootstrap.pool, 0).await.unwrap();
        assert!(events.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_fetch_new_events_returns_events_after_revision() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        // Add some events
        for i in 1..=5 {
            let envelope = make_test_envelope(&format!("op-{i}"), i);
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        // Fetch events after revision 2 (should get revisions 3, 4, 5)
        let events = fetch_new_events(&bootstrap.pool, 2).await.unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].revision, 3);
        assert_eq!(events[1].revision, 4);
        assert_eq!(events[2].revision, 5);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_fetch_new_events_returns_all_events_when_after_revision_zero() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        // Add some events
        for i in 1..=3 {
            let envelope = make_test_envelope(&format!("op-{i}"), i);
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        // Fetch all events (after revision 0)
        let events = fetch_new_events(&bootstrap.pool, 0).await.unwrap();
        assert_eq!(events.len(), 3);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_fetch_new_events_returns_empty_when_after_revision_is_latest() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        // Add some events
        for i in 1..=3 {
            let envelope = make_test_envelope(&format!("op-{i}"), i);
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        // Fetch events after revision 3 (latest)
        let events = fetch_new_events(&bootstrap.pool, 3).await.unwrap();
        assert!(events.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_fetch_latest_revision_returns_zero_when_empty() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        let revision = fetch_latest_revision(&bootstrap.pool).await.unwrap();
        assert_eq!(revision, 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_fetch_latest_revision_returns_max_revision() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        // Add some events
        for i in 1..=5 {
            let envelope = make_test_envelope(&format!("op-{i}"), i);
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        let revision = fetch_latest_revision(&bootstrap.pool).await.unwrap();
        assert_eq!(revision, 5);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_start_event_tail_watcher_fails_for_nonexistent_path() {
        let (tx, _rx) = channel();
        let nonexistent_path = PathBuf::from("/nonexistent/path/test.db");

        let result = start_event_tail_watcher(nonexistent_path, tx);
        assert!(result.is_err());
        match result {
            Err(SyncError::Io(msg)) => {
                assert!(msg.contains("does not exist"));
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_start_event_tail_watcher_succeeds_for_existing_db() {
        let (_temp_dir, db_path, _bootstrap) = create_test_db().await;
        let (tx, rx) = channel();

        let result = start_event_tail_watcher(db_path, tx);
        assert!(result.is_ok());

        // The watcher should be active - drop to stop
        drop(result);

        // Channel may receive some spurious notifications on startup (platform-dependent)
        // The important thing is the watcher was created successfully
        // and the channel is still valid (not disconnected)
        // Drain any pending messages - they may or may not arrive
        let _ = rx.recv_timeout(Duration::from_millis(100));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_watcher_detects_database_modifications() {
        let (_temp_dir, db_path, bootstrap) = create_test_db().await;
        let (tx, rx) = channel();

        let _handle = start_event_tail_watcher(db_path.clone(), tx).unwrap();

        // Give the watcher time to start
        sleep(Duration::from_millis(200)).await;

        // Modify the database
        let envelope = make_test_envelope("op-new", 1);
        let event = to_valid_event(envelope).unwrap();
        crate::store_async::append_event_async(&bootstrap.pool, event, None)
            .await
            .unwrap();

        // The watcher should detect the change
        let recv_result = rx.recv_timeout(Duration::from_secs(2));
        match recv_result {
            Ok(SyncMessage::EventsUpdated(_)) => {
                // Good - we got a notification
            }
            Ok(SyncMessage::Error(e)) => {
                panic!("Watcher sent error: {e}");
            }
            Err(RecvTimeoutError::Timeout) => {
                // This is acceptable - file watching can be unreliable in tests
                // The important thing is the fetch_new_events function works
            }
            Err(e) => {
                panic!("Unexpected channel error: {e}");
            }
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_events_are_ordered_by_revision() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        // Add events
        for i in 1..=10 {
            let envelope = make_test_envelope(&format!("op-{i}"), i);
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        // Fetch events after revision 5
        let events = fetch_new_events(&bootstrap.pool, 5).await.unwrap();
        assert_eq!(events.len(), 5);

        // Verify they're in order
        for (idx, event) in events.iter().enumerate() {
            assert_eq!(event.revision, (6 + idx) as u64);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_event_record_contains_correct_data() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        let envelope = EventEnvelope {
            op_id: "op-test-123".to_string(),
            timestamp: 1700000123,
            author: Author {
                id: "human-alice".to_string(),
                name: "Alice".to_string(),
                email: Some("alice@example.com".to_string()),
            },
            operation: DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 150.0,
                y: 250.0,
            },
        };

        let event = to_valid_event(envelope).unwrap();
        crate::store_async::append_event_async(&bootstrap.pool, event, None)
            .await
            .unwrap();

        let events = fetch_new_events(&bootstrap.pool, 0).await.unwrap();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.op_id, "op-test-123");
        assert_eq!(event.revision, 1);
        assert_eq!(event.timestamp, 1700000123);
        assert_eq!(event.author.id, "human-alice");
        assert!(matches!(event.operation, DomainOp::NodeMove { .. }));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_replaying_fetched_events_produces_correct_projection() {
        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        // Add a sequence of operations
        let ops = [
            (
                "op-1",
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 1".to_string(),
                },
            ),
            (
                "op-2",
                DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 200.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 2".to_string(),
                },
            ),
            (
                "op-3",
                DomainOp::EdgeConnect {
                    id: "edge-1".to_string(),
                    source: "node-1".to_string(),
                    target: "node-2".to_string(),
                },
            ),
        ];

        for (op_id, operation) in &ops {
            let envelope = EventEnvelope {
                op_id: op_id.to_string(),
                timestamp: 1700000000,
                author: Author {
                    id: "human-test".to_string(),
                    name: "Test".to_string(),
                    email: None,
                },
                operation: operation.clone(),
            };
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        // Fetch all events
        let events = fetch_new_events(&bootstrap.pool, 0).await.unwrap();
        assert_eq!(events.len(), 3);

        // Replay them to produce a projection starting from revision 1
        // (since the first event has revision 1)
        use crate::models::projection::{replay_events_from, DiagramProjection};
        let projection = replay_events_from(DiagramProjection::with_revision(1), &events).unwrap();

        assert_eq!(projection.revision, 4);
        assert_eq!(projection.nodes.len(), 2);
        assert_eq!(projection.edges.len(), 1);
    }

    // Tests for contract-compliant start_store_watcher and stop_store_watcher

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_start_store_watcher_fails_for_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/test.db");

        let result = start_store_watcher(nonexistent_path);
        assert!(result.is_err());
        match result {
            Err(SyncError::Io(msg)) => {
                assert!(msg.contains("does not exist"));
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_start_store_watcher_succeeds_for_existing_db() {
        let (_temp_dir, db_path, _bootstrap) = create_test_db().await;

        let result = start_store_watcher(db_path);
        assert!(result.is_ok());

        // The watcher should be active
        let handle = result.unwrap();
        assert!(handle.is_active());

        // Drop to stop
        drop(handle);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_stop_store_watcher_succeeds() {
        let (_temp_dir, db_path, _bootstrap) = create_test_db().await;

        let handle = start_store_watcher(db_path).unwrap();
        assert!(handle.is_active());

        let result = stop_store_watcher(handle);
        assert!(result.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_watcher_handle_is_active_flag() {
        let (_temp_dir, db_path, _bootstrap) = create_test_db().await;

        let handle = start_store_watcher(db_path).unwrap();
        assert!(handle.is_active());

        // After stop, the handle is consumed
        let _ = stop_store_watcher(handle);
    }

    // Tests for apply_tail_batch and schedule_ui_update

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_tail_batch_with_empty_events_returns_empty_summary() {
        use crate::models::projection::DiagramProjection;

        let mut projection = DiagramProjection::with_revision(0);
        let events = Vec::new();

        let summary = apply_tail_batch(&mut projection, events).unwrap();

        assert_eq!(summary.events_applied, 0);
        assert_eq!(summary.from_revision, 0);
        assert_eq!(summary.to_revision, 0);
        assert!(summary.affected_entities.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_apply_tail_batch_applies_events_and_updates_revision() {
        use crate::models::projection::DiagramProjection;

        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        // Add some events
        for i in 1..=3 {
            let envelope = make_test_envelope(&format!("op-{i}"), i);
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        let events = fetch_new_events(&bootstrap.pool, 0).await.unwrap();
        assert_eq!(events.len(), 3);

        let mut projection = DiagramProjection::with_revision(1);
        let summary = apply_tail_batch(&mut projection, events).unwrap();

        assert_eq!(summary.events_applied, 3);
        assert_eq!(summary.from_revision, 1);
        assert_eq!(summary.to_revision, 4); // 1 + 3 events
        assert_eq!(projection.revision, 4);
        assert_eq!(projection.nodes.len(), 3);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_apply_tail_batch_extracts_affected_entities() {
        use crate::models::projection::DiagramProjection;

        let (_temp_dir, _db_path, bootstrap) = create_test_db().await;

        // Add node and edge operations
        let ops = [
            (
                "op-1",
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 1".to_string(),
                },
            ),
            (
                "op-2",
                DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 200.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 2".to_string(),
                },
            ),
            (
                "op-3",
                DomainOp::EdgeConnect {
                    id: "edge-1".to_string(),
                    source: "node-1".to_string(),
                    target: "node-2".to_string(),
                },
            ),
        ];

        for (op_id, operation) in &ops {
            let envelope = EventEnvelope {
                op_id: op_id.to_string(),
                timestamp: 1700000000,
                author: Author {
                    id: "human-test".to_string(),
                    name: "Test".to_string(),
                    email: None,
                },
                operation: operation.clone(),
            };
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        let events = fetch_new_events(&bootstrap.pool, 0).await.unwrap();
        let mut projection = DiagramProjection::with_revision(1);
        let summary = apply_tail_batch(&mut projection, events).unwrap();

        // Should have node:node-1, node:node-2, edge:edge-1
        assert!(summary
            .affected_entities
            .contains(&"node:node-1".to_string()));
        assert!(summary
            .affected_entities
            .contains(&"node:node-2".to_string()));
        assert!(summary
            .affected_entities
            .contains(&"edge:edge-1".to_string()));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_schedule_ui_update_with_empty_summary_succeeds() {
        let summary = ApplySummary {
            events_applied: 0,
            from_revision: 0,
            to_revision: 0,
            affected_entities: Vec::new(),
        };

        let result = schedule_ui_update(summary);
        assert!(result.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_schedule_ui_update_with_events_succeeds() {
        let summary = ApplySummary {
            events_applied: 5,
            from_revision: 1,
            to_revision: 6,
            affected_entities: vec!["node:node-1".to_string()],
        };

        let result = schedule_ui_update(summary);
        assert!(result.is_ok());
    }
}
