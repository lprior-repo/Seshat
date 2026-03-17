#[cfg(test)]
mod tests_part_4 {
use super::*;
use tempfile::TempDir;
use diagram_models::envelope::EventEnvelope;

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_load_projection_with_no_tail_events() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: diagram_models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
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
                .expect("append failed");
        }

        let projection = diagram_models::projection::DiagramProjection::with_revision(5);
        save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");

        let loaded = load_projection_from_snapshot(&pool)
            .await
            .expect("load_projection_from_snapshot failed");
        assert_eq!(loaded.revision, 5);

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_multiple_snapshots_same_revision_replaces() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=5 {
            let envelope = EventEnvelope {
                op_id: format!("op-{i}"),
                operation: diagram_models::envelope::DomainOp::NodeAdd {
                    id: format!("node-{i}"),
                    x: 10.0 * i as f64,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    label: format!("Node {i}"),
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
                .expect("append failed");
        }

        let projection1 = diagram_models::projection::DiagramProjection::with_revision(5);
        let meta1 = save_snapshot(&pool, &projection1)
            .await
            .expect("save_snapshot failed");

        let projection2 = diagram_models::projection::DiagramProjection::with_revision(5);
        let meta2 = save_snapshot(&pool, &projection2)
            .await
            .expect("save_snapshot failed");

        let snapshots = list_snapshots(&pool).await.expect("list_snapshots failed");
        assert_eq!(snapshots.len(), 1);

        let latest = get_latest_snapshot_meta(&pool)
            .await
            .expect("get_latest_snapshot_meta failed")
            .expect("snapshot exists");
        assert_eq!(latest.id, meta2.id);
        assert_ne!(latest.id, meta1.id);

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_delete_snapshot_fails_with_negative_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        let result = delete_snapshot(&pool, -1).await;
        assert!(matches!(result, Err(StoreError::InvalidInput(_))));

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_returns_error_when_invalid_db_path_provided() {
        let invalid_path = Path::new("/nonexistent directory that does not exist/test.db");

        let result = bootstrap_store(invalid_path).await;
        assert!(matches!(result, Err(StoreError::Sqlx(_))));
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_returns_error_when_appending_with_revision_gap() {
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

        let result = append_event(&pool, envelope, Some(5)).await;
        assert!(matches!(
            result,
            Err(StoreError::RevisionMismatch {
                expected: 5,
                found: 0
            })
        ));

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_handles_concurrent_async_appends_gracefully() {
        use tokio::task::JoinSet;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let pool = bootstrap_store(&db_path)
            .await
            .expect("Failed to bootstrap async store")
            .pool;

        let mut join_set = JoinSet::new();

        for i in 0..10 {
            let pool = pool.clone();
            join_set.spawn(async move {
                let envelope = EventEnvelope {
                    op_id: format!("concurrent-op-{}", i),
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
                append_event(&pool, envelope, None).await
            });
        }

        let mut success_count = 0;
        while let Some(result) = join_set.join_next().await {
            if matches!(result, Ok(Ok(_))) {
                success_count += 1;
            }
        }

        assert!(
            success_count > 0,
            "At least some concurrent appends should succeed, got {}",
            success_count
        );

        let final_revision = current_revision(&pool)
            .await
            .expect("Failed to get revision");

        assert_eq!(
            final_revision, success_count as i64,
            "Revision should match successful appends"
        );

        pool.close().await;
    }

}
