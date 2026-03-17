#[cfg(test)]
mod tests_part_3 {
use super::*;
use tempfile::TempDir;
use diagram_models::envelope::EventEnvelope;

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_delete_snapshot_removes_record() {
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

        delete_snapshot(&pool, 5)
            .await
            .expect("delete_snapshot failed");

        let meta = get_latest_snapshot_meta(&pool)
            .await
            .expect("get_latest_snapshot_meta failed");
        assert!(meta.is_none());

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_list_snapshots_returns_all_snapshots() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        for i in 1..=8 {
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

            if i == 2 || i == 5 || i == 8 {
                let projection =
                use diagram_models::projection::DiagramProjection::with_revision(i as u64);
                save_snapshot(&pool, &projection)
                    .await
                    .expect("save_snapshot failed");
            }
        }

        let snapshots = list_snapshots(&pool).await.expect("list_snapshots failed");
        assert_eq!(snapshots.len(), 3);

        let revisions: Vec<i64> = snapshots.iter().map(|s| s.revision).collect();
        assert_eq!(revisions, vec![8, 5, 2]);

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_save_snapshot_fails_with_stale_projection() {
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

        let stale_projection = diagram_models::projection::DiagramProjection::with_revision(5);
        let result = save_snapshot(&pool, &stale_projection).await;
        assert!(matches!(
            result,
            Err(StoreError::SnapshotStale {
                expected: 10,
                found: 5
            })
        ));

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_load_projection_from_snapshot_falls_back_to_replay_when_no_snapshots() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        // When no snapshot exists, it should fall back to replay from empty projection
        let result = load_projection_from_snapshot(&pool).await;
        // Now returns Ok with empty projection (replays all events from revision 0)
        assert!(result.is_ok());
        let projection = result.expect("should return projection");
        assert_eq!(projection.revision, 0);
        assert!(projection.nodes.is_empty());
        assert!(projection.edges.is_empty());

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_delete_snapshot_fails_when_revision_not_found() {
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

        let result = delete_snapshot(&pool, 99).await;
        assert!(matches!(result, Err(StoreError::NotFound(_))));

        pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_save_snapshot_at_revision_zero_succeeds() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path).await.expect("bootstrap failed");
        let pool = bootstrap.pool;

        let projection = diagram_models::projection::DiagramProjection::empty();
        let meta = save_snapshot(&pool, &projection)
            .await
            .expect("save_snapshot failed");
        assert_eq!(meta.revision, 0);

        pool.close().await;
    }

}
