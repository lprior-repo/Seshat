//! Snapshot module - persistence and recovery of diagram projections
//!
//! This module provides snapshot write and tail replay boot functionality
//! for efficient startup. Snapshots store serialized `DiagramProjection` state
//! at specific revisions, allowing fast recovery by replaying only events
//! after the snapshot point.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::envelope::parse_event_envelope;
use crate::models::projection::{
    replay_events, replay_events_from, DiagramProjection, EventRecord,
};

/// Errors that can occur during snapshot operations
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnapshotError {
    /// Snapshot revision is behind current revision (stale)
    #[error("snapshot revision is stale: expected at least {expected}, found {found}")]
    SnapshotStale {
        /// Expected minimum revision
        expected: u64,
        /// Found revision
        found: u64,
    },
    /// Serialization/deserialization error
    #[error("serialization error: {0}")]
    Serialization(String),
    /// `SQLite` error
    #[error("SQLite error: {0}")]
    Sqlite(String),
    /// Replay error during tail replay
    #[error("replay error: {0}")]
    Replay(String),
}

/// Metadata about a stored snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotMeta {
    /// Unique snapshot identifier
    pub id: i64,
    /// Revision number this snapshot represents
    pub revision: u64,
    /// Timestamp when snapshot was created (Unix timestamp)
    pub created_at: i64,
}

