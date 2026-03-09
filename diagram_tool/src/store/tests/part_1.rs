use super::*;


#[test]
fn test_bootstrap_store_creates_database_with_schema() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    assert_eq!(
        bootstrap.schema_version,
        crate::store::CURRENT_SCHEMA_VERSION
    );
    assert_eq!(bootstrap.db_path, db_path);

    // Verify the database file exists
    assert!(db_path.exists(), "Database file should exist");
}

#[test]
fn test_bootstrap_store_enforces_wal_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");
    let config = current_store_config(&bootstrap.conn).expect("Failed to get config");

    assert_eq!(config.pragmas.journal_mode, JournalMode::Wal);
}

#[test]
fn test_bootstrap_store_enforces_synchronous_full() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");
    let config = current_store_config(&bootstrap.conn).expect("Failed to get config");

    assert_eq!(config.pragmas.synchronous, SynchronousMode::Full);
}

#[test]
fn test_bootstrap_store_with_invalid_path() {
    // Try to create a database in a non-existent directory
    let invalid_path = Path::new("/nonexistent/path/test.db");

    let result = bootstrap_store(invalid_path);

    assert!(result.is_err());
    match result {
        Err(StoreError::Io(_)) => {}
        Err(StoreError::Sqlite(_)) => {}
        Err(other) => panic!("Expected Io or Sqlite error, got {:?}", other),
        _ => panic!("Expected error, got success"),
    }
}

#[test]
fn test_bootstrap_store_creates_schema_tables() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Verify the schema_version table exists and has correct version
    let version: i32 = bootstrap
        .conn
        .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
        .expect("Failed to read schema version");

    assert_eq!(version, CURRENT_SCHEMA_VERSION);
}

#[test]
fn test_current_store_config_returns_pragmas_and_version() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");
    let config = current_store_config(&bootstrap.conn).expect("Failed to get config");

    assert_eq!(config.pragmas.journal_mode, JournalMode::Wal);
    assert_eq!(config.pragmas.synchronous, SynchronousMode::Full);
    assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
}

#[test]
fn test_bootstrap_idempotent_on_existing_schema() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // First bootstrap
    let bootstrap1 = bootstrap_store(&db_path).expect("First bootstrap failed");
    let config1 = current_store_config(&bootstrap1.conn).expect("Failed to get config1");

    // Second bootstrap should be idempotent
    let bootstrap2 = bootstrap_store(&db_path).expect("Second bootstrap failed");
    let config2 = current_store_config(&bootstrap2.conn).expect("Failed to get config2");

    assert_eq!(config1.schema_version, config2.schema_version);
}

#[test]
fn test_open_store_with_existing_wal_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // First create with bootstrap
    let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Then open with open_store
    let store = open_store(&db_path).expect("Failed to open store");
    let pragmas = read_store_pragmas(&store.conn).expect("Failed to read pragmas");

    assert_eq!(pragmas.journal_mode, JournalMode::Wal);
    assert_eq!(pragmas.synchronous, SynchronousMode::Full);
}

// Recovery mode tests

#[test]
fn test_startup_integrity_check_on_valid_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create a valid database
    let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Run integrity check
    let status = startup_integrity_check(&db_path).expect("Integrity check failed");

    assert!(status.is_valid, "Database should be valid");
    assert!(
        status.error_message.is_none(),
        "Should have no error message"
    );
    assert!(
        status.schema_version.is_some(),
        "Should have schema version"
    );
}

#[test]
fn test_startup_integrity_check_on_nonexistent_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("nonexistent.db");

    // Run integrity check on nonexistent file
    let status = startup_integrity_check(&db_path).expect("Integrity check failed");

    assert!(!status.is_valid, "Nonexistent database should not be valid");
    assert!(status.error_message.is_some(), "Should have error message");
}

#[test]
fn test_open_recovery_mode_on_valid_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create a valid database
    let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Open in recovery mode
    let handle = open_recovery_mode(&db_path).expect("Failed to open recovery mode");

    // Verify connection is read-only
    let result = handle
        .conn
        .query_row("SELECT 1", [], |row| row.get::<_, i32>(0));
    assert!(result.is_ok(), "Should be able to read from recovery mode");
}

#[test]
fn test_recovery_handle_export_to_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create a valid database
    let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Add some test events
    use crate::models::envelope::{Author, DomainOp, EventEnvelope};
    let envelope = EventEnvelope {
        op_id: "test-op-1".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Test Node".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };
    let _ = append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append event");

    // Open in recovery mode and export
    let mut handle = open_recovery_mode(&db_path).expect("Failed to open recovery mode");
    let export_path = temp_dir.path().join("export.json");

    let export_result = handle.export_to_json(&export_path);
    assert!(
        export_result.is_ok(),
        "Export should succeed: {:?}",
        export_result.err()
    );
    assert!(export_path.exists(), "Export file should exist");
}

// Contract signature tests - bd-7rt

#[test]
fn test_integrity_check_on_valid_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create a valid database
    let _bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

    // Run integrity check using contract signature
    let status = integrity_check(&db_path).expect("Integrity check failed");

    assert!(status.is_valid, "Database should be valid");
    assert!(
        status.error_message.is_none(),
        "Should have no error message"
    );
    assert!(
        status.schema_version.is_some(),
        "Should have schema version"
    );
}

#[test]
fn test_integrity_check_on_nonexistent_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("nonexistent.db");

    // Run integrity check on nonexistent file using contract signature
    let status = integrity_check(&db_path).expect("Integrity check failed");

    assert!(!status.is_valid, "Nonexistent database should not be valid");
    assert!(status.error_message.is_some(), "Should have error message");
}
