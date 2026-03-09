use super::*;


#[test]
fn test_append_idempotent_preserves_revision_on_duplicate() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Add several operations first
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
        let _ = append_idempotent(&mut bootstrap.conn, envelope).expect("Failed to append");
    }

    // Verify we're at revision 5
    let rev_before = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(rev_before, 5);

    // Now try to append exact duplicate of op-3
    let envelope_dup = EventEnvelope {
        op_id: "op-3".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-3".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Node 3".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000003,
    };

    let result = append_idempotent(&mut bootstrap.conn, envelope_dup);
    assert!(
        result.is_ok(),
        "Exact duplicate should succeed: {:?}",
        result.err()
    );

    // Revision should be unchanged
    let rev_after = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(
        rev_after, rev_before,
        "Revision should be unchanged after exact duplicate"
    );
}

#[test]
fn test_append_idempotent_multiple_different_ops() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Add op-1
    let envelope1 = EventEnvelope {
        op_id: "op-1".to_string(),
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
    };
    let result1 = append_idempotent(&mut bootstrap.conn, envelope1);
    assert!(result1.is_ok());
    assert_eq!(result1.expect("checked is_ok").revision, 1);

    // Add op-2
    let envelope2 = EventEnvelope {
        op_id: "op-2".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-2".to_string(),
            x: 20.0,
            y: 30.0,
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
    };
    let result2 = append_idempotent(&mut bootstrap.conn, envelope2);
    assert!(result2.is_ok());
    assert_eq!(result2.expect("checked is_ok").revision, 2);

    // Add op-3 (new)
    let envelope3 = EventEnvelope {
        op_id: "op-3".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-3".to_string(),
            x: 30.0,
            y: 40.0,
            width: 100.0,
            height: 50.0,
            label: "Node 3".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000003,
    };
    let result3 = append_idempotent(&mut bootstrap.conn, envelope3);
    assert!(result3.is_ok());
    assert_eq!(result3.expect("checked is_ok").revision, 3);
}

#[test]
fn test_duplicate_kind_equality() {
    assert_eq!(DuplicateKind::Exact, DuplicateKind::Exact);
    assert_eq!(DuplicateKind::Conflict, DuplicateKind::Conflict);
    assert_ne!(DuplicateKind::Exact, DuplicateKind::Conflict);
}

#[test]
fn test_append_idempotent_with_different_operation_types() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    // Add a NodeAdd operation
    let envelope_add = EventEnvelope {
        op_id: "op-add".to_string(),
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
    let result_add = append_idempotent(&mut bootstrap.conn, envelope_add.clone());
    assert!(result_add.is_ok());

    // Exact duplicate of NodeAdd
    let result_dup = append_idempotent(&mut bootstrap.conn, envelope_add);
    assert!(result_dup.is_ok());

    // Add a NodeMove operation
    let envelope_move = EventEnvelope {
        op_id: "op-move".to_string(),
        operation: DomainOp::NodeMove {
            id: "node-1".to_string(),
            x: 100.0,
            y: 200.0,
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000001,
    };
    let result_move = append_idempotent(&mut bootstrap.conn, envelope_move.clone());
    assert!(result_move.is_ok());

    // Exact duplicate of NodeMove
    let result_move_dup = append_idempotent(&mut bootstrap.conn, envelope_move);
    assert!(result_move_dup.is_ok());
}

// append_batch tests
