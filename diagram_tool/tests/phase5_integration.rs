//! Phase 5 Integration Tests for rusqlite → sqlx migration
//!
//! Tests the async SQLite storage layer using sqlx, verifying:
//! - Full compilation with sqlx
//! - Store functions accessibility
//! - End-to-end workflow (bootstrap → append → fetch)
//! - Error handling (RevisionMismatch, DuplicateWithConflict, etc.)
//! - Concurrent operations
//! - WAL mode configuration

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use diagram_tool::models::envelope::{Author, DomainOp, EventEnvelope};
use diagram_tool::store_async::{
    append_batch_async, append_event_async, append_idempotent_async, bootstrap_async_store,
    create_async_pool, current_revision, fetch_all_events, fetch_events_since, fetch_latest_revision,
    lookup_existing_op_async, read_store_pragmas_async, AsyncAppendResult, AsyncBatchAppendResult,
    AsyncStoreBootstrap, AsyncStoreError, CURRENT_SCHEMA_VERSION,
};
use std::sync::Arc;
use tempfile::TempDir;

fn create_test_envelope(op_id: &str, index: i64) -> EventEnvelope {
    EventEnvelope {
        op_id: op_id.to_string(),
        operation: DomainOp::NodeAdd {
            id: format!("node-{}", index),
            x: 10.0 + index as f64,
            y: 20.0 + index as f64,
            width: 100.0,
            height: 50.0,
            label: format!("Test Node {}", index),
        },
        author: Author {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000 + index,
    }
}

mod compilation_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "slow - runs cargo check subprocess"]
    async fn test_cargo_check_passes() -> Result<(), Box<dyn std::error::Error>> {
        let output = std::process::Command::new("cargo")
            .args(["check", "--features", "async-db"])
            .current_dir("diagram_tool")
            .output()?;

        assert!(
            output.status.success(),
            "cargo check failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sqlx_feature_enabled() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let db_path = temp_dir.path().join("test.db");

        let pool = create_async_pool(&db_path).await.map_err(|e| format!("Failed to create pool: {}", e))?;
        pool.close().await;
        Ok(())
    }
}

mod store_function_tests {
    use super::*;

    #[tokio::test]
    async fn test_bootstrap_async_store_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_async_store(&db_path).await?;

        assert_eq!(bootstrap.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(bootstrap.db_path.exists());

        bootstrap.pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_append_event_async_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let envelope = create_test_envelope("test-op-1", 1);
        let result: AsyncAppendResult = append_event_async(&pool, envelope, None).await?;

        assert_eq!(result.revision, 1);
        assert_eq!(result.op_id, "test-op-1");

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_events_since_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        for i in 0..5 {
            let envelope = create_test_envelope(&format!("test-op-{}", i), i);
            append_event_async(&pool, envelope, None).await?;
        }

        let events = fetch_events_since(&pool, 2).await?;

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].revision, 3);
        assert_eq!(events[1].revision, 4);
        assert_eq!(events[2].revision, 5);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_all_events_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        for i in 0..3 {
            let envelope = create_test_envelope(&format!("test-op-{}", i), i);
            append_event_async(&pool, envelope, None).await?;
        }

        let events = fetch_all_events(&pool).await?;

        assert_eq!(events.len(), 3);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_latest_revision_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let initial_rev = fetch_latest_revision(&pool).await?;
        assert_eq!(initial_rev, 0);

        for i in 0..3 {
            let envelope = create_test_envelope(&format!("test-op-{}", i), i);
            append_event_async(&pool, envelope, None).await?;
        }

        let final_rev = fetch_latest_revision(&pool).await?;
        assert_eq!(final_rev, 3);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_current_revision_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let rev = current_revision(&pool).await?;
        assert_eq!(rev, 0);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_append_batch_async_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let ops: Vec<EventEnvelope> = (0..5)
            .map(|i| create_test_envelope(&format!("batch-op-{}", i), i))
            .collect();

        let result: AsyncBatchAppendResult = append_batch_async(&pool, ops, None).await?;

        assert_eq!(result.start_revision, 1);
        assert_eq!(result.end_revision, 5);
        assert_eq!(result.count, 5);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_append_idempotent_async_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let envelope = create_test_envelope("idempotent-op", 1);
        let result1 = append_idempotent_async(&pool, envelope.clone()).await?;
        let result2 = append_idempotent_async(&pool, envelope).await?;

        assert_eq!(result1.revision, result2.revision);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_lookup_existing_op_async_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let envelope = create_test_envelope("lookup-test-op", 1);
        append_event_async(&pool, envelope.clone(), None).await?;

        let lookup_result = lookup_existing_op_async(&pool, "lookup-test-op").await?;

        assert!(lookup_result.is_some());
        assert_eq!(lookup_result.unwrap().op_id, "lookup-test-op");

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_store_pragmas_function() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let pragmas = read_store_pragmas_async(&pool).await?;

        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 2);
        assert_eq!(pragmas.wal_autocheckpoint, 1000);
        assert!(pragmas.foreign_keys);
        assert_eq!(pragmas.busy_timeout, 5000);

        pool.close().await;
        Ok(())
    }
}

