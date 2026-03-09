use super::*;


#[test]
fn test_append_batch_atomicity_on_failure() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // First, add an event that will cause a duplicate conflict later
    let first_event = EventEnvelope {
        op_id: "duplicate-op".to_string(),
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

    // Now try to add a batch with a duplicate op_id (will fail due to unique constraint)
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
            op_id: "duplicate-op".to_string(), // This will cause a failure
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
    // The batch should fail due to the duplicate
    assert!(result.is_err(), "Expected error for duplicate op_id");

    // Verify that no events were added (atomicity)
    let count: i64 = bootstrap
        .conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .expect("Failed to count events");
    assert_eq!(count, 1, "Only the first event should exist (atomicity)");
}

#[test]
fn test_verify_batch_atomicity_valid() {
    let result = BatchAppendResult {
        start_revision: 1,
        end_revision: 3,
        count: 3,
        op_ids: vec!["op-1".to_string(), "op-2".to_string(), "op-3".to_string()],
        last_timestamp: 1700000003,
    };

    let verification = verify_batch_atomicity(&result);
    assert!(
        verification.is_ok(),
        "Expected Ok, got: {:?}",
        verification.err()
    );
}

#[test]
fn test_verify_batch_atomicity_invalid_start_revision() {
    let result = BatchAppendResult {
        start_revision: 0,
        end_revision: 2,
        count: 3,
        op_ids: vec!["op-1".to_string(), "op-2".to_string(), "op-3".to_string()],
        last_timestamp: 1700000003,
    };

    let verification = verify_batch_atomicity(&result);
    assert!(verification.is_err());

    match verification {
        Err(StoreError::ValidationFailed(msg)) => {
            assert!(msg.contains("start_revision"));
        }
        Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[test]
fn test_verify_batch_atomicity_invalid_revision_range() {
    let result = BatchAppendResult {
        start_revision: 5,
        end_revision: 3, // end < start
        count: 0,
        op_ids: vec![],
        last_timestamp: 1700000003,
    };

    let verification = verify_batch_atomicity(&result);
    assert!(verification.is_err());

    match verification {
        Err(StoreError::ValidationFailed(msg)) => {
            assert!(msg.contains("end_revision"));
        }
        Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[test]
fn test_verify_batch_atomicity_count_mismatch() {
    let result = BatchAppendResult {
        start_revision: 1,
        end_revision: 3,
        count: 5, // Should be 3
        op_ids: vec!["op-1".to_string(), "op-2".to_string(), "op-3".to_string()],
        last_timestamp: 1700000003,
    };

    let verification = verify_batch_atomicity(&result);
    assert!(verification.is_err());

    match verification {
        Err(StoreError::ValidationFailed(msg)) => {
            assert!(msg.contains("count"));
        }
        Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[test]
fn test_verify_batch_atomicity_empty_op_id() {
    let result = BatchAppendResult {
        start_revision: 1,
        end_revision: 2,
        count: 2,
        op_ids: vec!["op-1".to_string(), "".to_string()], // Empty op_id
        last_timestamp: 1700000002,
    };

    let verification = verify_batch_atomicity(&result);
    assert!(verification.is_err());

    match verification {
        Err(StoreError::ValidationFailed(msg)) => {
            assert!(msg.contains("op_id"));
        }
        Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[test]
fn test_verify_batch_atomicity_invalid_timestamp() {
    let result = BatchAppendResult {
        start_revision: 1,
        end_revision: 1,
        count: 1,
        op_ids: vec!["op-1".to_string()],
        last_timestamp: 0, // Invalid timestamp
    };

    let verification = verify_batch_atomicity(&result);
    assert!(verification.is_err());

    match verification {
        Err(StoreError::ValidationFailed(msg)) => {
            assert!(msg.contains("last_timestamp"));
        }
        Err(other) => panic!("Expected ValidationFailed, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[test]
fn test_append_batch_single_event() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let events = vec![EventEnvelope {
        op_id: "single-op".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Single Node".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000001,
    }];

    let result = append_batch(&mut bootstrap.conn, events, None);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

    let batch_result = result.unwrap();
    assert_eq!(batch_result.start_revision, 1);
    assert_eq!(batch_result.end_revision, 1);
    assert_eq!(batch_result.count, 1);
    assert_eq!(batch_result.op_ids, vec!["single-op"]);
}

// OCC idempotency regression tests (bd-ahf)

// Regression test: stale revision must be rejected with no append
//
// This test verifies that when a client attempts to append with an
// outdated (stale) expected revision, the operation is rejected
// with RevisionMismatch and no event is appended.
