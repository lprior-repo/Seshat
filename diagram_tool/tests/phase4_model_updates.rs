#![allow(clippy::let_underscore_future)]
//! Phase 4 Tests: Model updates for rusqlite → sqlx migration
//! Strictly < 300 lines, EXTREMELY DRY, Code is a Liability.

#![cfg(not(target_arch = "wasm32"))]

use diagram_models::document::{EdgeId, NodeId};
use diagram_models::envelope::{Author, DomainOp, EventEnvelope};
use diagram_models::projection::{
    replay_events_from, DiagramProjection, EventRecord as ProjectionEventRecord,
};
use diagram_tool::store::Revision;
use diagram_tool::store_async::{
    append_event_async, bootstrap_async_store, envelope_to_valid_event, fetch_all_events,
    fetch_events_since, AsyncAppendResult, AsyncStoreError,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tempfile::TempDir;

/// Test context for async store tests.
/// Manages pool lifecycle and temporary directory cleanup.
struct TestStore {
    pool: Option<Arc<SqlitePool>>,
    _temp_dir: TempDir,
}

impl TestStore {
    async fn new() -> Result<Self, AsyncStoreError> {
        let temp_dir = TempDir::new().map_err(AsyncStoreError::Io)?;
        let db_path = temp_dir.path().join("test.db");
        let bootstrap = bootstrap_async_store(&db_path).await?;
        Ok(Self {
            pool: Some(Arc::new(bootstrap.pool)),
            _temp_dir: temp_dir,
        })
    }

    fn pool(&self) -> Arc<SqlitePool> {
        self.pool.as_ref().expect("pool taken").clone()
    }
}

impl Drop for TestStore {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.take() {
            if Arc::strong_count(&pool) == 1 {
                let _ = pool.close();
            }
        }
    }
}

/// Helper to create a standard test envelope for deduplication.
fn test_envelope(op_id: &str, revision: i64) -> EventEnvelope {
    EventEnvelope {
        op_id: op_id.to_string(),
        timestamp: 1700000000 + revision,
        author: Author {
            id: "u".into(),
            name: "U".into(),
            email: None,
        },
        operation: DomainOp::NodeAdd {
            id: NodeId::new(format!("n-{}", revision)),
            x: 10.0 * revision as f64,
            y: 20.0 * revision as f64,
            width: 80.0,
            height: 40.0,
            label: format!("N{}", revision),
        },
    }
}

/// Helper to append a generic event and abstract away serialization.
async fn append_test_event(
    pool: &SqlitePool,
    id: &str,
    rev: i64,
    exp_rev: Option<Revision>,
) -> Result<AsyncAppendResult, AsyncStoreError> {
    let env = test_envelope(id, rev);
    let valid =
        envelope_to_valid_event(&env).map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;
    append_event_async(pool, valid, exp_rev).await
}

#[tokio::test]
async fn test_store_core_operations() -> Result<(), AsyncStoreError> {
    let store = TestStore::new().await?;
    let pool = store.pool();

    // 1. New store is empty
    assert!(fetch_all_events(&pool).await?.is_empty());

    // 2. Append first event
    let res = append_test_event(&pool, "op-1", 1, None).await?;
    assert_eq!(res.revision, 1);
    assert_eq!(res.op_id, "op-1");

    // 3. Append multiple increments revision
    for i in 2..=5 {
        assert_eq!(
            append_test_event(&pool, &format!("op-{}", i), i, None)
                .await?
                .revision,
            i
        );
    }

    // 4. Fetch events since
    let evs = fetch_events_since(&pool, 2).await?;
    assert_eq!(evs.len(), 3);
    assert!(fetch_events_since(&pool, 5).await?.is_empty());

    // 5. Fetch all events
    assert_eq!(fetch_all_events(&pool).await?.len(), 5);

    // 6. Very large batch
    for i in 6..=105 {
        append_test_event(&pool, &format!("op-batch-{}", i), i, None).await?;
    }
    let all = fetch_all_events(&pool).await?;
    assert_eq!(all.len(), 105);
    assert_eq!(all.last().unwrap().revision, 105);

    Ok(())
}

