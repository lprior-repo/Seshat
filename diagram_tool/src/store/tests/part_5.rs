use super::*;


#[test]
fn test_revision_gap_error_display() {
    let err = StoreError::RevisionGap {
        expected: 5,
        found: 7,
    };
    let msg = err.to_string();
    assert!(msg.contains("Revision gap detected"));
    assert!(msg.contains("expected sequential revision 5"));
    assert!(msg.contains("gap at 7"));
}

#[test]
fn test_map_error_code_revision_gap() {
    let err = StoreError::RevisionGap {
        expected: 5,
        found: 7,
    };
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::RevisionMismatch);
}

// ensure_op_id_uniqueness and lookup_existing_op tests (bd-1ua)

#[test]
fn test_ensure_op_id_uniqueness_creates_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Should succeed without error
    let result = ensure_op_id_uniqueness(&mut bootstrap.conn);
    assert!(
        result.is_ok(),
        "ensure_op_id_uniqueness should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_ensure_op_id_uniqueness_is_idempotent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Call twice - should be idempotent
    let result1 = ensure_op_id_uniqueness(&mut bootstrap.conn);
    assert!(result1.is_ok());

    let result2 = ensure_op_id_uniqueness(&mut bootstrap.conn);
    assert!(result2.is_ok(), "Second call should also succeed");
}

#[test]
fn test_lookup_existing_op_returns_none_for_nonexistent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    let result = lookup_existing_op(&bootstrap.conn, "nonexistent-op-id");
    assert!(result.is_ok(), "lookup should succeed: {:?}", result.err());
    assert!(
        result.expect("checked is_ok").is_none(),
        "Should return None for nonexistent op_id"
    );
}

#[test]
fn test_lookup_existing_op_returns_record_for_existing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Add an event
    let envelope = EventEnvelope {
        op_id: "op-lookup-test".to_string(),
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

    // Lookup should find it
    let result = lookup_existing_op(&bootstrap.conn, "op-lookup-test");
    assert!(result.is_ok(), "lookup should succeed: {:?}", result.err());
    let record = result.expect("checked is_ok").expect("should find record");
    assert_eq!(record.op_id, "op-lookup-test");
    assert_eq!(record.revision, 1);
    assert_eq!(record.timestamp, 1700000000);
}

#[test]
fn test_duplicate_op_id_rejected_by_unique_constraint() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let op_id = "op-duplicate-constraint";

    // Add first event
    let envelope1 = EventEnvelope {
        op_id: op_id.to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "First".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };
    let result1 = append_event(&mut bootstrap.conn, envelope1, None);
    assert!(result1.is_ok(), "First append should succeed");

    // Try duplicate op_id - should fail
    let envelope2 = EventEnvelope {
        op_id: op_id.to_string(), // Same op_id
        operation: DomainOp::NodeAdd {
            id: "node-2".to_string(),
            x: 20.0,
            y: 30.0,
            width: 100.0,
            height: 50.0,
            label: "Second".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000001,
    };
    let result2 = append_event(&mut bootstrap.conn, envelope2, None);
    assert!(result2.is_err(), "Duplicate op_id should be rejected");
}

#[test]
fn test_duplicate_with_conflict_error_display() {
    let err = StoreError::DuplicateWithConflict("op-123".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Duplicate op_id"));
    assert!(msg.contains("op-123"));
}

#[test]
fn test_map_error_code_duplicate_with_conflict() {
    let err = StoreError::DuplicateWithConflict("op-123".to_string());
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::RevisionMismatch);
}

// Idempotent append tests (bd-2qg)

#[test]
fn test_classify_duplicate_exact_match() {
    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

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

    let payload = encode_event_envelope(&envelope).expect("Failed to encode envelope");

    let record = EventRecord {
        op_id: OpId::new("op-1".to_string()).unwrap(),
        revision: StoreRevision::new(1).unwrap(),
        timestamp: Timestamp::new(1700000000).unwrap(),
        payload,
    };

    let kind = classify_duplicate(&record, &envelope);
    assert!(
        kind.is_ok(),
        "classify_duplicate should succeed: {:?}",
        kind.err()
    );
    assert_eq!(kind.expect("checked is_ok"), DuplicateKind::Exact);
}
