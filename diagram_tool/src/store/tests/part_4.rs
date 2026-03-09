use super::*;


#[test]
fn test_append_with_occ_success() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};
    let envelope = EventEnvelope {
        op_id: "op-occ-1".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 100.0,
            y: 200.0,
            width: 80.0,
            height: 40.0,
            label: "Test Node".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };

    let result = append_with_occ(&mut bootstrap.conn, envelope, None);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let outcome = result.expect("Checked is_ok");
    assert_eq!(outcome.revision, 1);
    assert_eq!(outcome.op_id, "op-occ-1");
}

#[test]
fn test_append_with_occ_revision_mismatch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};
    let envelope = EventEnvelope {
        op_id: "op-occ-2".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 100.0,
            y: 200.0,
            width: 80.0,
            height: 40.0,
            label: "Test Node".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };

    // Expect revision 5 but database is at 0
    let result = append_with_occ(&mut bootstrap.conn, envelope, Some(5));
    assert!(result.is_err());
    match result {
        Err(StoreError::RevisionMismatch { expected, found }) => {
            assert_eq!(expected, 5);
            assert_eq!(found, 0);
        }
        _ => panic!("Expected RevisionMismatch error"),
    }
}

#[test]
fn test_verify_occ_append_valid_result() {
    let result = AppendResult {
        revision: StoreRevision::new(1).unwrap(),
        op_id: OpId::new("op-valid".to_string()).unwrap(),
        timestamp: Timestamp::new(1700000000).unwrap(),
    };

    assert!(verify_occ_append(&result).is_ok());
}

// current_revision and next_revision tests

#[test]
fn test_current_revision_empty_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Empty database should return 0
    let revision = current_revision(&bootstrap.conn).expect("Failed to get current revision");
    assert_eq!(revision, 0, "Empty database should have revision 0");
}

#[test]
fn test_current_revision_with_events() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Add an event
    let envelope = EventEnvelope {
        op_id: "op-1".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Test".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };
    let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

    // Should return 1 after one event
    let revision = current_revision(&bootstrap.conn).expect("Failed to get current revision");
    assert_eq!(revision, 1);
}

#[test]
fn test_current_revision_multiple_events() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Add multiple events
    for i in 1..=5 {
        let envelope = EventEnvelope {
            op_id: format!("op-{i}"),
            operation: DomainOp::NodeAdd {
                id: format!("node-{i}"),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: format!("Node {i}"),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000 + i,
        };
        let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");
    }

    // Should return 5 after five events
    let revision = current_revision(&bootstrap.conn).expect("Failed to get current revision");
    assert_eq!(revision, 5);
}

#[test]
fn test_next_revision_empty_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Empty database: current=0, next=1
    let revision = next_revision(&bootstrap.conn).expect("Failed to get next revision");
    assert_eq!(revision, 1, "Next revision should be 1 for empty database");
}

#[test]
fn test_next_revision_with_events() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Add an event
    let envelope = EventEnvelope {
        op_id: "op-1".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Test".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };
    let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

    // After one event: current=1, next=2
    let revision = next_revision(&bootstrap.conn).expect("Failed to get next revision");
    assert_eq!(revision, 2);
}

#[test]
fn test_next_revision_monotonic_increase() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Verify monotonic increase across multiple appends
    for i in 1..=3 {
        let next_before = next_revision(&bootstrap.conn).expect("Failed to get next revision");
        assert_eq!(next_before, i);

        let envelope = EventEnvelope {
            op_id: format!("op-{i}"),
            operation: DomainOp::NodeAdd {
                id: format!("node-{i}"),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: format!("Node {i}"),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000 + i,
        };
        let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

        let current_after =
            current_revision(&bootstrap.conn).expect("Failed to get current revision");
        assert_eq!(current_after, i);
    }
}

// RevisionGap error tests