#[tokio::test]
async fn test_projection_replay() -> Result<(), AsyncStoreError> {
    let store = TestStore::new().await?;
    let pool = store.pool();

    for i in 1..=3 {
        append_test_event(&pool, &format!("op-{}", i), i, None).await?;
    }

    let records = fetch_events_since(&pool, 0).await?;
    assert_eq!(records.len(), 3);

    let parsed_events: Result<Vec<ProjectionEventRecord>, _> = records
        .into_iter()
        .enumerate()
        .map(|(i, r)| -> Result<ProjectionEventRecord, AsyncStoreError> {
            let env = diagram_models::envelope::parse_event_envelope(&r.payload)
                .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;
            Ok(ProjectionEventRecord {
                op_id: r.op_id,
                revision: i as u64,
                operation: env.operation,
                author: env.author,
                timestamp: r.timestamp,
            })
        })
        .collect();

    let proj = replay_events_from(DiagramProjection::empty(), &parsed_events?)
        .map_err(|e| AsyncStoreError::Serialization(e.to_string()))?;

    assert_eq!(proj.revision, 3);
    assert_eq!(proj.nodes.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_store_error_cases() -> Result<(), AsyncStoreError> {
    let store = TestStore::new().await?;
    let pool = store.pool();

    // Setup initial state
    append_test_event(&pool, "op-1", 1, None).await?;

    // 1. Revision mismatch is typed
    let mismatch_res = append_test_event(&pool, "op-2", 2, Some(Revision::new(5).unwrap())).await;
    assert!(matches!(
        mismatch_res,
        Err(AsyncStoreError::RevisionMismatch {
            expected: 5,
            found: 1
        })
    ));

    // 2. Duplicate op_id fails
    let dup_res = append_test_event(&pool, "op-1", 1, None).await;
    assert!(dup_res.is_err(), "Duplicate append should fail");

    Ok(())
}

#[tokio::test]
async fn test_batch_append_with_edges() -> Result<(), AsyncStoreError> {
    let store = TestStore::new().await?;
    let pool = store.pool();

    let envelopes = vec![
        EventEnvelope {
            op_id: "op-1".into(),
            timestamp: 1,
            author: Author {
                id: "u".into(),
                name: "U".into(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: NodeId::new("node-1".into()),
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
                label: "N1".into(),
            },
        },
        EventEnvelope {
            op_id: "op-2".into(),
            timestamp: 2,
            author: Author {
                id: "u".into(),
                name: "U".into(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: NodeId::new("node-2".into()),
                x: 200.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
                label: "N2".into(),
            },
        },
        EventEnvelope {
            op_id: "op-3".into(),
            timestamp: 3,
            author: Author {
                id: "u".into(),
                name: "U".into(),
                email: None,
            },
            operation: DomainOp::EdgeConnect {
                id: EdgeId::new("edge-1".into()),
                source: NodeId::new("node-1".into()),
                target: NodeId::new("node-2".into()),
            },
        },
    ];

    for env in envelopes {
        let valid = envelope_to_valid_event(&env).unwrap();
        append_event_async(&pool, valid, None).await?;
    }

    let records = fetch_events_since(&pool, 0).await?;
    let parsed: Vec<_> = records
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let env = diagram_models::envelope::parse_event_envelope(&r.payload).unwrap();
            ProjectionEventRecord {
                op_id: r.op_id,
                revision: i as u64,
                operation: env.operation,
                author: env.author,
                timestamp: r.timestamp,
            }
        })
        .collect();

    let proj = replay_events_from(DiagramProjection::empty(), &parsed).unwrap();
    assert_eq!(proj.revision, 3);
    assert_eq!(proj.nodes.len(), 2);
    assert_eq!(proj.edges.len(), 1);

    Ok(())
}
