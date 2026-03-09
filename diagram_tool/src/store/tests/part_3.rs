use super::*;


#[test]
fn test_submit_cli_op_success() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).unwrap();

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

    let result = submit_cli_op(&mut bootstrap.conn, envelope, None);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let outcome = result.unwrap();
    assert_eq!(outcome.revision, 1);
    assert_eq!(outcome.op_id, "op-1");
}

#[test]
fn test_submit_cli_op_revision_mismatch() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).unwrap();

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

    // Expect revision 5 but database is at 0
    let result = submit_cli_op(&mut bootstrap.conn, envelope, Some(5));
    assert!(result.is_err());
    match result {
        Err(CliError::StoreFailure(StoreError::RevisionMismatch { expected, found })) => {
            assert_eq!(expected, 5);
            assert_eq!(found, 0);
        }
        _ => panic!("Expected RevisionMismatch error, got: {:?}", result),
    }
}

#[test]
fn test_cli_submit_response() {
    let outcome = AppendOutcome {
        revision: StoreRevision::new(42).unwrap(),
        op_id: OpId::new("op-123".to_string()).unwrap(),
        timestamp: Timestamp::new(1700000000).unwrap(),
    };

    let json = cli_submit_response(&outcome);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["revision"], 42);
    assert_eq!(parsed["op_id"], "op-123");
    assert_eq!(parsed["timestamp"], 1700000000);
}

#[test]
fn test_append_outcome_from_append_result() {
    let result = AppendResult {
        revision: StoreRevision::new(10).unwrap(),
        op_id: OpId::new("op-456".to_string()).unwrap(),
        timestamp: Timestamp::new(1700000001).unwrap(),
    };

    let outcome = AppendOutcome::from(result);

    assert_eq!(outcome.revision, 10);
    assert_eq!(outcome.op_id, "op-456");
    assert_eq!(outcome.timestamp, 1700000001);
}

// Transaction helper tests

#[test]
fn test_with_write_tx_commits_on_success() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Execute a successful write transaction
    let result: Result<i64, StoreError> = with_write_tx(&mut bootstrap.conn, |tx| {
        tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["test-op", 1, "{}", "2024-01-01"],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(42)
    });

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    assert_eq!(result.unwrap(), 42);

    // Verify the data was committed
    let count: i64 = bootstrap
        .conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE operation_id = 'test-op'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count events");
    assert_eq!(count, 1);
}

#[test]
fn test_with_write_tx_rolls_back_on_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Execute a transaction that fails after a write
    let result: Result<i64, StoreError> = with_write_tx(&mut bootstrap.conn, |tx| {
        tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["test-op-rollback", 1, "{}", "2024-01-01"],
            )
            .map_err(StoreError::Sqlite)?;
        // Simulate a failure
        Err(StoreError::ValidationFailed(
            "intentional failure".to_string(),
        ))
    });

    // Should get TransactionAborted error
    assert!(result.is_err());
    match result {
        Err(StoreError::TransactionAborted(msg)) => {
            assert!(msg.contains("intentional failure"));
        }
        Err(e) => panic!("Expected TransactionAborted, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }

    // Verify the data was rolled back
    let count: i64 = bootstrap
        .conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE operation_id = 'test-op-rollback'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count events");
    assert_eq!(count, 0, "Data should have been rolled back");
}

#[test]
fn test_transaction_aborted_error_display() {
    let err = StoreError::TransactionAborted("test error".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Transaction aborted"));
    assert!(msg.contains("test error"));
}

#[test]
fn test_map_error_code_transaction_aborted() {
    let err = StoreError::TransactionAborted("test".to_string());
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::Unknown);
}

#[test]
fn test_with_write_tx_multiple_operations_atomic() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Execute multiple operations in a transaction, then fail
    let result: Result<(), StoreError> = with_write_tx(&mut bootstrap.conn, |tx| {
        tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["op1", 1, "{}", "2024-01-01"],
            )
            .map_err(StoreError::Sqlite)?;
        tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["op2", 2, "{}", "2024-01-01"],
            )
            .map_err(StoreError::Sqlite)?;
        Err(StoreError::ValidationFailed(
            "fail after inserts".to_string(),
        ))
    });

    assert!(result.is_err());

    // Verify both inserts were rolled back
    let count: i64 = bootstrap
        .conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .expect("Failed to count events");
    assert_eq!(count, 0, "All operations should have been rolled back");
}

// append_with_occ and verify_occ_append tests