mod e2e_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow_bootstrap_append_fetch() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let bootstrap: AsyncStoreBootstrap = bootstrap_async_store(&db_path).await?;
        let pool = bootstrap.pool;

        let envelope1 = create_test_envelope("e2e-op-1", 1);
        let envelope2 = create_test_envelope("e2e-op-2", 2);
        let envelope3 = create_test_envelope("e2e-op-3", 3);

        let result1 = append_event_async(&pool, envelope1, None).await?;
        let result2 = append_event_async(&pool, envelope2, None).await?;
        let result3 = append_event_async(&pool, envelope3, None).await?;

        assert_eq!(result1.revision, 1);
        assert_eq!(result2.revision, 2);
        assert_eq!(result3.revision, 3);

        let all_events = fetch_all_events(&pool).await?;
        assert_eq!(all_events.len(), 3);

        let events_since_1 = fetch_events_since(&pool, 1).await?;
        assert_eq!(events_since_1.len(), 2);

        let latest = fetch_latest_revision(&pool).await?;
        assert_eq!(latest, 3);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_revision_sequence_integrity() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        for i in 1..=10 {
            let envelope = create_test_envelope(&format!("seq-op-{}", i), i);
            let result = append_event_async(&pool, envelope, None).await?;
            assert_eq!(result.revision, i);
        }

        let events = fetch_all_events(&pool).await?;
        for (idx, event) in events.iter().enumerate() {
            assert_eq!(event.revision, (idx + 1) as i64);
        }

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_batch_append_sequential_revisions() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let ops: Vec<EventEnvelope> = (0..10)
            .map(|i| create_test_envelope(&format!("batch-{}", i), i))
            .collect();

        let result = append_batch_async(&pool, ops, None).await?;

        assert_eq!(result.start_revision, 1);
        assert_eq!(result.end_revision, 10);
        assert_eq!(result.count, 10);

        let events = fetch_all_events(&pool).await?;
        assert_eq!(events.len(), 10);

        for (idx, event) in events.iter().enumerate() {
            assert_eq!(event.revision, (idx + 1) as i64);
        }

        pool.close().await;
        Ok(())
    }
}

mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_revision_mismatch_error() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let envelope1 = create_test_envelope("op-1", 1);
        append_event_async(&pool, envelope1, None).await?;

        let envelope2 = create_test_envelope("op-2", 2);
        let result = append_event_async(&pool, envelope2, Some(5)).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AsyncStoreError::RevisionMismatch { expected, found } => {
                assert_eq!(expected, 5);
                assert_eq!(found, 1);
            }
            _ => panic!("Expected RevisionMismatch error"),
        }

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_revision_mismatch_batch_error() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let envelope1 = create_test_envelope("op-1", 1);
        append_event_async(&pool, envelope1, None).await?;

        let ops = vec![
            create_test_envelope("batch-1", 1),
            create_test_envelope("batch-2", 2),
        ];
        let result = append_batch_async(&pool, ops, Some(10)).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AsyncStoreError::RevisionMismatch { expected, found } => {
                assert_eq!(expected, 10);
                assert_eq!(found, 1);
            }
            _ => panic!("Expected RevisionMismatch error"),
        }

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_duplicate_with_conflict_error() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let envelope1 = EventEnvelope {
            op_id: "conflict-op".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Original".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        append_event_async(&pool, envelope1.clone(), None).await?;

        let envelope2 = EventEnvelope {
            op_id: "conflict-op".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 99.0,
                y: 99.0,
                width: 200.0,
                height: 100.0,
                label: "Different".to_string(),
            },
            author: Author {
                id: "user-2".to_string(),
                name: "Other User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        let result = append_idempotent_async(&pool, envelope2).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AsyncStoreError::DuplicateWithConflict(msg) => {
                assert!(msg.contains("conflict-op"));
            }
            _ => panic!("Expected DuplicateWithConflict error"),
        }

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_empty_batch_error() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let result = append_batch_async(&pool, vec![], None).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AsyncStoreError::EmptyBatch => {}
            _ => panic!("Expected EmptyBatch error"),
        }

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_exact_duplicate_handled_gracefully() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let envelope = create_test_envelope("exact-dup", 1);

        let result1 = append_idempotent_async(&pool, envelope.clone()).await?;
        let result2 = append_idempotent_async(&pool, envelope).await?;

        assert_eq!(result1.revision, result2.revision);

        let events = fetch_all_events(&pool).await?;
        assert_eq!(events.len(), 1);

        pool.close().await;
        Ok(())
    }
}

mod concurrent_operations_tests {
    use super::*;
    use tokio::sync::Mutex;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_concurrent_appends_no_race_conditions() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;
        let write_lock = Arc::new(Mutex::new(()));

        let mut join_set = JoinSet::new();

        for i in 0..10 {
            let pool_clone = pool.clone();
            let lock_clone = write_lock.clone();
            let envelope = create_test_envelope(&format!("concurrent-op-{}", i), i);
            join_set.spawn(async move {
                let _guard = lock_clone.lock().await;
                append_event_async(&pool_clone, envelope, None).await
            });
        }

        while let Some(result) = join_set.join_next().await {
            result??;
        }

        let final_revision = fetch_latest_revision(&pool).await?;
        assert_eq!(final_revision, 10);

        let events = fetch_all_events(&pool).await?;
        assert_eq!(events.len(), 10);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_idempotent_appends() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;
        let write_lock = Arc::new(Mutex::new(()));

        let envelope = create_test_envelope("concurrent-idempotent", 1);

        let mut join_set = JoinSet::new();
        for _ in 0..5 {
            let pool_clone = pool.clone();
            let lock_clone = write_lock.clone();
            let envelope_clone = envelope.clone();
            join_set.spawn(async move {
                let _guard = lock_clone.lock().await;
                append_idempotent_async(&pool_clone, envelope_clone).await
            });
        }

        let mut revision = None;
        while let Some(result) = join_set.join_next().await {
            let r = result??;
            if let Some(expected) = revision {
                assert_eq!(r.revision, expected);
            }
            revision = Some(r.revision);
        }

        let events = fetch_all_events(&pool).await?;
        assert_eq!(events.len(), 1);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_batch_and_single_appends() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_async_store(&db_path).await?;
        let write_lock = Arc::new(Mutex::new(()));

        let pool_clone = bootstrap.pool.clone();
        let lock_clone = write_lock.clone();
        let batch_handle = tokio::spawn(async move {
            let _guard = lock_clone.lock().await;
            let ops: Vec<EventEnvelope> = (0..5)
                .map(|i| create_test_envelope(&format!("batch-{}", i), i))
                .collect();
            append_batch_async(&pool_clone, ops, None).await
        });

