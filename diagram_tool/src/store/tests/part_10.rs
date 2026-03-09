use super::*;


#[test]
fn test_occ_stale_revision_rejected_with_no_append() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Add initial events to advance revision to 3
    for i in 1..=3 {
        let envelope = EventEnvelope {
            op_id: format!("op-{i}"),
            operation: DomainOp::NodeAdd {
                id: format!("node-{i}"),
                x: 10.0 * i as f64,
                y: 20.0 * i as f64,
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
        let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append");
    }

    // Verify current revision is 3
    let current = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(current, 3, "Database should be at revision 3");

    // Attempt to append with stale revision (claiming revision 1)
    let stale_envelope = EventEnvelope {
        op_id: "op-stale".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-stale".to_string(),
            x: 999.0,
            y: 999.0,
            width: 100.0,
            height: 50.0,
            label: "Stale Node".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000999,
    };

    let result = append_with_occ(&mut bootstrap.conn, stale_envelope, Some(1));

    // Must reject with RevisionMismatch
    assert!(result.is_err(), "Stale revision should be rejected");
    match result {
        Err(StoreError::RevisionMismatch { expected, found }) => {
            assert_eq!(expected, 1, "Expected should be the stale revision");
            assert_eq!(found, 3, "Found should be the current revision");
        }
        Err(other) => panic!("Expected RevisionMismatch, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }

    // Verify no new event was appended (revision still 3)
    let after_revision = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(
        after_revision, 3,
        "Revision should still be 3 after rejected append"
    );

    // Verify the stale op_id does not exist in the database
    let count: i64 = bootstrap
        .conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE operation_id = 'op-stale'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count events");
    assert_eq!(count, 0, "Stale operation should not be in the database");
}

/// Regression test: exact duplicate op_id must return no-op success
///
/// This test verifies that when the same operation (same op_id and payload)
/// is submitted again via append_idempotent, it returns Ok with the
/// existing outcome and does not append a new row.
#[test]
fn test_occ_exact_duplicate_returns_noop_success() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let envelope = EventEnvelope {
        op_id: "op-duplicate-test".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-dup".to_string(),
            x: 42.0,
            y: 84.0,
            width: 100.0,
            height: 50.0,
            label: "Duplicate Test Node".to_string(),
        },
        author: Author {
            id: "user-dup".to_string(),
            name: "Duplicate User".to_string(),
            email: None,
        },
        timestamp: 1700000420,
    };

    // First append - should succeed with revision 1
    let result1 = append_idempotent(&mut bootstrap.conn, envelope.clone());
    assert!(
        result1.is_ok(),
        "First append should succeed: {:?}",
        result1.err()
    );
    let outcome1 = result1.expect("checked is_ok");
    assert_eq!(
        outcome1.revision, 1,
        "First append should create revision 1"
    );
    assert_eq!(outcome1.op_id, "op-duplicate-test");
    assert_eq!(outcome1.timestamp, 1700000420);

    // Second append with exact duplicate - must return no-op success
    let result2 = append_idempotent(&mut bootstrap.conn, envelope.clone());
    assert!(
        result2.is_ok(),
        "Exact duplicate should return Ok (no-op success): {:?}",
        result2.err()
    );
    let outcome2 = result2.expect("checked is_ok");

    // Must return the existing outcome (same revision, op_id, timestamp)
    assert_eq!(
        outcome2.revision, outcome1.revision,
        "Duplicate should return existing revision"
    );
    assert_eq!(
        outcome2.op_id, outcome1.op_id,
        "Duplicate should return existing op_id"
    );
    assert_eq!(
        outcome2.timestamp, outcome1.timestamp,
        "Duplicate should return existing timestamp"
    );

    // Verify only one row exists in the database
    let count: i64 = bootstrap
        .conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE operation_id = 'op-duplicate-test'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count events");
    assert_eq!(count, 1, "Exact duplicate should not create a new row");

    // Verify current revision is still 1
    let current = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(
        current, 1,
        "Revision should still be 1 after no-op duplicate"
    );
}

// Regression test: duplicate op_id with different payload must return error
//
// This test verifies that when an operation with the same op_id but
// different payload is submitted, it returns DuplicateWithConflict error.
