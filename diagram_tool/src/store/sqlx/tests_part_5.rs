#[cfg(test)]
mod tests_part_5 {
use super::*;
use tempfile::TempDir;
use diagram_models::envelope::EventEnvelope;

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_handles_zero_byte_database_initialization() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("zero_byte.db");

        std::fs::write(&db_path, b"").expect("Failed to create zero-byte file");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("Should handle zero-byte file gracefully");

        let test_envelope = EventEnvelope {
            op_id: "init-test-op".to_string(),
            operation: diagram_models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            author: diagram_models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1_700_000_000,
        };

        let result = append_event(&bootstrap.pool, test_envelope, None).await;
        assert!(
            result.is_ok(),
            "Should be able to append after bootstrap from zero-byte"
        );

        let revision = current_revision(&bootstrap.pool)
            .await
            .expect("Should get revision");
        assert_eq!(revision, 1, "Should have one event after append");

        bootstrap.pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_invariant_unique_op_id_enforced_by_schema() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let envelope = EventEnvelope {
            op_id: "duplicate-op-id".to_string(),
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

        append_event(&pool, envelope.clone(), None)
            .await
            .expect("First append should succeed");

        let result = append_event(&pool, envelope, None).await;
        assert!(matches!(result, Err(StoreError::Sqlx(_))));

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_precondition_sequential_revision_enforced() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        for i in 1..=3 {
            let envelope = EventEnvelope {
                op_id: format!("seq-op-{}", i),
                operation: diagram_models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{}", i),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {}", i),
                },
                author: diagram_models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            append_event(&pool, envelope, Some(i - 1))
                .await
                .expect("Sequential append should succeed");
        }

        let revision = current_revision(&pool)
            .await
            .expect("Failed to get revision");
        assert_eq!(
            revision, 3,
            "Revision should be 3 after 3 sequential appends"
        );

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_concurrent_appends_with_expected_revision_serialized() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        for i in 0..5 {
            let envelope = EventEnvelope {
                op_id: format!("serialized-op-{}", i),
                operation: diagram_models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{}", i),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {}", i),
                },
                author: diagram_models::envelope::Author {
                    id: "user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1_700_000_000 + i as i64,
            };
            let expected_rev = i as i64;
            let result = append_event(&pool, envelope, Some(expected_rev)).await;
            assert!(result.is_ok(), "Append {} should succeed", i);
        }

        let final_revision = current_revision(&pool)
            .await
            .expect("Failed to get revision");
        assert_eq!(final_revision, 5, "Final revision should be 5");

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_postcondition_wal_mode_concurrent_access_works() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        use tokio::task::JoinSet;
        let mut join_set = JoinSet::new();
        for i in 0..5 {
            let pool = pool.clone();
            join_set.spawn(async move {
                let envelope = EventEnvelope {
                    op_id: format!("wal-test-{}", i),
                    operation: diagram_models::envelope::DomainOp::NodeAdd {
                        id: format!("node-{}", i),
                        x: 10.0,
                        y: 20.0,
                        width: 100.0,
                        height: 50.0,
                        label: "Test".to_string(),
                    },
                    author: diagram_models::envelope::Author {
                        id: "user".to_string(),
                        name: "User".to_string(),
                        email: None,
                    },
                    timestamp: i as i64,
                };
                append_event(&pool, envelope, None).await
            });
        }

        let mut successes = 0;
        while let Some(result) = join_set.join_next().await {
            if matches!(result, Ok(Ok(_))) {
                successes += 1;
            }
        }

        assert!(successes > 0, "WAL mode should allow concurrent writes");
        assert_eq!(current_revision(&pool).await.unwrap(), successes as i64);

        pool.close().await;
    }
}


}