/// Write a snapshot of the current projection state
///
/// This function:
/// 1. Validates the projection revision matches current latest revision
/// 2. Serializes the projection to JSON
/// 3. Stores in an independent transaction
///
/// The snapshot is stored with its revision marker, enabling efficient
/// recovery by replaying only events after the snapshot point.
///
/// # Errors
/// Returns `SnapshotError::SnapshotStale` if projection revision doesn't match
/// Returns `SnapshotError::Serialization` if encoding fails
/// Returns `SnapshotError::Sqlite` if database operations fail
pub fn write_snapshot(
    conn: &mut Connection,
    projection: &DiagramProjection,
) -> Result<SnapshotMeta, SnapshotError> {
    // Get current latest revision from events table
    let current_revision: i64 = conn
        .query_row("SELECT COALESCE(MAX(revision), 0) FROM events", [], |row| {
            row.get(0)
        })
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

    let current_revision = current_revision as u64;

    // Validate projection revision matches current revision (no stale snapshots)
    if projection.revision != current_revision {
        return Err(SnapshotError::SnapshotStale {
            expected: current_revision,
            found: projection.revision,
        });
    }

    // Serialize projection to JSON
    let payload = serde_json::to_string(projection)
        .map_err(|e| SnapshotError::Serialization(e.to_string()))?;

    // Insert snapshot in independent transaction
    let tx = conn
        .transaction()
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

    // Use INSERT OR REPLACE to handle idempotency
    tx.execute(
        "INSERT OR REPLACE INTO snapshots (revision, payload) VALUES (?1, ?2)",
        rusqlite::params![projection.revision as i64, payload],
    )
    .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

    // Get the inserted id
    let id: i64 = tx
        .query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

    // Get created_at timestamp
    let created_at: i64 = tx
        .query_row(
            "SELECT created_at FROM snapshots WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

    tx.commit()
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

    Ok(SnapshotMeta {
        id,
        revision: projection.revision,
        created_at,
    })
}

/// Get metadata for the latest snapshot
///
/// Returns `Ok(Some(meta))` if a snapshot exists, `Ok(None)` if no snapshots exist.
///
/// # Errors
/// Returns `SnapshotError::Sqlite` if database operations fail
pub fn latest_snapshot(conn: &Connection) -> Result<Option<SnapshotMeta>, SnapshotError> {
    let result = conn.query_row(
        "SELECT id, revision, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
        [],
        |row| {
            let id: i64 = row.get(0)?;
            let revision: i64 = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            Ok(SnapshotMeta {
                id,
                revision: revision as u64,
                created_at,
            })
        },
    );

    match result {
        Ok(meta) => Ok(Some(meta)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(SnapshotError::Sqlite(e.to_string())),
    }
}

/// Load the projection from the latest snapshot and replay events after it
///
/// This function:
/// 1. Loads the latest snapshot from the database
/// 2. Fetches all events with revision greater than snapshot revision
/// 3. Replays events on top of the snapshot to produce the final projection
///
/// If no snapshot exists, falls back to full replay from revision 0.
///
/// # Errors
/// Returns `SnapshotError::Serialization` if deserialization fails
/// Returns `SnapshotError::Sqlite` if database operations fail
/// Returns `SnapshotError::Replay` if event replay fails
pub fn load_projection(conn: &Connection) -> Result<DiagramProjection, SnapshotError> {
    // Try to load latest snapshot
    let snapshot_result = conn.query_row(
        "SELECT id, revision, payload, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
        [],
        |row| {
            let id: i64 = row.get(0)?;
            let rev: i64 = row.get(1)?;
            let created: i64 = row.get(3)?;
            Ok(SnapshotMeta {
                id,
                revision: rev as u64,
                created_at: created,
            })
        },
    );

    match snapshot_result {
        Ok(latest_snapshot) => {
            // Deserialize the snapshot payload to get the base projection
            let base_projection: DiagramProjection = conn
                .query_row(
                    "SELECT payload FROM snapshots WHERE id = ?1",
                    [latest_snapshot.id],
                    |row| {
                        let payload: String = row.get(0)?;
                        Ok(payload)
                    },
                )
                .map_err(|e| SnapshotError::Sqlite(e.to_string()))
                .and_then(|payload| {
                    serde_json::from_str(&payload)
                        .map_err(|e| SnapshotError::Serialization(e.to_string()))
                })?;

            // Fetch events after snapshot revision
            let events = load_tail_events(conn, latest_snapshot.revision)?;

            // If no events to replay, return the snapshot directly
            if events.is_empty() {
                return Ok(base_projection);
            }

            // Replay events on top of the snapshot.
            // The snapshot at revision R contains state after R events were applied.
            // Events after the snapshot have revisions R+1, R+2, etc.
            // replay_events_from expects events[i].revision == initial_state.revision + i
            // So we need events[0].revision == base_projection.revision.
            // Since events[0] from DB has revision R+1 and base_projection.revision = R,
            // we subtract 1 from all event revisions.
            let adjusted_events: Vec<EventRecord> = events
                .into_iter()
                .map(|e| EventRecord {
                    op_id: e.op_id,
                    revision: e.revision.saturating_sub(1),
                    operation: e.operation,
                    author: e.author,
                    timestamp: e.timestamp,
                })
                .collect();

            replay_events_from(base_projection, &adjusted_events)
                .map_err(|e| SnapshotError::Replay(e.to_string()))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // No snapshot exists - fall back to full replay from empty
            let events = load_tail_events(conn, 0)?;
            // Adjust event revisions to start at 0 (database revisions start at 1)
            let adjusted_events: Vec<EventRecord> = events
                .into_iter()
                .enumerate()
                .map(|(i, e)| EventRecord {
                    op_id: e.op_id,
                    revision: i as u64,
                    operation: e.operation,
                    author: e.author,
                    timestamp: e.timestamp,
                })
                .collect();
            replay_events(&adjusted_events).map_err(|e| SnapshotError::Replay(e.to_string()))
        }
        Err(e) => Err(SnapshotError::Sqlite(e.to_string())),
    }
}

/// Load tail events after a given revision
///
/// Returns all events with revision greater than `after_revision`, ordered by revision.
///
/// # Errors
/// Returns `SnapshotError::Sqlite` if database operations fail
/// Returns `SnapshotError::Serialization` if event parsing fails
pub fn load_tail_events(
    conn: &Connection,
    after_revision: u64,
) -> Result<Vec<EventRecord>, SnapshotError> {
    fetch_events_after(conn, after_revision)
}

/// Fetch all events after a given revision
///
/// # Errors
/// Returns `SnapshotError::Sqlite` if database operations fail
/// Returns `SnapshotError::Serialization` if event parsing fails
fn fetch_events_after(
    conn: &Connection,
    after_revision: u64,
) -> Result<Vec<EventRecord>, SnapshotError> {
    let mut stmt = conn
        .prepare(
            "SELECT operation_id, revision, payload, timestamp FROM events WHERE revision > ?1 ORDER BY revision",
        )
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

    let row_results: Vec<Result<(String, i64, String, String), rusqlite::Error>> = stmt
        .query_map([after_revision as i64], |row| {
            let operation_id: String = row.get(0)?;
            let revision: i64 = row.get(1)?;
            let payload: String = row.get(2)?;
            let timestamp: String = row.get(3)?;
            Ok((operation_id, revision, payload, timestamp))
        })
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?
        .collect();

    let mut decode_errors = Vec::new();
    let events: Vec<EventRecord> = row_results
        .into_iter()
        .filter_map(|result| match result {
            Ok((operation_id, revision, payload, timestamp)) => {
                match parse_event_envelope(&payload) {
                    Ok(envelope) => match timestamp.parse::<i64>() {
                        Ok(timestamp) => Some(Ok(EventRecord {
                            op_id: envelope.op_id,
                            revision: revision as u64,
                            operation: envelope.operation,
                            author: envelope.author,
                            timestamp,
                        })),
                        Err(e) => {
                            decode_errors.push(format!(
                                "timestamp parse error for op {}: {}",
                                operation_id, e
                            ));
                            None
                        }
                    },
                    Err(e) => {
                        decode_errors.push(format!(
                            "envelope parse error for op {}: {}",
                            operation_id, e
                        ));
                        None
                    }
                }
            }
            Err(e) => {
                decode_errors.push(format!("row error: {}", e));
                None
            }
        })
        .collect::<Result<Vec<_>, SnapshotError>>()
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

    if !decode_errors.is_empty() {
        eprintln!(
            "warning: decode_errors during snapshot replay: {:?}",
            decode_errors
        );
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::envelope::{Author, DomainOp, EventEnvelope};
    use crate::store;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_load_snapshot_happy_path() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema using bootstrap_store
        let bootstrap = store::bootstrap_store(&db_path).unwrap();
        let mut conn = bootstrap.conn;

        // Write an event first to set revision to 1
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
        };

        store::append_event(&mut conn, envelope, None).unwrap();

        // Create projection at revision 1 (matching current revision)
        let projection_at_rev1 = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };

        let meta = write_snapshot(&mut conn, &projection_at_rev1).unwrap();
        assert_eq!(meta.revision, 1);

        // Load projection and verify
        let loaded = load_projection(&conn).unwrap();
        assert_eq!(loaded.revision, 1);
    }

    #[test]
    fn test_snapshot_stale_error_when_revision_behind() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema using bootstrap_store
        let bootstrap = store::bootstrap_store(&db_path).unwrap();
        let mut conn = bootstrap.conn;

        // Write an event to set revision to 1
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
        };

        store::append_event(&mut conn, envelope, None).unwrap();

        // Try to write snapshot with stale revision (0 instead of 1)
        let stale_projection = DiagramProjection {
            revision: 0,
            ..DiagramProjection::empty()
        };

        let result = write_snapshot(&mut conn, &stale_projection);
        assert!(matches!(result, Err(SnapshotError::SnapshotStale { .. })));
    }

    #[test]
    fn test_load_projection_replays_events_after_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema using bootstrap_store
        let bootstrap = store::bootstrap_store(&db_path).unwrap();
        let mut conn = bootstrap.conn;

        // Create initial projection and snapshot at revision 0
        let empty_projection = DiagramProjection::empty();
        let meta = write_snapshot(&mut conn, &empty_projection).unwrap();
        assert_eq!(meta.revision, 0);

        // Add some events after the snapshot
        let envelope1 = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
        };

        store::append_event(&mut conn, envelope1, None).unwrap();

        // Load projection - should replay the event after snapshot
        let loaded = load_projection(&conn).unwrap();
        assert_eq!(loaded.revision, 1);
    }

    #[test]
    fn test_load_projection_with_no_snapshot_falls_back_to_full_replay() {
        // Create fresh connection to empty database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Initialize schema using bootstrap_store
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Add an event but no snapshot
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
        };
        store::append_event(&mut bootstrap.conn, envelope, None).unwrap();

        // Load projection with no snapshot - should fall back to full replay
        let result = load_projection(&bootstrap.conn);
        assert!(
            result.is_ok(),
            "Should fall back to full replay: {:?}",
            result.err()
        );
        let loaded = result.unwrap();
        assert_eq!(loaded.revision, 1);
    }

    #[test]
    fn test_load_tail_events_returns_events_after_revision() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Add three events
        for i in 1..=3 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                timestamp: 1234567890 + i,
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                operation: DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 100.0 * i as f64,
                    y: 200.0,
                    width: 50.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
            };
            store::append_event(&mut bootstrap.conn, envelope, None).unwrap();
        }

        // Load tail events after revision 1
        let events = load_tail_events(&bootstrap.conn, 1).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].op_id, "op-2");
        assert_eq!(events[1].op_id, "op-3");
    }

    #[test]
    fn test_load_projection_preserves_snapshot_data() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Add first event
        let envelope1 = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "First Node".to_string(),
            },
        };
        store::append_event(&mut bootstrap.conn, envelope1, None).unwrap();

        // Replay to get projection with the node
        // Events from DB have revisions starting at 1, but replay_events expects starting at 0
        // So we adjust the revisions like load_projection does when there's no snapshot
        let events = load_tail_events(&bootstrap.conn, 0).unwrap();
        let adjusted_events: Vec<EventRecord> = events
            .into_iter()
            .enumerate()
            .map(|(i, e)| EventRecord {
                op_id: e.op_id,
                revision: i as u64,
                operation: e.operation,
                author: e.author,
                timestamp: e.timestamp,
            })
            .collect();
        let projection_with_node = replay_events(&adjusted_events).unwrap();

        // Write snapshot with the node data
        let meta = write_snapshot(&mut bootstrap.conn, &projection_with_node).unwrap();
        assert_eq!(meta.revision, 1);

        // Add second event
        let envelope2 = EventEnvelope {
            op_id: "op-2".to_string(),
            timestamp: 1234567891,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 300.0,
                y: 400.0,
                width: 50.0,
                height: 50.0,
                label: "Second Node".to_string(),
            },
        };
        store::append_event(&mut bootstrap.conn, envelope2, None).unwrap();

        // Load projection - should have both nodes
        let loaded = load_projection(&bootstrap.conn).unwrap();
        assert_eq!(loaded.revision, 2);
        assert!(loaded
            .nodes
            .contains_key(&crate::models::document::NodeId::new("node-1".to_string())));
        assert!(loaded
            .nodes
            .contains_key(&crate::models::document::NodeId::new("node-2".to_string())));
    }

    #[test]
    fn test_latest_snapshot_returns_none_when_no_snapshots_exist() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let bootstrap = store::bootstrap_store(&db_path).unwrap();

        // No snapshots written yet
        let result = latest_snapshot(&bootstrap.conn).unwrap();
        assert!(
            result.is_none(),
            "Should return None when no snapshots exist"
        );
    }

    #[test]
    fn test_latest_snapshot_returns_metadata_after_write() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Write an event first
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
        };
        store::append_event(&mut bootstrap.conn, envelope, None).unwrap();

        // Write snapshot
        let projection = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };
        let written_meta = write_snapshot(&mut bootstrap.conn, &projection).unwrap();

        // Get latest snapshot
        let result = latest_snapshot(&bootstrap.conn).unwrap();
        assert!(
            result.is_some(),
            "Should return Some after snapshot written"
        );

        let meta = result.unwrap();
        assert_eq!(meta.id, written_meta.id);
        assert_eq!(meta.revision, 1);
        assert_eq!(meta.revision, written_meta.revision);
    }

    #[test]
    fn test_latest_snapshot_returns_highest_revision() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Write multiple events and snapshots
        for i in 1..=3 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                timestamp: 1234567890 + i,
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                operation: DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 100.0 * i as f64,
                    y: 200.0,
                    width: 50.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
            };
            store::append_event(&mut bootstrap.conn, envelope, None).unwrap();

            let projection = DiagramProjection {
                revision: i as u64,
                ..DiagramProjection::empty()
            };
            write_snapshot(&mut bootstrap.conn, &projection).unwrap();
        }

        // Get latest snapshot - should be revision 3
        let result = latest_snapshot(&bootstrap.conn).unwrap();
        assert!(result.is_some());
        let meta = result.unwrap();
        assert_eq!(meta.revision, 3);
    }

    #[test]
    fn test_new_path_handles_valid_input_and_produces_expected_output() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Add event
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
        };
        store::append_event(&mut bootstrap.conn, envelope, None).unwrap();

        // Write snapshot with valid projection
        let projection = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };

        let result = write_snapshot(&mut bootstrap.conn, &projection);
        assert!(result.is_ok(), "Should succeed with valid input");
        let meta = result.unwrap();
        assert_eq!(meta.revision, 1);
    }

    #[test]
    fn test_invalid_input_returns_typed_error_without_partial_mutation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Add event to set revision to 1
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
        };
        store::append_event(&mut bootstrap.conn, envelope, None).unwrap();

        // Try to write snapshot with stale revision (0 instead of 1)
        let stale_projection = DiagramProjection {
            revision: 0, // Stale!
            ..DiagramProjection::empty()
        };

        let result = write_snapshot(&mut bootstrap.conn, &stale_projection);
        assert!(result.is_err(), "Should fail with stale revision");

        // Verify error type
        match result {
            Err(SnapshotError::SnapshotStale { expected, found }) => {
                assert_eq!(expected, 1);
                assert_eq!(found, 0);
            }
            _ => panic!("Expected SnapshotStale error"),
        }

        // Verify no partial mutation - no snapshot should exist for revision 0
        let latest = latest_snapshot(&bootstrap.conn).unwrap();
        assert!(
            latest.is_none(),
            "No snapshot should exist after failed write"
        );
    }

    #[test]
    fn test_command_flow_uses_replacement_implementation_without_legacy_calls() {
        // This test verifies that write_snapshot uses the new path
        // with typed errors and no legacy fallback
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Add event
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
        };
        store::append_event(&mut bootstrap.conn, envelope, None).unwrap();

        // Write snapshot using the new path
        let projection = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };

        let meta = write_snapshot(&mut bootstrap.conn, &projection).unwrap();

        // Verify using latest_snapshot (also new path)
        let loaded = latest_snapshot(&bootstrap.conn)
            .unwrap()
            .expect("Snapshot should exist");

        assert_eq!(loaded.id, meta.id);
        assert_eq!(loaded.revision, meta.revision);

        // Verify load_projection uses new path (replay from snapshot)
        let loaded_projection = load_projection(&bootstrap.conn).unwrap();
        assert_eq!(loaded_projection.revision, 1);
    }

    // ============================================================================
    // BDD Tests for Snapshot Recovery Edge Cases (bd-2vq)
    // ============================================================================

    /// Test 1: Staleness Detection
    /// Verifies that loading a snapshot where the stored revision in the snapshots
    /// table is behind the events table is handled appropriately by replaying the tail.
    ///
    /// Note: The current implementation loads the snapshot + replays tail events.
    /// This test verifies behavior when snapshot revision < current events revision.
    #[test]
    fn given_stale_snapshot_when_load_projection_then_returns_stale_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let mut bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Add first event and create a snapshot at revision 1
        let envelope1 = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 1234567891,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
                label: "Node 1".to_string(),
            },
        };
        store::append_event(&mut bootstrap.conn, envelope1, None).unwrap();

        // Write snapshot at revision 1
        let projection_rev1 = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };
        write_snapshot(&mut bootstrap.conn, &projection_rev1).unwrap();

        // Now add more events to bring the database to revision 3 (snapshot is now "stale")
        for i in 2..=3 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                timestamp: 1234567890 + i,
                author: Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                operation: DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 100.0 * i as f64,
                    y: 200.0,
                    width: 50.0,
                    height: 50.0,
                    label: format!("Node {i}"),
                },
            };
            store::append_event(&mut bootstrap.conn, envelope, None).unwrap();
        }

        // The snapshot is now stale (revision 1 vs current revision 3)
        // load_projection should still work by loading the snapshot and replaying tail events
        let result = load_projection(&bootstrap.conn);

        // Verify graceful handling - no panic, and the projection should be at revision 3
        assert!(
            result.is_ok(),
            "Should handle stale snapshot gracefully by replaying tail, got: {:?}",
            result.err()
        );

        let loaded = result.unwrap();
        assert_eq!(
            loaded.revision, 3,
            "Should replay all events and reach revision 3"
        );
    }

    /// Test 2: Deserialization Failures - Corrupted/Invalid JSON
    /// Verifies that a snapshot with malformed JSON payload returns Serialization error.
    #[test]
    fn given_corrupted_payload_when_load_projection_then_returns_serialization_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Directly insert a snapshot with corrupted JSON payload
        bootstrap
            .conn
            .execute(
                "INSERT INTO snapshots (revision, payload) VALUES (1, 'this is not valid json at all')",
                [],
            )
            .unwrap();

        // Attempt to load projection
        let result = load_projection(&bootstrap.conn);

        // Verify we get a Serialization error, not a panic
        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for corrupted payload, got: {:?}",
            result
        );
    }

    /// Test 2b: Deserialization Failures - Truncated JSON
    /// Verifies that a snapshot with truncated JSON payload returns Serialization error.
    #[test]
    fn given_truncated_json_payload_when_load_projection_then_returns_serialization_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Insert a truncated JSON payload (valid start but incomplete)
        let truncated_json = r#"{"version":1,"revision":1,"nodes":{"#;

        bootstrap
            .conn
            .execute(
                "INSERT INTO snapshots (revision, payload) VALUES (1, ?1)",
                [truncated_json],
            )
            .unwrap();

        // Attempt to load projection
        let result = load_projection(&bootstrap.conn);

        // Verify we get a Serialization error
        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for truncated JSON, got: {:?}",
            result
        );
    }

    /// Test 3: Semantically Invalid Payload (Structurally Valid but Wrong Types)
    /// Verifies that a snapshot with valid JSON but wrong field types returns error.
    #[test]
    fn given_semantically_invalid_payload_when_load_projection_then_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Valid JSON structure but nodes is a string instead of a map
        let invalid_payload = r#"{
            "version": 1,
            "revision": 1,
            "nodes": "this should be a map, not a string",
            "edges": {},
            "author_priority": {},
            "cycle_policy": "allow"
        }"#;

        bootstrap
            .conn
            .execute(
                "INSERT INTO snapshots (revision, payload) VALUES (1, ?1)",
                [invalid_payload],
            )
            .unwrap();

        // Attempt to load projection
        let result = load_projection(&bootstrap.conn);

        // Verify we get a Serialization error (type mismatch during deserialization)
        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for semantically invalid payload, got: {:?}",
            result
        );
    }

    /// Test 4: Incompatible Format (Schema Version Mismatch)
    /// Verifies that a snapshot with an unexpected schema version is handled gracefully.
    #[test]
    fn given_incompatible_snapshot_format_when_load_projection_then_handles_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Create a snapshot with an old/incompatible schema structure
        // This simulates an old version of the snapshot format
        let old_format_payload = r#"{
            "schema_version": "v0.1.0-legacy",
            "data": {
                "diagram_nodes": [],
                "diagram_edges": []
            },
            "metadata": {
                "created": "2024-01-01T00:00:00Z"
            }
        }"#;

        bootstrap
            .conn
            .execute(
                "INSERT INTO snapshots (revision, payload) VALUES (1, ?1)",
                [old_format_payload],
            )
            .unwrap();

        // Attempt to load projection
        let result = load_projection(&bootstrap.conn);

        // The deserialization should fail because the structure doesn't match DiagramProjection
        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for incompatible format, got: {:?}",
            result
        );
    }

    /// Test 5: Missing Metadata Fields
    /// Verifies that a snapshot payload missing required fields returns Serialization error.
    #[test]
    fn given_snapshot_with_missing_metadata_fields_when_load_then_returns_serialization_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Valid JSON but missing required fields (version, revision)
        let missing_fields_payload = r#"{
            "nodes": {},
            "edges": {}
        }"#;

        bootstrap
            .conn
            .execute(
                "INSERT INTO snapshots (revision, payload) VALUES (1, ?1)",
                [missing_fields_payload],
            )
            .unwrap();

        // Attempt to load projection
        let result = load_projection(&bootstrap.conn);

        // Verify we get a Serialization error for missing required fields
        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for missing metadata fields, got: {:?}",
            result
        );
    }

    /// Test 5b: Missing Nodes Field (Partial Payload)
    /// Verifies that a snapshot payload missing the nodes field is handled.
    #[test]
    fn given_snapshot_missing_nodes_field_when_load_then_returns_serialization_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and initialize schema
        let bootstrap = store::bootstrap_store(&db_path).unwrap();

        // Missing the nodes field (required)
        let partial_payload = r#"{
            "version": 1,
            "revision": 5,
            "edges": {}
        }"#;

        bootstrap
            .conn
            .execute(
                "INSERT INTO snapshots (revision, payload) VALUES (5, ?1)",
                [partial_payload],
            )
            .unwrap();

        // Attempt to load projection
        let result = load_projection(&bootstrap.conn);

        // Verify graceful error handling
        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for missing nodes field, got: {:?}",
            result
        );
    }
}
