use super::*;


#[test]
fn test_migration_forbidden_error_display() {
    let err = StoreError::MigrationForbidden { version: 0 };
    let msg = err.to_string();
    assert!(
        msg.contains("Migration forbidden"),
        "Error message should contain 'Migration forbidden': {}",
        msg
    );
    assert!(
        msg.contains("version 0"),
        "Error message should contain version: {}",
        msg
    );
}

/// BDD: Given MigrationForbidden error, when mapping to CliErrorCode,
/// then Unknown is returned.
#[test]
fn test_map_error_code_migration_forbidden() {
    let err = StoreError::MigrationForbidden { version: 0 };
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::Unknown);
}

// ------------------------------------------------------------
// RevisionMismatch Error Path Tests (BDD-style)
// ------------------------------------------------------------

/// BDD: Given RevisionMismatch error, when displayed,
/// then the message shows expected and found revisions.
#[test]
fn test_revision_mismatch_error_display() {
    let err = StoreError::RevisionMismatch {
        expected: 10,
        found: 5,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("Revision mismatch"),
        "Error message should contain 'Revision mismatch': {}",
        msg
    );
    assert!(
        msg.contains("expected 10"),
        "Error message should contain expected revision: {}",
        msg
    );
    assert!(
        msg.contains("found 5"),
        "Error message should contain found revision: {}",
        msg
    );
}

/// BDD: Given RevisionMismatch error, when mapping to CliErrorCode,
/// then RevisionMismatch is returned.
#[test]
fn test_map_error_code_revision_mismatch_variant() {
    let err = StoreError::RevisionMismatch {
        expected: 5,
        found: 3,
    };
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::RevisionMismatch);
}

// ------------------------------------------------------------
// RevisionGap Error Path Tests (BDD-style)
// ------------------------------------------------------------

/// BDD: Given a RevisionGap error, when verified,
/// then it maps to RevisionMismatch code and displays correctly.
#[test]
fn test_revision_gap_full_error_path() {
    // Test display
    let err = StoreError::RevisionGap {
        expected: 5,
        found: 7,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("Revision gap detected"),
        "Error message should contain 'Revision gap detected': {}",
        msg
    );
    assert!(
        msg.contains("sequential revision 5"),
        "Error message should contain expected sequential revision: {}",
        msg
    );
    assert!(
        msg.contains("gap at 7"),
        "Error message should contain found gap revision: {}",
        msg
    );

    // Test error code mapping
    let code = map_error_code(&err);
    assert_eq!(
        code,
        CliErrorCode::RevisionMismatch,
        "RevisionGap should map to RevisionMismatch code"
    );
}

// ------------------------------------------------------------
// EmptyBatch Error Path Tests (BDD-style)
// ------------------------------------------------------------

/// BDD: Given EmptyBatch error, when displayed,
/// then the message mentions zero events.
#[test]
fn test_empty_batch_error_display() {
    let err = StoreError::EmptyBatch;
    let msg = err.to_string();
    assert!(
        msg.contains("Empty batch"),
        "Error message should contain 'Empty batch': {}",
        msg
    );
    assert!(
        msg.contains("zero events"),
        "Error message should mention zero events: {}",
        msg
    );
}

/// BDD: Given EmptyBatch error, when mapping to CliErrorCode,
/// then ValidationFailed is returned.
#[test]
fn test_map_error_code_empty_batch() {
    let err = StoreError::EmptyBatch;
    let code = map_error_code(&err);
    assert_eq!(code, CliErrorCode::ValidationFailed);
}

// ------------------------------------------------------------
// CorruptDatabase Error Path Tests
// ------------------------------------------------------------

/// BDD: Given CorruptDatabase error, when displayed,
/// then the message shows the corruption detail.
#[test]
fn test_corrupt_database_error_display() {
    let err = RecoveryError::CorruptDatabase("page 42 is malformed".to_string());
    let msg = err.to_string();
    assert!(
        msg.contains("integrity check failed"),
        "Error message should contain 'integrity check failed': {}",
        msg
    );
    assert!(
        msg.contains("page 42 is malformed"),
        "Error message should contain detail: {}",
        msg
    );
}

/// BDD: Given a corrupted database file, when integrity check runs,
/// then CorruptDatabase error is returned.
#[test]
fn test_corrupt_database_on_invalid_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("corrupt.db");

    // Write invalid SQLite header
    std::fs::write(&db_path, b"This is not a valid SQLite database file")
        .expect("Failed to write corrupt file");

    let result = startup_integrity_check(&db_path);
    assert!(result.is_err(), "Expected error for corrupt database");

    match result {
        Err(RecoveryError::CorruptDatabase(msg)) => {
            assert!(
                !msg.is_empty(),
                "CorruptDatabase error should have a message"
            );
        }
        Err(RecoveryError::Sqlite(_)) => {
            // SQLite error is also acceptable for corrupt file
        }
        Err(other) => panic!("Expected CorruptDatabase or Sqlite error, got: {:?}", other),
        Ok(status) => {
            // If it returns Ok, the status should indicate invalid
            assert!(
                !status.is_valid,
                "Corrupt database should be marked as invalid"
            );
        }
    }
}

// ------------------------------------------------------------
// BackupUnavailable Error Path Tests
// ------------------------------------------------------------

/// BDD: Given BackupUnavailable error, when displayed,
/// then the message shows the unavailability reason.
#[test]
fn test_backup_unavailable_error_display() {
    let err = RecoveryError::BackupUnavailable("/path/to/backup.db not found".to_string());
    let msg = err.to_string();
    assert!(
        msg.contains("Backup file unavailable"),
        "Error message should contain 'Backup file unavailable': {}",
        msg
    );
    assert!(
        msg.contains("/path/to/backup.db not found"),
        "Error message should contain detail: {}",
        msg
    );
}

// BDD: Given a nonexistent backup path, when recovery is attempted,
// then appropriate error is returned.
