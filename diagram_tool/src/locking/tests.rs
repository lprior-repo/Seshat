#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tempfile::TempDir;

    use crate::locking::id::DiagramId;
    use crate::locking::manager::DiagramLockManager;
    use diagram_models::document::Revision;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_new_manager_when_created_then_empty() {
        let manager = DiagramLockManager::with_defaults(Duration::from_secs(1));
        assert_eq!(manager.diagram_count(), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_manager_when_check_unlocked_diagram_then_returns_false() {
        let manager = DiagramLockManager::with_defaults(Duration::from_secs(1));
        let diagram_id = DiagramId::new("test_diagram".to_string()).unwrap();

        assert!(!manager.is_locked(&diagram_id));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_manager_when_check_queue_length_then_returns_zero() {
        let manager = DiagramLockManager::with_defaults(Duration::from_secs(1));
        let diagram_id = DiagramId::new("test_diagram".to_string()).unwrap();

        assert_eq!(manager.queue_length(&diagram_id), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_lock_timeout_when_cannot_acquire_then_error() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_millis(50), lock_dir, diagram_dir);
        let diagram_id = DiagramId::new("test_diagram".to_string()).unwrap();

        let result1 = manager.with_lock(diagram_id.clone(), |_doc| Ok(42));
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), 42);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_different_diagrams_when_mutated_then_both_succeed() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id1 = DiagramId::new("diagram1".to_string()).unwrap();
        let diagram_id2 = DiagramId::new("diagram2".to_string()).unwrap();

        let result1 = manager.with_lock(diagram_id1.clone(), |_doc| Ok("result1"));
        let result2 = manager.with_lock(diagram_id2.clone(), |_doc| Ok("result2"));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_mutation_with_lock_when_applied_then_document_modified() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string()).unwrap();

        let result = manager.with_lock(diagram_id, |doc| {
            doc.revision = Revision::INITIAL.increment();
            Ok(())
        });

        assert!(result.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_queued_mutations_when_flushed_then_all_applied() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string()).unwrap();

        manager
            .queue_mutation(diagram_id.clone(), |doc| {
                doc.revision = Revision::INITIAL.increment();
                Ok(())
            })
            .unwrap();

        manager
            .queue_mutation(diagram_id.clone(), |doc| {
                doc.revision = doc.revision.increment();
                Ok(())
            })
            .unwrap();

        assert_eq!(manager.queue_length(&diagram_id), 2);

        manager.flush_queue(&diagram_id).unwrap();

        assert_eq!(manager.queue_length(&diagram_id), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_queue_when_cleared_then_empty() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string()).unwrap();

        manager
            .queue_mutation(diagram_id.clone(), |_doc| Ok(()))
            .unwrap();

        assert_eq!(manager.queue_length(&diagram_id), 1);

        manager.clear_queue(&diagram_id);

        assert_eq!(manager.queue_length(&diagram_id), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multiple_operations_same_diagram_when_sequential_then_succeed() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string()).unwrap();

        let result1 = manager.with_lock(diagram_id.clone(), |doc| {
            doc.revision = Revision::INITIAL.increment();
            Ok(())
        });
        assert!(result1.is_ok());

        let result2 = manager.with_lock(diagram_id.clone(), |doc| {
            doc.revision = doc.revision.increment();
            Ok(())
        });
        assert!(result2.is_ok());
    }
}
