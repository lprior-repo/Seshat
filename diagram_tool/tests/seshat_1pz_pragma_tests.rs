#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    dead_code
)]
//! Integration tests for seshat-1pz: PRAGMA synchronous=NORMAL
//!
//! These tests verify that the async `SQLite` store correctly configures
//! PRAGMA synchronous=NORMAL when using WAL mode.
//!
//! Run with: cargo test --package `diagram_tool` --test `seshat_1pz_pragma_tests`
//!
//! All tests follow the BDD pattern: `given_X_when_Y_then_Z`

#![cfg(not(target_arch = "wasm32"))]

use diagram_tool::store_async::{
    bootstrap_async_store, create_async_pool, read_store_pragmas_async, AsyncStoreError,
    AsyncStorePragmas,
};
use sqlx::SqlitePool;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create a temporary database pool for testing
async fn create_test_pool() -> Result<(TempDir, SqlitePool), AsyncStoreError> {
    let temp_dir = TempDir::new().map_err(AsyncStoreError::Io)?;
    let db_path = temp_dir.path().join("test.db");
    let pool = create_async_pool(&db_path).await?;
    Ok((temp_dir, pool))
}

/// Helper to create a test pool and read its pragmas
async fn create_test_pool_with_pragmas(
) -> Result<(TempDir, SqlitePool, AsyncStorePragmas), AsyncStoreError> {
    let temp_dir = TempDir::new().map_err(AsyncStoreError::Io)?;
    let db_path = temp_dir.path().join("test.db");
    let pool = create_async_pool(&db_path).await?;
    let pragmas = read_store_pragmas_async(&pool).await?;
    Ok((temp_dir, pool, pragmas))
}

// ============================================================================
// Happy Path Tests - Verify Q1 and Q2 postconditions
// ============================================================================

/// Given: A valid temporary database path
/// When: `create_async_pool(db_path)` is called
/// Then: PRAGMA synchronous returns value 1 (NORMAL)
#[tokio::test]
async fn given_valid_path_when_create_async_pool_then_synchronous_is_normal() {
    // Given: temp directory (implicit via create_test_pool_with_pragmas)
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // When & Then: synchronous should be NORMAL (value = 1)
    assert_eq!(
        pragmas.synchronous, 1,
        "Expected synchronous=NORMAL (1), got {}",
        pragmas.synchronous
    );
}

/// Given: A valid temporary database path
/// When: `create_async_pool(db_path)` is called
/// Then: PRAGMA `journal_mode` returns "wal"
#[tokio::test]
async fn given_wal_mode_when_pool_created_then_journal_mode_is_wal() {
    // Given & When: pool created
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Then: journal_mode should be "wal"
    assert_eq!(
        pragmas.journal_mode, "wal",
        "Expected journal_mode=wal, got {}",
        pragmas.journal_mode
    );
}

/// Given: A valid temporary database path with pool created
/// When: `read_store_pragmas_async(pool)` is called
/// Then: All PRAGMA values are correct
#[tokio::test]
async fn given_pool_created_when_pragma_values_queried_then_all_are_correct() {
    // Given & When: pool created with pragmas read
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Then: All pragma values should match expected
    assert_eq!(pragmas.journal_mode, "wal");
    assert_eq!(pragmas.synchronous, 1); // NORMAL
    assert_eq!(pragmas.wal_autocheckpoint, 1000);
    assert!(pragmas.foreign_keys);
    assert_eq!(pragmas.busy_timeout, 5000);
}

/// Given: A valid temporary database path
/// When: `bootstrap_async_store(db_path)` completes
/// Then: Schema version table exists and has version 1
#[tokio::test]
async fn given_bootstrap_store_when_init_complete_then_schema_version_set() {
    // Given: temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // When: bootstrap completes
    let bootstrap = bootstrap_async_store(&db_path)
        .await
        .expect("Failed to bootstrap store");

    // Then: schema version should be 1
    assert_eq!(
        bootstrap.schema_version, 1,
        "Expected schema_version=1, got {}",
        bootstrap.schema_version
    );
}

// ============================================================================
// Error Path Tests - Verify precondition P1 and P2 error handling
// ============================================================================

/// Given: An invalid path "/nonexistent/invalid/path.db"
/// When: `create_async_pool(invalid_path)` is called
/// Then: Returns `Err(AsyncStoreError::Sqlx`(_) or `AsyncStoreError::Io`(_))
#[tokio::test]
async fn given_invalid_path_when_create_async_pool_then_returns_error() {
    // Given: invalid path
    let invalid_path = Path::new("/nonexistent/invalid/path.db");

    // When: pool creation attempted
    let result = create_async_pool(invalid_path).await;

    // Then: should return error
    assert!(
        result.is_err(),
        "Expected error for invalid path, got {result:?}"
    );
}

