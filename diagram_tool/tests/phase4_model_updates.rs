//! Phase 4 Tests: Model updates for rusqlite → sqlx migration

use diagram_tool::models::envelope::{Author, DomainOp, EventEnvelope};
use diagram_tool::models::projection::{replay_events_from, DiagramProjection, EventRecord};
use diagram_tool::store_async::{
    append_event_async, bootstrap_async_store, fetch_all_events, fetch_events_since,
    AsyncAppendResult, AsyncStoreError,
};
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

struct PoolGuard {
    pool: Option<Arc<SqlitePool>>,
}

impl PoolGuard {
    fn new(pool: SqlitePool) -> Self {
        Self {
            pool: Some(Arc::new(pool)),
        }
    }

    fn pool(&self) -> Arc<SqlitePool> {
        self.pool.as_ref().expect("pool already taken").clone()
    }
}

impl Drop for PoolGuard {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.take() {
            if Arc::strong_count(&pool) == 1 {
                let _ = pool.close();
            }
        }
    }
}

fn create_test_envelope(op_id: &str, revision: i64) -> EventEnvelope {
    EventEnvelope {
        op_id: op_id.to_string(),
        timestamp: 1700000000 + revision,
        author: Author {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        operation: DomainOp::NodeAdd {
            id: format!("node-{}", revision),
            x: 100.0 * revision as f64,
            y: 200.0 * revision as f64,
            width: 80.0,
            height: 40.0,
            label: format!("Node {}", revision),
        },
    }
}

async fn setup_async_store(db_path: &Path) -> Result<PoolGuard, AsyncStoreError> {
    let bootstrap = bootstrap_async_store(db_path).await?;
    Ok(PoolGuard::new(bootstrap.pool))
}

mod test_events_module {
    use super::*;

    #[tokio::test]
    async fn test_events_module_imports_async_store_error() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let envelope = create_test_envelope("op-1", 1);
        let result = append_event_async(&pool, envelope, None).await?;

        assert_eq!(result.revision, 1, "First event should have revision 1");
        assert_eq!(result.op_id, "op-1", "Operation ID should match");

        Ok(())
    }

    #[tokio::test]
    async fn test_events_schema_created_with_async_store() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let events = fetch_all_events(&pool).await?;
        assert!(events.is_empty(), "New store should have no events");

        Ok(())
    }

    #[tokio::test]
    async fn test_append_multiple_events_increments_revision() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        for i in 1..=5 {
            let envelope = create_test_envelope(&format!("op-{}", i), i as i64);
            let result = append_event_async(&pool, envelope, None).await?;
            assert_eq!(
                result.revision, i as i64,
                "Revision should match event number"
            );
        }

        Ok(())
    }
}

mod test_snapshot_module {
    use super::*;

    #[tokio::test]
    async fn test_snapshot_works_with_async_store() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let envelope1 = create_test_envelope("op-1", 1);
        append_event_async(&pool, envelope1, None).await?;

        let events = fetch_events_since(&pool, 0).await?;
        assert_eq!(events.len(), 1, "Should have one event");

        Ok(())
    }

    #[tokio::test]
    async fn test_snapshot_load_projection_from_events() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        for i in 1..=3 {
            let envelope = create_test_envelope(&format!("op-{}", i), i as i64);
            append_event_async(&pool, envelope, None).await?;
        }

        let event_records = fetch_events_since(&pool, 0).await?;
        assert_eq!(event_records.len(), 3, "Should have three events");

        let parsed_events: Vec<EventRecord> = event_records
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                let envelope = diagram_tool::models::envelope::parse_event_envelope(&r.payload)
                    .map_err(|e| AsyncStoreError::Serialization(e.to_string()));
                match envelope {
                    Ok(env) => Ok(EventRecord {
                        op_id: r.op_id,
                        revision: i as u64,
                        operation: env.operation,
                        author: env.author,
                        timestamp: r.timestamp,
                    }),
                    Err(e) => Err(e),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let projection = replay_events_from(DiagramProjection::empty(), &parsed_events)
            .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;

        assert_eq!(
            projection.revision, 3,
            "Projection should have revision 3 after 3 events"
        );
        assert_eq!(projection.nodes.len(), 3, "Should have 3 nodes");

        Ok(())
    }
}

