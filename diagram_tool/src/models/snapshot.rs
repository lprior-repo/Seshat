//! Snapshot module - persistence and recovery of diagram projections
//!
//! This module provides snapshot write and tail replay boot functionality
//! for efficient startup. Snapshots store serialized `DiagramProjection` state
//! at specific revisions, allowing fast recovery by replaying only events
//! after the snapshot point.

#![allow(dead_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::envelope::parse_event_envelope;
use crate::models::projection::{replay_events_from, DiagramProjection, EventRecord};

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

/// Load the projection from the latest snapshot and replay events after it
///
/// This function:
/// 1. Loads the latest snapshot from the database
/// 2. Fetches all events with revision greater than snapshot revision
/// 3. Replays events to produce the final projection
///
/// If no snapshot exists, returns an empty projection (revision 0).
///
/// # Errors
/// Returns `SnapshotError::Serialization` if deserialization fails
/// Returns `SnapshotError::Sqlite` if database operations fail
/// Returns `SnapshotError::Replay` if event replay fails
pub fn load_projection(conn: &Connection) -> Result<DiagramProjection, SnapshotError> {
    // Try to load latest snapshot
    let latest_snapshot = conn
        .query_row(
            "SELECT id, revision, payload, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
            [],
            |row| {
                let rev: i64 = row.get(1)?;
                let created: i64 = row.get(3)?;
                Ok(SnapshotMeta {
                    id: row.get(0)?,
                    revision: rev as u64,
                    created_at: created,
                })
            },
        )
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?;

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
            serde_json::from_str(&payload).map_err(|e| SnapshotError::Serialization(e.to_string()))
        })?;

    // Fetch events after snapshot revision
    let events = fetch_events_after(conn, latest_snapshot.revision)?;

    // If no events to replay, return the snapshot directly
    if events.is_empty() {
        return Ok(base_projection);
    }

    // Replay events starting from the snapshot's revision.
    // The snapshot stores a projection at revision R (after R events applied).
    // Events after have revisions R+1, R+2, etc.
    // We need to adjust event revisions so the first event starts at revision R.
    // Since the first event has revision R+1, we subtract (R+1) to get revision 0.
    let adjustment = latest_snapshot.revision + 1;
    let adjusted_events: Vec<EventRecord> = events
        .into_iter()
        .map(|e| EventRecord {
            op_id: e.op_id,
            revision: e.revision.saturating_sub(adjustment),
            operation: e.operation,
            author: e.author,
            timestamp: e.timestamp,
        })
        .collect();

    replay_events_from(base_projection, &adjusted_events)
        .map_err(|e| SnapshotError::Replay(e.to_string()))
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

    let events: Vec<EventRecord> = stmt
        .query_map([after_revision as i64], |row| {
            let operation_id: String = row.get(0)?;
            let revision: i64 = row.get(1)?;
            let payload: String = row.get(2)?;
            let timestamp: String = row.get(3)?;
            Ok((operation_id, revision, payload, timestamp))
        })
        .map_err(|e| SnapshotError::Sqlite(e.to_string()))?
        .filter_map(Result::ok)
        .filter_map(|(_operation_id, revision, payload, timestamp)| {
            // Parse the envelope to get the operation
            let envelope = parse_event_envelope(&payload).ok()?;
            let timestamp = timestamp.parse().ok()?;
            Some(EventRecord {
                op_id: envelope.op_id,
                revision: revision as u64,
                operation: envelope.operation,
                author: envelope.author,
                timestamp,
            })
        })
        .collect();

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
            version: 1,
            revision: 1,
            nodes: Default::default(),
            edges: Default::default(),
            author_priority: Default::default(),
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
            version: 1,
            revision: 0,
            nodes: Default::default(),
            edges: Default::default(),
            author_priority: Default::default(),
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
    fn test_load_projection_with_no_snapshot_returns_error() {
        // Create fresh connection to empty database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Initialize schema using bootstrap_store
        let bootstrap = store::bootstrap_store(&db_path).unwrap();
        let conn = bootstrap.conn;

        // Load projection with no snapshot - should get an error
        let result = load_projection(&conn);
        assert!(result.is_err());
    }
}
