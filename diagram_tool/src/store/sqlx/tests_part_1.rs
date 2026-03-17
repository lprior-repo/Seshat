#[cfg(test)]
mod tests_part_1 {
use super::*;
use tempfile::TempDir;
use diagram_models::envelope::EventEnvelope;

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_store_exports_fetch_latest_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let initial = fetch_latest_revision(&bootstrap.pool)
            .await
            .expect("fetch_latest_revision failed");
        assert_eq!(initial, 0);

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

        let after = fetch_latest_revision(&bootstrap.pool)
            .await
            .expect("fetch_latest_revision failed");
        assert_eq!(after, 1);

        bootstrap.pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_store_exports_read_store_pragmas() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let pragmas = read_store_pragmas(&bootstrap.pool)
            .await
            .expect("read_store_pragmas failed");

        assert_eq!(pragmas.journal_mode, "wal");
        assert_eq!(pragmas.synchronous, 1);
        assert_eq!(pragmas.wal_autocheckpoint, 1000);
        assert!(pragmas.foreign_keys);
        assert_eq!(pragmas.busy_timeout, 5000);

        bootstrap.pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_store_exports_current_store_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = bootstrap_store(&db_path)
            .await
            .expect("bootstrap_store failed");

        let config = current_store_config(&bootstrap.pool)
            .await
            .expect("current_store_config failed");

        assert_eq!(config.pragmas.journal_mode, "wal");
        assert_eq!(config.schema_version, 1);

        bootstrap.pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_startup_integrity_check_valid_db() {
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

        let status = startup_integrity_check(&db_path)
            .await
            .expect("startup_integrity_check failed");

        assert!(status.is_valid);
        assert!(status.error_message.is_none());
        assert_eq!(status.schema_version, Some(1));
        assert_eq!(status.event_count, 1);
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_startup_integrity_check_nonexistent_db() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let nonexistent_path = temp_dir.path().join("nonexistent.db");

        let status = startup_integrity_check(&nonexistent_path)
            .await
            .expect("startup_integrity_check failed");

        assert!(!status.is_valid);
        assert!(status.error_message.is_some());
        assert_eq!(status.page_count, 0);
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_open_recovery_mode() {
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

        let handle = open_recovery_mode(&db_path)
            .await
            .expect("open_recovery_mode failed");

        assert_eq!(handle.db_path, db_path);

        handle.pool.close().await;
    }

#[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn test_phase2_open_recovery_only() {
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

        let session = open_recovery_only(&db_path)
            .await
            .expect("open_recovery_only failed");

        assert_eq!(session.db_path, db_path);

        session.pool.close().await;
    }

}