mod test_sync_module {
    use super::*;

    #[tokio::test]
    async fn test_sync_fetch_events_since_with_async_store() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        for i in 1..=5 {
            let envelope = create_test_envelope(&format!("op-{}", i), i as i64);
            append_event_async(&pool, envelope, None).await?;
        }

        let events_after_2 = fetch_events_since(&pool, 2).await?;
        assert_eq!(
            events_after_2.len(),
            3,
            "Should have 3 events after revision 2"
        );

        let events_after_5 = fetch_events_since(&pool, 5).await?;
        assert!(
            events_after_5.is_empty(),
            "Should have no events after revision 5"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_sync_fetch_all_events_with_async_store() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let all_before = fetch_all_events(&pool).await?;
        assert!(all_before.is_empty(), "Should have no events initially");

        for i in 1..=3 {
            let envelope = create_test_envelope(&format!("op-{}", i), i as i64);
            append_event_async(&pool, envelope, None).await?;
        }

        let all_after = fetch_all_events(&pool).await?;
        assert_eq!(all_after.len(), 3, "Should have 3 events after appending");

        Ok(())
    }
}

mod test_no_rusqlite_in_models {
    use super::*;

    #[tokio::test]
    async fn test_models_use_async_not_rusqlite() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let envelope = EventEnvelope {
            op_id: "test-op".to_string(),
            timestamp: 1234567890,
            author: Author {
                id: "user-1".to_string(),
                name: "User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
        };

        let result: AsyncAppendResult = append_event_async(&pool, envelope, None).await?;
        assert_eq!(
            result.revision, 1,
            "Should append successfully using async store"
        );

        Ok(())
    }
}

mod test_append_event_async_full_roundtrip {
    use super::*;

    #[tokio::test]
    async fn test_append_and_fetch_verifies_data_integrity() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let original_envelope = EventEnvelope {
            op_id: "op-roundtrip-1".to_string(),
            timestamp: 1700000000,
            author: Author {
                id: "author-1".to_string(),
                name: "Author Name".to_string(),
                email: Some("author@example.com".to_string()),
            },
            operation: DomainOp::NodeAdd {
                id: "node-roundtrip".to_string(),
                x: 150.5,
                y: 250.75,
                width: 120.0,
                height: 60.0,
                label: "Roundtrip Test Node".to_string(),
            },
        };

        let append_result = append_event_async(&pool, original_envelope.clone(), None).await?;
        assert_eq!(append_result.revision, 1, "Should have revision 1");
        assert_eq!(
            append_result.op_id, "op-roundtrip-1",
            "Operation ID should match"
        );

        let fetched_events = fetch_events_since(&pool, 0).await?;
        assert_eq!(fetched_events.len(), 1, "Should fetch exactly one event");

        let fetched_record = &fetched_events[0];
        assert_eq!(fetched_record.op_id, "op-roundtrip-1", "Op ID should match");
        assert_eq!(fetched_record.revision, 1, "Revision should be 1");
        assert_eq!(
            fetched_record.timestamp, 1700000000,
            "Timestamp should match"
        );

        let parsed = diagram_tool::models::envelope::parse_event_envelope(&fetched_record.payload)
            .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;
        assert_eq!(parsed.op_id, "op-roundtrip-1", "Parsed op_id should match");
        assert_eq!(parsed.author.id, "author-1", "Parsed author should match");

