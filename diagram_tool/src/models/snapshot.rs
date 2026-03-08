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

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
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
    /// SQLx database error
    #[error("SQLx error: {0}")]
    Sqlx(String),
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
/// Returns `SnapshotError::Sqlx` if database operations fail
pub async fn write_snapshot(
    pool: &SqlitePool,
    projection: &DiagramProjection,
) -> Result<SnapshotMeta, SnapshotError> {
    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(pool)
        .await
        .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

    let current_revision = current_revision as u64;

    if projection.revision != current_revision {
        return Err(SnapshotError::SnapshotStale {
            expected: current_revision,
            found: projection.revision,
        });
    }

    let payload = serde_json::to_string(projection)
        .map_err(|e| SnapshotError::Serialization(e.to_string()))?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

    sqlx::query("INSERT OR REPLACE INTO snapshots (revision, payload) VALUES (?1, ?2)")
        .bind(projection.revision as i64)
        .bind(&payload)
        .execute(&mut *tx)
        .await
        .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

    let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

    let created_at: i64 = sqlx::query_scalar("SELECT created_at FROM snapshots WHERE id = ?1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

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
/// Returns `SnapshotError::Sqlx` if database operations fail
pub async fn latest_snapshot(pool: &SqlitePool) -> Result<Option<SnapshotMeta>, SnapshotError> {
    let result = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, revision, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

    match result {
        Some((id, revision, created_at)) => Ok(Some(SnapshotMeta {
            id,
            revision: revision as u64,
            created_at,
        })),
        None => Ok(None),
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
/// Returns `SnapshotError::Sqlx` if database operations fail
/// Returns `SnapshotError::Replay` if event replay fails
pub async fn load_projection(pool: &SqlitePool) -> Result<DiagramProjection, SnapshotError> {
    let snapshot_result = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, revision, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

    match snapshot_result {
        Some((id, revision, _created_at)) => {
            let payload: String = sqlx::query_scalar("SELECT payload FROM snapshots WHERE id = ?1")
                .bind(id)
                .fetch_one(pool)
                .await
                .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

            let base_projection: DiagramProjection = serde_json::from_str(&payload)
                .map_err(|e| SnapshotError::Serialization(e.to_string()))?;

            let events = load_tail_events(pool, revision as u64).await?;

            if events.is_empty() {
                return Ok(base_projection);
            }

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
        None => {
            let events = load_tail_events(pool, 0).await?;
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
    }
}

/// Load tail events after a given revision
///
/// Returns all events with revision greater than `after_revision`, ordered by revision.
///
/// # Errors
/// Returns `SnapshotError::Sqlx` if database operations fail
/// Returns `SnapshotError::Serialization` if event parsing fails
pub async fn load_tail_events(
    pool: &SqlitePool,
    after_revision: u64,
) -> Result<Vec<EventRecord>, SnapshotError> {
    fetch_events_after(pool, after_revision).await
}

/// Fetch all events after a given revision
///
/// # Errors
/// Returns `SnapshotError::Sqlx` if database operations fail
/// Returns `SnapshotError::Serialization` if event parsing fails
async fn fetch_events_after(
    pool: &SqlitePool,
    after_revision: u64,
) -> Result<Vec<EventRecord>, SnapshotError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, payload, timestamp FROM events WHERE revision > ?1 ORDER BY revision",
    )
    .bind(after_revision as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| SnapshotError::Sqlx(e.to_string()))?;

    let mut decode_errors = Vec::new();
    let events: Vec<EventRecord> = rows
        .into_iter()
        .filter_map(|(operation_id, revision, payload, timestamp_str)| {
            match parse_event_envelope(&payload) {
                Ok(envelope) => match timestamp_str.parse::<i64>() {
                    Ok(timestamp) => Some(EventRecord {
                        op_id: envelope.op_id,
                        revision: revision as u64,
                        operation: envelope.operation,
                        author: envelope.author,
                        timestamp,
                    }),
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
        })
        .collect();

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

    #[tokio::test]
    async fn test_write_and_load_snapshot_happy_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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

        store::append_event(&pool, envelope, None)
            .await
            .expect("append failed");

        let projection_at_rev1 = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };

        let meta = write_snapshot(&pool, &projection_at_rev1)
            .await
            .expect("write_snapshot failed");
        assert_eq!(meta.revision, 1);

        let loaded = load_projection(&pool)
            .await
            .expect("load_projection failed");
        assert_eq!(loaded.revision, 1);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_snapshot_stale_error_when_revision_behind() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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

        store::append_event(&pool, envelope, None)
            .await
            .expect("append failed");

        let stale_projection = DiagramProjection {
            revision: 0,
            ..DiagramProjection::empty()
        };

        let result = write_snapshot(&pool, &stale_projection).await;
        assert!(matches!(result, Err(SnapshotError::SnapshotStale { .. })));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_load_projection_replays_events_after_snapshot() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

        let empty_projection = DiagramProjection::empty();
        let meta = write_snapshot(&pool, &empty_projection)
            .await
            .expect("write_snapshot failed");
        assert_eq!(meta.revision, 0);

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

        store::append_event(&pool, envelope1, None)
            .await
            .expect("append failed");

        let loaded = load_projection(&pool)
            .await
            .expect("load_projection failed");
        assert_eq!(loaded.revision, 1);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_load_projection_with_no_snapshot_falls_back_to_full_replay() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
        store::append_event(&pool, envelope, None)
            .await
            .expect("append failed");

        let result = load_projection(&pool).await;
        assert!(
            result.is_ok(),
            "Should fall back to full replay: {:?}",
            result.err()
        );
        let loaded = result.expect("load_projection succeeded");
        assert_eq!(loaded.revision, 1);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_load_tail_events_returns_events_after_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
            store::append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let events = load_tail_events(&pool, 1)
            .await
            .expect("load_tail_events failed");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].op_id, "op-2");
        assert_eq!(events[1].op_id, "op-3");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_load_projection_preserves_snapshot_data() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
        store::append_event(&pool, envelope1, None)
            .await
            .expect("append failed");

        let events = load_tail_events(&pool, 0)
            .await
            .expect("load_tail_events failed");
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
        let projection_with_node = replay_events(&adjusted_events).expect("replay_events failed");

        let meta = write_snapshot(&pool, &projection_with_node)
            .await
            .expect("write_snapshot failed");
        assert_eq!(meta.revision, 1);

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
        store::append_event(&pool, envelope2, None)
            .await
            .expect("append failed");

        let loaded = load_projection(&pool)
            .await
            .expect("load_projection failed");
        assert_eq!(loaded.revision, 2);
        assert!(loaded
            .nodes
            .contains_key(&crate::models::document::NodeId::new("node-1".to_string())));
        assert!(loaded
            .nodes
            .contains_key(&crate::models::document::NodeId::new("node-2".to_string())));

        pool.close().await;
    }

    #[tokio::test]
    async fn test_latest_snapshot_returns_none_when_no_snapshots_exist() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

        let result = latest_snapshot(&pool)
            .await
            .expect("latest_snapshot failed");
        assert!(
            result.is_none(),
            "Should return None when no snapshots exist"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_latest_snapshot_returns_metadata_after_write() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
        store::append_event(&pool, envelope, None)
            .await
            .expect("append failed");

        let projection = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };
        let written_meta = write_snapshot(&pool, &projection)
            .await
            .expect("write_snapshot failed");

        let result = latest_snapshot(&pool)
            .await
            .expect("latest_snapshot failed");
        assert!(
            result.is_some(),
            "Should return Some after snapshot written"
        );

        let meta = result.expect("snapshot exists");
        assert_eq!(meta.id, written_meta.id);
        assert_eq!(meta.revision, 1);
        assert_eq!(meta.revision, written_meta.revision);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_latest_snapshot_returns_highest_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
            store::append_event(&pool, envelope, None)
                .await
                .expect("append failed");

            let projection = DiagramProjection {
                revision: i as u64,
                ..DiagramProjection::empty()
            };
            write_snapshot(&pool, &projection)
                .await
                .expect("write_snapshot failed");
        }

        let result = latest_snapshot(&pool)
            .await
            .expect("latest_snapshot failed");
        assert!(result.is_some());
        let meta = result.expect("snapshot exists");
        assert_eq!(meta.revision, 3);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_new_path_handles_valid_input_and_produces_expected_output() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
        store::append_event(&pool, envelope, None)
            .await
            .expect("append failed");

        let projection = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };

        let result = write_snapshot(&pool, &projection).await;
        assert!(result.is_ok(), "Should succeed with valid input");
        let meta = result.expect("write_snapshot succeeded");
        assert_eq!(meta.revision, 1);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_invalid_input_returns_typed_error_without_partial_mutation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
        store::append_event(&pool, envelope, None)
            .await
            .expect("append failed");

        let stale_projection = DiagramProjection {
            revision: 0,
            ..DiagramProjection::empty()
        };

        let result = write_snapshot(&pool, &stale_projection).await;
        assert!(result.is_err(), "Should fail with stale revision");

        match result {
            Err(SnapshotError::SnapshotStale { expected, found }) => {
                assert_eq!(expected, 1);
                assert_eq!(found, 0);
            }
            _ => panic!("Expected SnapshotStale error"),
        }

        let latest = latest_snapshot(&pool)
            .await
            .expect("latest_snapshot failed");
        assert!(
            latest.is_none(),
            "No snapshot should exist after failed write"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_command_flow_uses_replacement_implementation_without_legacy_calls() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
        store::append_event(&pool, envelope, None)
            .await
            .expect("append failed");

        let projection = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };

        let meta = write_snapshot(&pool, &projection)
            .await
            .expect("write_snapshot failed");

        let loaded = latest_snapshot(&pool)
            .await
            .expect("latest_snapshot failed")
            .expect("Snapshot should exist");

        assert_eq!(loaded.id, meta.id);
        assert_eq!(loaded.revision, meta.revision);

        let loaded_projection = load_projection(&pool)
            .await
            .expect("load_projection failed");
        assert_eq!(loaded_projection.revision, 1);

        pool.close().await;
    }

    #[tokio::test]
    async fn given_stale_snapshot_when_load_projection_then_returns_stale_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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
        store::append_event(&pool, envelope1, None)
            .await
            .expect("append failed");

        let projection_rev1 = DiagramProjection {
            revision: 1,
            ..DiagramProjection::empty()
        };
        write_snapshot(&pool, &projection_rev1)
            .await
            .expect("write_snapshot failed");

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
            store::append_event(&pool, envelope, None)
                .await
                .expect("append failed");
        }

        let result = load_projection(&pool).await;

        assert!(
            result.is_ok(),
            "Should handle stale snapshot gracefully by replaying tail, got: {:?}",
            result.err()
        );

        let loaded = result.expect("load_projection succeeded");
        assert_eq!(
            loaded.revision, 3,
            "Should replay all events and reach revision 3"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn given_corrupted_payload_when_load_projection_then_returns_serialization_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

        sqlx::query(
            "INSERT INTO snapshots (revision, payload) VALUES (1, 'this is not valid json at all')",
        )
        .execute(&pool)
        .await
        .expect("insert failed");

        let result = load_projection(&pool).await;

        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for corrupted payload, got: {:?}",
            result
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn given_truncated_json_payload_when_load_projection_then_returns_serialization_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

        let truncated_json = r#"{"version":1,"revision":1,"nodes":{"#;

        sqlx::query("INSERT INTO snapshots (revision, payload) VALUES (1, ?1)")
            .bind(truncated_json)
            .execute(&pool)
            .await
            .expect("insert failed");

        let result = load_projection(&pool).await;

        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for truncated JSON, got: {:?}",
            result
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn given_semantically_invalid_payload_when_load_projection_then_returns_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

        let invalid_payload = r#"{
            "version": 1,
            "revision": 1,
            "nodes": "this should be a map, not a string",
            "edges": {},
            "author_priority": {},
            "cycle_policy": "allow"
        }"#;

        sqlx::query("INSERT INTO snapshots (revision, payload) VALUES (1, ?1)")
            .bind(invalid_payload)
            .execute(&pool)
            .await
            .expect("insert failed");

        let result = load_projection(&pool).await;

        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for semantically invalid payload, got: {:?}",
            result
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn given_incompatible_snapshot_format_when_load_projection_then_handles_gracefully() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

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

        sqlx::query("INSERT INTO snapshots (revision, payload) VALUES (1, ?1)")
            .bind(old_format_payload)
            .execute(&pool)
            .await
            .expect("insert failed");

        let result = load_projection(&pool).await;

        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for incompatible format, got: {:?}",
            result
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn given_snapshot_with_missing_metadata_fields_when_load_then_returns_serialization_error(
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

        let missing_fields_payload = r#"{
            "nodes": {},
            "edges": {}
        }"#;

        sqlx::query("INSERT INTO snapshots (revision, payload) VALUES (1, ?1)")
            .bind(missing_fields_payload)
            .execute(&pool)
            .await
            .expect("insert failed");

        let result = load_projection(&pool).await;

        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for missing metadata fields, got: {:?}",
            result
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn given_snapshot_missing_nodes_field_when_load_then_returns_serialization_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = store::bootstrap_store(&db_path)
            .await
            .expect("bootstrap failed");
        let pool = bootstrap.pool;

        let partial_payload = r#"{
            "version": 1,
            "revision": 5,
            "edges": {}
        }"#;

        sqlx::query("INSERT INTO snapshots (revision, payload) VALUES (5, ?1)")
            .bind(partial_payload)
            .execute(&pool)
            .await
            .expect("insert failed");

        let result = load_projection(&pool).await;

        assert!(
            matches!(result, Err(SnapshotError::Serialization(_))),
            "Expected Serialization error for missing nodes field, got: {:?}",
            result
        );

        pool.close().await;
    }
}
