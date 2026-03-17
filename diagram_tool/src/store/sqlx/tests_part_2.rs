#[cfg(test)]
mod tests_part_2 {
use super::*;
use tempfile::TempDir;
use diagram_models::envelope::EventEnvelope;

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_integrity_check() {
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
        append_event(&bootstrap.pool, envelope, None)
            .await
            .expect("append failed");

        bootstrap.pool.close().await;

        let status = integrity_check(&db_path)
            .await
            .expect("integrity_check failed");

        assert!(status.is_valid);
        assert_eq!(status.schema_version, Some(1));
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_save_snapshot_returns_meta_with_correct_revision() {
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
        let meta = save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");
        assert_eq!(meta.revision, 5);

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_load_projection_from_snapshot_replays_tail() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=3 {
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

        let projection_rev3 = diagram_models::projection::DiagramProjection::with_revision(3);
        save_snapshot(&pool, &projection_rev3)
            .await
            .expect("save_snapshot failed");

        for i in 4..=5 {
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

        let loaded = load_projection_from_snapshot(&pool)
            .await
            .expect("load_projection_from_snapshot failed");
        assert_eq!(loaded.revision, 5);

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_get_latest_snapshot_meta_returns_correct_data() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=10 {
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

        let projection = diagram_models::projection::DiagramProjection::with_revision(10);
        save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");

        let meta = get_latest_snapshot_meta(&pool)
            .await
            .expect("get_latest_snapshot_meta failed");
        assert!(meta.is_some());
        let meta = meta.expect("snapshot exists");
        assert_eq!(meta.revision, 10);

        pool.close().await;
    }

}