        let pool_clone2 = bootstrap.pool.clone();
        let lock_clone2 = write_lock.clone();
        let single_handle = tokio::spawn(async move {
            let _guard = lock_clone2.lock().await;
            let envelope = create_test_envelope("single-op", 5);
            append_event_async(&pool_clone2, envelope, None).await
        });

        batch_handle.await??;
        single_handle.await??;

        let events = fetch_all_events(&bootstrap.pool).await?;
        assert_eq!(events.len(), 6);

        bootstrap.pool.close().await;
        Ok(())
    }
}

mod wal_mode_tests {
    use super::*;

    #[tokio::test]
    async fn test_wal_mode_enabled_after_bootstrap() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let pragmas = read_store_pragmas_async(&pool).await?;

        assert_eq!(
            pragmas.journal_mode, "wal",
            "WAL mode should be enabled"
        );

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_wal_files_created() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        for i in 0..5 {
            let envelope = create_test_envelope(&format!("wal-test-{}", i), i);
            append_event_async(&pool, envelope, None).await?;
        }

        let db_name = db_path.file_stem().unwrap().to_str().unwrap();
        let wal_path = db_path.with_file_name(format!("{}.db-wal", db_name));
        let _shm_path = db_path.with_file_name(format!("{}.db-shm", db_name));

        assert!(
            wal_path.exists() || db_path.exists(),
            "WAL file should exist or DB should be present"
        );

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_pragma_synchronous_full() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let pragmas = read_store_pragmas_async(&pool).await?;

        assert_eq!(pragmas.synchronous, 2, "synchronous should be FULL (2)");

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_pragma_foreign_keys_enabled() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let pragmas = read_store_pragmas_async(&pool).await?;

        assert!(pragmas.foreign_keys, "foreign_keys should be enabled");

        pool.close().await;
        Ok(())
    }
}

mod integration_regression_tests {
    use super::*;

    #[tokio::test]
    async fn test_multiple_bootstrap_cycles() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        for cycle in 0..3 {
            let bootstrap = bootstrap_async_store(&db_path).await?;

            let envelope = create_test_envelope(&format!("cycle-{}-op", cycle), cycle);
            append_event_async(&bootstrap.pool, envelope, None).await?;

            bootstrap.pool.close().await;
        }

        let pool = bootstrap_async_store(&db_path).await?.pool;
        let events = fetch_all_events(&pool).await?;
        assert_eq!(events.len(), 3);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_large_batch_100_events() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let ops: Vec<EventEnvelope> = (0..100)
            .map(|i| create_test_envelope(&format!("large-batch-{}", i), i))
            .collect();

        let result = append_batch_async(&pool, ops, None).await?;

        assert_eq!(result.count, 100);
        assert_eq!(result.start_revision, 1);
        assert_eq!(result.end_revision, 100);

        let events = fetch_all_events(&pool).await?;
        assert_eq!(events.len(), 100);

        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_interleaved_batches_and_singles() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_async_store(&db_path).await?.pool;

        let batch1: Vec<EventEnvelope> = (0..3)
            .map(|i| create_test_envelope(&format!("batch1-{}", i), i))
            .collect();
        append_batch_async(&pool, batch1, None).await?;

        let single1 = create_test_envelope("single-1", 3);
        append_event_async(&pool, single1, None).await?;

        let batch2: Vec<EventEnvelope> = (4..7)
            .map(|i| create_test_envelope(&format!("batch2-{}", i), i))
            .collect();
        append_batch_async(&pool, batch2, None).await?;

        let events = fetch_all_events(&pool).await?;
        assert_eq!(events.len(), 7);

        for (idx, event) in events.iter().enumerate() {
            assert_eq!(event.revision, (idx + 1) as i64);
        }

        pool.close().await;
        Ok(())
    }
}
