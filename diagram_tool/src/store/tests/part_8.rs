use super::*;


#[test]
fn test_append_batch_with_valid_events() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let events = vec![
        EventEnvelope {
            op_id: "batch-op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Node 1".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        },
        EventEnvelope {
            op_id: "batch-op-2".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 30.0,
                y: 40.0,
                width: 100.0,
                height: 50.0,
                label: "Node 2".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000002,
        },
        EventEnvelope {
            op_id: "batch-op-3".to_string(),
            operation: DomainOp::EdgeConnect {
                id: "edge-1".to_string(),
                source: "node-1".to_string(),
                target: "node-2".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000003,
        },
    ];

    let result = append_batch(&mut bootstrap.conn, events, None);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.start_revision, 1);
    assert_eq!(batch_result.end_revision, 3);
    assert_eq!(batch_result.count, 3);
    assert_eq!(batch_result.op_ids.len(), 3);
    assert_eq!(batch_result.last_timestamp, 1700000003);
}

#[test]
fn test_append_batch_empty_returns_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    let result = append_batch(&mut bootstrap.conn, vec![], None);
    assert!(result.is_err());

    match result {
        Err(StoreError::EmptyBatch) => {}
        Err(other) => panic!("Expected EmptyBatch error, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[test]
fn test_append_batch_with_revision_mismatch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let events = vec![EventEnvelope {
        op_id: "batch-op-1".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Node 1".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000001,
    }];

    // Expect revision 5, but actual is 0
    let result = append_batch(&mut bootstrap.conn, events, Some(5));
    assert!(result.is_err());

    match result {
        Err(StoreError::RevisionMismatch { expected, found }) => {
            assert_eq!(expected, 5);
            assert_eq!(found, 0);
        }
        Err(other) => panic!("Expected RevisionMismatch error, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[test]
fn test_append_batch_with_valid_expected_revision() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // First, add a single event to get to revision 1
    let first_event = EventEnvelope {
        op_id: "first-op".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-0".to_string(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            label: "Node 0".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };
    let first_result = append_event(&mut bootstrap.conn, first_event, None);
    assert!(first_result.is_ok());
    assert_eq!(first_result.unwrap().revision, 1);

    // Now add a batch with expected revision 1
    let events = vec![
        EventEnvelope {
            op_id: "batch-op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Node 1".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        },
        EventEnvelope {
            op_id: "batch-op-2".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 30.0,
                y: 40.0,
                width: 100.0,
                height: 50.0,
                label: "Node 2".to_string(),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000002,
        },
    ];

    let result = append_batch(&mut bootstrap.conn, events, Some(1));
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.start_revision, 2);
    assert_eq!(batch_result.end_revision, 3);
    assert_eq!(batch_result.count, 2);
}
