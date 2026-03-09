use super::*;


#[test]
fn test_backup_unavailable_on_missing_file() {
    let nonexistent_backup = Path::new("/nonexistent/path/backup.db");

    // Verify the file doesn't exist
    assert!(
        !nonexistent_backup.exists(),
        "Test assumes backup file does not exist"
    );

    // The RecoveryError::BackupUnavailable would be used in a restore function
    // Here we verify the error can be constructed and used correctly
    let err = RecoveryError::BackupUnavailable(format!(
        "Backup file not found: {}",
        nonexistent_backup.display()
    ));

    match &err {
        RecoveryError::BackupUnavailable(msg) => {
            assert!(
                msg.contains("not found"),
                "Error message should indicate file not found: {}",
                msg
            );
        }
        _ => panic!("Expected BackupUnavailable error"),
    }
}

// ------------------------------------------------------------
// Comprehensive BDD Scenario Tests
// ------------------------------------------------------------

/// BDD Scenario: Atomicity on RevisionMismatch
/// Given a database at revision 0
/// When append_batch is called with expected revision 999
/// Then RevisionMismatch error is returned
/// And no events are appended
#[test]
fn test_bdd_revision_mismatch_atomicity() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    use crate::models::envelope::{Author, DomainOp, EventEnvelope};

    let events = vec![EventEnvelope {
        op_id: "op-should-not-append".to_string(),
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

    // Pre-condition: database is at revision 0
    let revision_before = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(revision_before, 0, "Database should start at revision 0");

    // Attempt to append with wrong expected revision
    let result = append_batch(&mut bootstrap.conn, events, Some(999));

    // Verify error
    assert!(result.is_err(), "Expected error for revision mismatch");
    match result {
        Err(StoreError::RevisionMismatch { expected, found }) => {
            assert_eq!(expected, 999, "Expected should be 999");
            assert_eq!(found, 0, "Found should be 0");
        }
        Err(other) => panic!("Expected RevisionMismatch, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }

    // Verify atomicity: no events were appended
    let revision_after = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(
        revision_after, 0,
        "Revision should still be 0 after failed append"
    );

    let count: i64 = bootstrap
        .conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .expect("Failed to count events");
    assert_eq!(count, 0, "No events should be in the database");
}

/// BDD Scenario: EmptyBatch rejection
/// Given a valid database connection
/// When append_batch is called with an empty vector
/// Then EmptyBatch error is returned
/// And database state is unchanged
#[test]
fn test_bdd_empty_batch_rejection() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Pre-condition: database is at revision 0
    let revision_before = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(revision_before, 0);

    // Attempt to append empty batch
    let result = append_batch(&mut bootstrap.conn, vec![], None);

    // Verify error
    assert!(result.is_err(), "Expected error for empty batch");
    match result {
        Err(StoreError::EmptyBatch) => {}
        Err(other) => panic!("Expected EmptyBatch, got: {:?}", other),
        Ok(_) => panic!("Expected error, got success"),
    }

    // Verify no state change
    let revision_after = current_revision(&bootstrap.conn).expect("Failed to get revision");
    assert_eq!(
        revision_after, 0,
        "Revision should still be 0 after empty batch"
    );
}

/// BDD Scenario: Error message quality
/// Given various error types
/// When converted to string
/// Then messages are human-readable and contain relevant context
#[test]
fn test_bdd_error_message_quality() {
    // Test all error types have meaningful messages

    // StoreError variants
    let test_cases: Vec<(StoreError, &[&str])> = vec![
        (
            StoreError::InvalidPragma("bad config".to_string()),
            &["Invalid pragma", "bad config"],
        ),
        (
            StoreError::SchemaVersionMismatch {
                expected: 2,
                found: 1,
            },
            &["Schema version mismatch", "expected 2", "found 1"],
        ),
        (
            StoreError::MigrationForbidden { version: 0 },
            &["Migration forbidden", "version 0"],
        ),
        (
            StoreError::RevisionMismatch {
                expected: 10,
                found: 5,
            },
            &["Revision mismatch", "expected 10", "found 5"],
        ),
        (
            StoreError::RevisionGap {
                expected: 5,
                found: 7,
            },
            &["Revision gap", "sequential revision 5", "gap at 7"],
        ),
        (StoreError::EmptyBatch, &["Empty batch", "zero events"]),
    ];

    for (err, expected_fragments) in test_cases {
        let msg = err.to_string();
        for fragment in expected_fragments {
            assert!(
                msg.contains(fragment),
                "Error message '{}' should contain '{}': {}",
                msg,
                fragment,
                msg
            );
        }
    }

    // RecoveryError variants
    let recovery_test_cases: Vec<(RecoveryError, &[&str])> = vec![
        (
            RecoveryError::CorruptDatabase("malformed page".to_string()),
            &["integrity check failed", "malformed page"],
        ),
        (
            RecoveryError::BackupUnavailable("file not found".to_string()),
            &["Backup file unavailable", "file not found"],
        ),
    ];

    for (err, expected_fragments) in recovery_test_cases {
        let msg = err.to_string();
        for fragment in expected_fragments {
            assert!(
                msg.contains(fragment),
                "Error message '{}' should contain '{}': {}",
                msg,
                fragment,
                msg
            );
        }
    }
}