        if let DomainOp::NodeAdd { id, x, y, .. } = parsed.operation {
            assert_eq!(id, "node-roundtrip", "Node ID should match");
            assert!((x - 150.5).abs() < 1e-6, "X coordinate should match");
            assert!((y - 250.75).abs() < 1e-6, "Y coordinate should match");
        } else {
            return Err(AsyncStoreError::Serialization(
                "Expected NodeAdd operation".to_string(),
            ));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_append_and_replay_produces_correct_projection(
    ) -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let envelopes = vec![
            EventEnvelope {
                op_id: "op-batch-1".to_string(),
                timestamp: 1700000001,
                author: Author {
                    id: "user-1".to_string(),
                    name: "User 1".to_string(),
                    email: None,
                },
                operation: DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 1".to_string(),
                },
            },
            EventEnvelope {
                op_id: "op-batch-2".to_string(),
                timestamp: 1700000002,
                author: Author {
                    id: "user-1".to_string(),
                    name: "User 1".to_string(),
                    email: None,
                },
                operation: DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 200.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Node 2".to_string(),
                },
            },
            EventEnvelope {
                op_id: "op-batch-3".to_string(),
                timestamp: 1700000003,
                author: Author {
                    id: "user-1".to_string(),
                    name: "User 1".to_string(),
                    email: None,
                },
                operation: DomainOp::EdgeConnect {
                    id: "edge-1".to_string(),
                    source: "node-1".to_string(),
                    target: "node-2".to_string(),
                },
            },
        ];

        for envelope in envelopes {
            append_event_async(&pool, envelope, None).await?;
        }

        let event_records = fetch_events_since(&pool, 0).await?;
        assert_eq!(event_records.len(), 3, "Should have 3 events");

        let parsed_events: Vec<EventRecord> = event_records
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                let envelope = diagram_tool::models::envelope::parse_event_envelope(&r.payload)
                    .map_err(|e| AsyncStoreError::Serialization(e.to_string()));
                match envelope {
                    Ok(env) => Ok(EventRecord {
                        op_id: r.op_id,
                        revision: i as u64, // Start from revision 0 for replay
                        operation: env.operation,
                        author: env.author,
                        timestamp: r.timestamp,
                    }),
                    Err(e) => Err(e),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let projection = replay_events_from(DiagramProjection::empty(), &parsed_events)
            .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;

        assert_eq!(projection.revision, 3, "Projection should have revision 3");
        assert_eq!(projection.nodes.len(), 2, "Should have 2 nodes");
        assert_eq!(projection.edges.len(), 1, "Should have 1 edge");

        Ok(())
    }

    #[tokio::test]
    async fn test_revision_mismatch_error_is_typed() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let envelope = create_test_envelope("op-1", 1);
        append_event_async(&pool, envelope, None).await?;

        let wrong_envelope = create_test_envelope("op-2", 2);
        let mismatch_result = append_event_async(&pool, wrong_envelope, Some(5)).await;

        match mismatch_result {
            Err(AsyncStoreError::RevisionMismatch { expected, found }) => {
                assert_eq!(expected, 5, "Expected revision should be 5");
                assert_eq!(found, 1, "Found revision should be 1");
            }
            Err(other) => {
                return Err(AsyncStoreError::Serialization(format!(
                    "Expected RevisionMismatch, got {:?}",
                    other
                )));
            }
            Ok(_) => {
                return Err(AsyncStoreError::Serialization(
                    "Expected error but got success".to_string(),
                ));
            }
        }

        Ok(())
    }
}

mod test_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_duplicate_op_id_returns_error() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let envelope = create_test_envelope("op-dup", 1);
        let first_result = append_event_async(&pool, envelope.clone(), None).await?;
        assert_eq!(first_result.revision, 1);

        // Idempotent append not implemented - should return error on duplicate
        let duplicate_result = append_event_async(&pool, envelope, None).await;
        assert!(duplicate_result.is_err(), "Duplicate should return error");

        Ok(())
    }

    #[tokio::test]
    async fn test_very_large_batch_append() -> Result<(), AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(|e| AsyncStoreError::Io(e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool_guard = setup_async_store(&db_path).await?;
        let pool = pool_guard.pool();

        let batch_size = 100;
        for i in 1..=batch_size {
            let envelope = create_test_envelope(&format!("op-{}", i), i as i64);
            append_event_async(&pool, envelope, None).await?;
        }

        let all_events = fetch_all_events(&pool).await?;
        assert_eq!(
            all_events.len(),
            batch_size,
            "Should have all {} events",
            batch_size
        );

        let last_event = &all_events[batch_size - 1];
        assert_eq!(
            last_event.revision, batch_size as i64,
            "Last revision should be batch_size"
        );

        Ok(())
    }
}
