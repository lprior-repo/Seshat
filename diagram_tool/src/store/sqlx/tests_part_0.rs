#[cfg(test)]
mod tests_part_0 {
use super::*;
use tempfile::TempDir;
use diagram_models::envelope::EventEnvelope;

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_create_pool() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.expect("Failed to create pool");

        // Verify pragmas are set correctly
        let pragmas = read_store_pragmas(&pool)
            .await
            .expect("Failed to read pragmas");

        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 1); // NORMAL = 1
        assert_eq!(pragmas.wal_autocheckpoint, 1000);
        assert!(pragmas.foreign_keys);
        assert_eq!(pragmas.busy_timeout, 5000);

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_bootstrap_store() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store");

        assert_eq!(bootstrap.schema_version, CURRENT_SCHEMA_VERSION);

        bootstrap.pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_append_event() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: diagram_models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: diagram_models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result = append_event(&pool, envelope, None)
            .await
            .expect("Failed to append event");

        assert_eq!(result.revision, 1);
        assert_eq!(result.op_id, "test-op-1");

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_append_idempotent() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: diagram_models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: diagram_models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result1 = append_idempotent(&pool, envelope.clone())
            .await
            .expect("Failed to append first");

        let result2 = append_idempotent(&pool, envelope)
            .await
            .expect("Failed to append second (should be exact duplicate)");

        assert_eq!(result1.revision, result2.revision);
        assert_eq!(result1.op_id, result2.op_id);

        let events = fetch_all_events(&pool).await.expect("Failed to fetch all");
        assert_eq!(events.len(), 1);

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_fetch_events_since() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        // Add some events
        for i in 0..5 {
            let envelope = EventEnvelope {
                op_id: format!("test-op-{}", i),
                operation: diagram_models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{}", i),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Test Node {}", i),
                },
                author: diagram_models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, None)
                .await
                .expect("Failed to append");
        }

        let events = fetch_events_since(&pool, 2).await.expect("Failed to fetch");
        assert_eq!(events.len(), 3); // revisions 3, 4, 5

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_store_exports_bootstrap_store() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        assert_eq!(bootstrap.schema_version, 1);
        assert_eq!(bootstrap.db_path, db_path);

        bootstrap.pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_store_exports_append_event() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: diagram_models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: diagram_models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result = append_event(&bootstrap.pool, envelope, None)
            .await
            .expect("append_event failed");

        assert_eq!(result.revision, 1);
        assert_eq!(result.op_id, "test-op-1");

        bootstrap.pool.close().await;
    }

}