/// Given: A path to a file that exists but is not a valid `SQLite` database
/// When: `create_async_pool(path)` is called
/// Then: Returns `Err(AsyncStoreError::Sqlx`(_))
#[tokio::test]
async fn given_corrupted_file_when_create_pool_then_connection_fails() {
    // Given: temp dir with invalid file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let invalid_db = temp_dir.path().join("not_a_db.txt");

    // Create a non-database file
    std::fs::write(&invalid_db, "This is not a SQLite database")
        .expect("Failed to write test file");

    // When: pool creation attempted
    let result = create_async_pool(&invalid_db).await;

    // Then: should return error
    assert!(
        result.is_err(),
        "Expected error for corrupted file, got {result:?}"
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Given: A database that already has WAL files (created by first pool)
/// When: A second pool is created for the same database
/// Then: Pool creation succeeds and PRAGMA values are correct
#[tokio::test]
async fn given_existing_wal_files_when_pool_created_then_succeeds() {
    // Given: first pool creates WAL files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let _pool1 = create_async_pool(&db_path)
        .await
        .expect("Failed to create first pool");

    // When: second pool created for same database
    let pool2 = create_async_pool(&db_path)
        .await
        .expect("Failed to create second pool");

    // Then: pragmas should still be correct
    let pragmas = read_store_pragmas_async(&pool2)
        .await
        .expect("Failed to read pragmas");

    assert_eq!(pragmas.synchronous, 1);
    assert_eq!(pragmas.journal_mode, "wal");
}

/// Given: A pool created with correct PRAGMA settings
/// When: PRAGMA values are queried after multiple operations
/// Then: All PRAGMA values remain unchanged (Invariant I3)
#[tokio::test]
async fn given_pool_lifetime_when_pragmas_queried_multiple_times_then_values_stable() {
    // Given: pool with correct pragmas
    let (_temp_dir, pool, initial_pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // When: multiple queries executed (simulate some operations)
    let _ = sqlx::query("SELECT 1").execute(&pool).await;
    let _ = sqlx::query("SELECT 1").execute(&pool).await;
    let _ = sqlx::query("SELECT 1").execute(&pool).await;

    // Then: pragmas should be unchanged
    let final_pragmas = read_store_pragmas_async(&pool)
        .await
        .expect("Failed to read pragmas");

    assert_eq!(initial_pragmas.synchronous, final_pragmas.synchronous);
    assert_eq!(initial_pragmas.journal_mode, final_pragmas.journal_mode);
    assert_eq!(
        initial_pragmas.wal_autocheckpoint,
        final_pragmas.wal_autocheckpoint
    );
}

// ============================================================================
// Contract Verification Tests - Verify each Q postcondition individually
// ============================================================================

/// Given: A newly initialized async pool
/// When: `read_store_pragmas_async(pool)` is called
/// Then: synchronous field equals 1
#[tokio::test]
async fn given_pool_created_when_synchronous_queried_then_returns_one() {
    // Given & When
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Then
    assert_eq!(pragmas.synchronous, 1);
}

/// Given: A newly initialized async pool
/// When: `read_store_pragmas_async(pool)` is called
/// Then: `journal_mode` field equals "wal"
#[tokio::test]
async fn given_pool_created_when_journal_mode_queried_then_returns_wal() {
    // Given & When
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Then
    assert_eq!(pragmas.journal_mode, "wal");
}

/// Given: A newly initialized async pool
/// When: `read_store_pragmas_async(pool)` is called
/// Then: `wal_autocheckpoint` field equals 1000
#[tokio::test]
async fn given_pool_created_when_wal_autocheckpoint_queried_then_returns_1000() {
    // Given & When
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Then
    assert_eq!(pragmas.wal_autocheckpoint, 1000);
}

/// Given: A newly initialized async pool
/// When: `read_store_pragmas_async(pool)` is called
/// Then: `foreign_keys` field equals true
#[tokio::test]
async fn given_pool_created_when_foreign_keys_queried_then_returns_true() {
    // Given & When
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Then
    assert!(pragmas.foreign_keys, "Expected foreign_keys=true");
}

/// Given: A newly initialized async pool
/// When: `read_store_pragmas_async(pool)` is called
/// Then: `busy_timeout` field equals 5000
#[tokio::test]
async fn given_pool_created_when_busy_timeout_queried_then_returns_5000() {
    // Given & When
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Then
    assert_eq!(pragmas.busy_timeout, 5000);
}

// ============================================================================
// Contract Violation Tests - Verify error detection
// ============================================================================

/// Given: A pool (any valid pool)
/// When: PRAGMA synchronous=FULL is manually executed
/// Then: Querying PRAGMA synchronous returns value 2
/// This test verifies we can detect the bug that was fixed
#[tokio::test]
async fn given_full_synchronous_pragma_when_read_then_returns_two() {
    // Given: a valid pool
    let (_temp_dir, pool, _pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // When: we manually set synchronous to FULL (simulating the bug)
    sqlx::query("PRAGMA synchronous=FULL")
        .execute(&pool)
        .await
        .expect("Failed to set PRAGMA");

    // Then: reading synchronous returns 2 (FULL)
    let result: (i32,) = sqlx::query_as("PRAGMA synchronous")
        .fetch_one(&pool)
        .await
        .expect("Failed to read PRAGMA");

    assert_eq!(result.0, 2, "Expected synchronous=2 (FULL)");
}

/// Given: An invalid path that cannot be accessed
/// When: `create_async_pool(invalid_path)` is called
/// Then: Returns `Err(AsyncStoreError::Io`(_) or `AsyncStoreError::Sqlx`(_))
#[tokio::test]
async fn given_invalid_database_path_when_pool_created_then_returns_io_error() {
    // Given: invalid path with no parent permissions
    let invalid_path = Path::new("/proc/0/1/2/3/invalid.db");

    // When
    let result = create_async_pool(invalid_path).await;

    // Then
    assert!(result.is_err());
}

// ============================================================================
// Data Persistence Tests - Verify Invariant I1 (data safety)
// ============================================================================

/// Given: A pool with WAL mode and synchronous=NORMAL
/// When: Data is written to the database (INSERT events)
/// Then: WAL file contains the data (or at least data is committed)
#[tokio::test]
async fn given_wal_mode_when_data_written_then_data_committed() {
    // Given: pool with WAL mode
    let (_temp_dir, pool, _pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // When: data is written
    sqlx::query("CREATE TABLE IF NOT EXISTS test_data (id INTEGER PRIMARY KEY, value TEXT)")
        .execute(&pool)
        .await
        .expect("Failed to create table");

    sqlx::query("INSERT INTO test_data (value) VALUES ('test_value')")
        .execute(&pool)
        .await
        .expect("Failed to insert data");

    // Then: data can be read back
    let result: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM test_data")
        .fetch_one(&pool)
        .await
        .expect("Failed to count data");

    assert_eq!(result.0, 1);
}

/// Given: A pool with data written but not checkpointed
/// When: PRAGMA `wal_checkpoint(TRUNCATE)` is executed
/// Then: Checkpoint succeeds and data is persisted to main database
#[tokio::test]
async fn given_data_in_wal_when_checkpoint_forced_then_data_persisted() {
    // Given: pool with data
    let (_temp_dir, pool, _pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Create table and insert data
    sqlx::query("CREATE TABLE IF NOT EXISTS checkpoint_test (id INTEGER PRIMARY KEY, data TEXT)")
        .execute(&pool)
        .await
        .expect("Failed to create table");

    sqlx::query("INSERT INTO checkpoint_test (data) VALUES ('checkpoint_test_data')")
        .execute(&pool)
        .await
        .expect("Failed to insert");

    // When: checkpoint is forced
    let _checkpoint_result: (i32, i32, i32) = sqlx::query_as("PRAGMA wal_checkpoint(TRUNCATE)")
        .fetch_one(&pool)
        .await
        .expect("Failed to checkpoint");

    // Then: data can still be read after checkpoint (proves persistence)
    let result: (String,) = sqlx::query_as("SELECT data FROM checkpoint_test LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to read data");

    assert_eq!(result.0, "checkpoint_test_data");
}

/// Given: Pool with data written and properly closed
/// When: New pool is created for the same database
/// Then: Previously written data is readable
#[tokio::test]
async fn given_pool_with_data_when_reopened_then_data_recovered() {
    // Given: temp directory and initial pool
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create initial pool and write data
    {
        let pool = create_async_pool(&db_path)
            .await
            .expect("Failed to create pool");

        sqlx::query("CREATE TABLE IF NOT EXISTS recovery_test (id INTEGER PRIMARY KEY, data TEXT)")
            .execute(&pool)
            .await
            .expect("Failed to create table");

        sqlx::query("INSERT INTO recovery_test (data) VALUES ('recovery_data')")
            .execute(&pool)
            .await
            .expect("Failed to insert");

        // Pool goes out of scope and closes
    }

    // When: new pool opened
    let new_pool = create_async_pool(&db_path)
        .await
        .expect("Failed to open new pool");

    // Then: data is readable
    let result: (String,) = sqlx::query_as("SELECT data FROM recovery_test LIMIT 1")
        .fetch_one(&new_pool)
        .await
        .expect("Failed to read data");

    assert_eq!(result.0, "recovery_data");
}

// ============================================================================
// Async/Sync Parity Tests - Verify Invariant I2
// ============================================================================

/// This test verifies that the async store uses synchronous=NORMAL
/// which matches what the sync store (`store_sqlx.rs`) should also use.
#[tokio::test]
async fn given_async_store_configuration_should_match_sync_store_contract() {
    // Given: async store configured
    let (_temp_dir, _pool, pragmas) = create_test_pool_with_pragmas()
        .await
        .expect("Failed to create test pool");

    // Then: should match sync store contract (synchronous=NORMAL)
    // The sync store (store_sqlx.rs line 179) uses PRAGMA synchronous=NORMAL
    assert_eq!(
        pragmas.synchronous, 1,
        "Async store should use NORMAL (1) to match sync store contract"
    );
    assert_eq!(
        pragmas.journal_mode, "wal",
        "Async store should use WAL to match sync store contract"
    );
}
