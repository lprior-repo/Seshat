use super::*;


#[test]
fn test_open_recovery_only_on_valid_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create a valid database
    let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Open in recovery-only mode using contract signature
    let session = open_recovery_only(&db_path).expect("Failed to open recovery only mode");

    // Verify connection is read-only
    let result = session
        .conn
        .query_row("SELECT 1", [], |row| row.get::<_, i32>(0));
    assert!(
        result.is_ok(),
        "Should be able to read from recovery only mode"
    );
}

#[test]
fn test_recovery_session_is_same_as_recovery_handle() {
    // Verify RecoverySession is an alias for RecoveryHandle
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    let handle = open_recovery_mode(&db_path).expect("Failed to open recovery mode");
    let session = open_recovery_only(&db_path).expect("Failed to open recovery only");

    // Both should have same structure
    assert_eq!(handle.db_path, session.db_path);
}

// CliErrorCode tests

#[test]
fn test_map_error_code_revision_mismatch() {
    let err = StoreError::RevisionMismatch {
        expected: 5,
        found: 3,
    };
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::RevisionMismatch);
}

#[test]
fn test_map_error_code_human_priority_block() {
    let err = StoreError::HumanPriorityBlock("user is editing".to_string());
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::HumanPriorityBlock);
}

#[test]
fn test_map_error_code_validation_failed() {
    let err = StoreError::ValidationFailed("invalid node position".to_string());
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::ValidationFailed);
}

#[test]
fn test_map_error_code_sqlite() {
    let err = StoreError::Sqlite(rusqlite::Error::InvalidQuery);
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::Unknown);
}

#[test]
fn test_map_error_code_io() {
    let err = StoreError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file not found",
    ));
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::Unknown);
}

#[test]
fn test_render_error_json_revision_mismatch() {
    let json = render_error_json(
        CliErrorCode::RevisionMismatch,
        "expected revision 5 but found 3",
    );
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(parsed["code"], "revision_mismatch");
    assert_eq!(parsed["message"], "expected revision 5 but found 3");
}

#[test]
fn test_render_error_json_human_priority_block() {
    let json = render_error_json(CliErrorCode::HumanPriorityBlock, "user is editing");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(parsed["code"], "human_priority_block");
    assert_eq!(parsed["message"], "user is editing");
}

#[test]
fn test_render_error_json_validation_failed() {
    let json = render_error_json(CliErrorCode::ValidationFailed, "invalid node position");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(parsed["code"], "validation_failed");
    assert_eq!(parsed["message"], "invalid node position");
}

#[test]
fn test_render_error_json_policy_violation() {
    let json = render_error_json(CliErrorCode::PolicyViolation, "operation not allowed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(parsed["code"], "policy_violation");
    assert_eq!(parsed["message"], "operation not allowed");
}

#[test]
fn test_render_error_json_unknown() {
    let json = render_error_json(CliErrorCode::Unknown, "internal error");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(parsed["code"], "unknown");
    assert_eq!(parsed["message"], "internal error");
}

#[test]
fn test_cli_error_code_serialization() {
    let code = CliErrorCode::RevisionMismatch;
    let json = serde_json::to_string(&code).expect("valid JSON");
    assert_eq!(json, "\"revision_mismatch\"");
}

// CliError and submit_cli_op tests

#[test]
fn test_cli_error_error_code_invalid_input() {
    let err = CliError::InvalidInput("test".to_string());
    assert_eq!(err.error_code(), CliErrorCode::ValidationFailed);
}

#[test]
fn test_cli_error_error_code_conflict() {
    let err = CliError::Conflict("revision mismatch".to_string());
    assert_eq!(err.error_code(), CliErrorCode::RevisionMismatch);
}

#[test]
fn test_cli_error_error_code_serialization() {
    let err = CliError::Serialization("failed".to_string());
    assert_eq!(err.error_code(), CliErrorCode::Unknown);
}

#[test]
fn test_cli_error_error_code_store_failure() {
    let store_err = StoreError::RevisionMismatch {
        expected: 1,
        found: 2,
    };
    let err = CliError::StoreFailure(store_err);
    assert_eq!(err.error_code(), CliErrorCode::RevisionMismatch);
}

#[test]
fn test_submit_cli_op_missing_op_id() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).unwrap();

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};
    let envelope = EventEnvelope {
        op_id: String::new(), // Empty op_id
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
    assert!(result.is_err());
    match result {
        Err(CliError::InvalidInput(msg)) => assert!(msg.contains("op_id")),
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_submit_cli_op_missing_author_id() {
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
            id: String::new(), // Empty author id
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };

    let result = submit_cli_op(&mut bootstrap.conn, envelope, None);
    assert!(result.is_err());
    match result {
        Err(CliError::InvalidInput(msg)) => assert!(msg.contains("author.id")),
        _ => panic!("Expected InvalidInput error"),
    }
}
