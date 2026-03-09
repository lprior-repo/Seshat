use super::*;


#[test]
fn test_classify_duplicate_conflict() {
    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let envelope1 = EventEnvelope {
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

    let payload1 = encode_event_envelope(&envelope1).expect("Failed to encode envelope");

    let record = EventRecord {
        op_id: OpId::new("op-1".to_string()).unwrap(),
        revision: StoreRevision::new(1).unwrap(),
        timestamp: Timestamp::new(1700000000).unwrap(),
        payload: payload1,
    };

    // Different envelope with same op_id but different payload
    let envelope2 = EventEnvelope {
        op_id: "op-1".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 999.0, // Different x coordinate
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

    let kind = classify_duplicate(&record, &envelope2);
    assert!(
        kind.is_ok(),
        "classify_duplicate should succeed: {:?}",
        kind.err()
    );
    assert_eq!(kind.expect("checked is_ok"), DuplicateKind::Conflict);
}

#[test]
fn test_append_idempotent_new_operation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let envelope = EventEnvelope {
        op_id: "op-new".to_string(),
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

    let result = append_idempotent(&mut bootstrap.conn, envelope);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let outcome = result.expect("checked is_ok");
    assert_eq!(
        outcome.revision, 1,
        "New operation should create revision 1"
    );
    assert_eq!(outcome.op_id, "op-new");
}

#[test]
fn test_append_idempotent_exact_duplicate_returns_existing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let envelope = EventEnvelope {
        op_id: "op-exact".to_string(),
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

    // First append
    let result1 = append_idempotent(&mut bootstrap.conn, envelope.clone());
    assert!(
        result1.is_ok(),
        "First append should succeed: {:?}",
        result1.err()
    );
    let outcome1 = result1.expect("checked is_ok");
    assert_eq!(outcome1.revision, 1);

    // Second append with exact duplicate
    let result2 = append_idempotent(&mut bootstrap.conn, envelope);
    assert!(
        result2.is_ok(),
        "Exact duplicate should return Ok: {:?}",
        result2.err()
    );
    let outcome2 = result2.expect("checked is_ok");

    // Should return existing outcome (no-op)
    assert_eq!(
        outcome2.revision, outcome1.revision,
        "Revision should be unchanged for exact duplicate"
    );
    assert_eq!(outcome2.op_id, outcome1.op_id);
    assert_eq!(outcome2.timestamp, outcome1.timestamp);

    // Verify only one row in database
    let count: i64 = bootstrap
        .conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE operation_id = 'op-exact'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count events");
    assert_eq!(count, 1, "Should have exactly one row for exact duplicate");
}

#[test]
fn test_append_idempotent_conflicting_duplicate_returns_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let envelope1 = EventEnvelope {
        op_id: "op-conflict".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Original".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };

    // First append
    let result1 = append_idempotent(&mut bootstrap.conn, envelope1);
    assert!(
        result1.is_ok(),
        "First append should succeed: {:?}",
        result1.err()
    );

    // Second append with conflicting payload (same op_id, different content)
    let envelope2 = EventEnvelope {
        op_id: "op-conflict".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 999.0, // Different x coordinate
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Modified".to_string(), // Different label
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };

    let result2 = append_idempotent(&mut bootstrap.conn, envelope2);
    assert!(
        result2.is_err(),
        "Conflicting duplicate should return error"
    );
    match result2 {
        Err(StoreError::DuplicateWithConflict(op_id)) => {
            assert_eq!(op_id, "op-conflict");
        }
        Err(e) => panic!("Expected DuplicateWithConflict error, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }
}
