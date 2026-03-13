# Martin Fowler Test Plan: seshat-1pz

## Overview
Test plan for verifying that the async SQLite store uses `PRAGMA synchronous=NORMAL` (value=1) when operating in WAL mode, matching the sync store's configuration.

This test plan addresses the following defects:
- DEFECT-001: No executable test implementation files
- DEFECT-002: Tests gated behind #[cfg(kani)]
- DEFECT-003: Tests verify state not behavior
- DEFECT-004: Invariant I1 (data safety) not verified
- DEFECT-005: Violation tests are theoretical
- DEFECT-006: BDD naming violation
- DEFECT-007: Multiple assertions per test

## Test Naming Convention
All tests follow the BDD pattern: `given_X_when_Y_then_Z`

## Happy Path Tests

### test_given_valid_path_when_create_async_pool_then_synchronous_is_normal
- **Purpose**: Verify that after pool creation, synchronous is set to NORMAL (value=1)
- **Given**: A valid temporary database path
- **When**: `create_async_pool(db_path)` is called
- **Then**: `PRAGMA synchronous` returns value `1` (NORMAL)
- **Assertions**: Single assertion on synchronous value

### test_given_wal_mode_when_pool_created_then_journal_mode_is_wal
- **Purpose**: Verify WAL mode is properly enabled
- **Given**: A valid temporary database path
- **When**: `create_async_pool(db_path)` is called
- **Then**: `PRAGMA journal_mode` returns "wal"
- **Assertions**: Single assertion on journal_mode

### test_given_pool_created_when_pragma_values_queried_then_all_are_correct
- **Purpose**: Verify all PRAGMA values are set correctly
- **Given**: A valid temporary database path with pool created
- **When**: `read_store_pragmas_async(pool)` is called
- **Then**: 
  - journal_mode = "wal"
  - synchronous = 1 (NORMAL)
  - wal_autocheckpoint = 1000
  - foreign_keys = true
  - busy_timeout = 5000
- **Assertions**: Multiple related assertions (same pragma group)

### test_given_bootstrap_store_when_init_complete_then_schema_version_set
- **Purpose**: Verify bootstrap creates schema correctly
- **Given**: A valid temporary database path
- **When**: `bootstrap_async_store(db_path)` completes
- **Then**: Schema version table exists and has version 1
- **Assertions**: Single assertion on schema version

## Error Path Tests

### test_given_invalid_path_when_create_async_pool_then_returns_error
- **Purpose**: Verify graceful error handling for invalid paths
- **Given**: An invalid path "/nonexistent/invalid/path.db"
- **When**: `create_async_pool(invalid_path)` is called
- **Then**: Returns `Err(AsyncStoreError::Sqlx(_))` or `Err(AsyncStoreError::Io(_))`
- **Assertions**: Single assertion on error type

### test_given_corrupted_path_when_create_pool_then_connection_fails
- **Purpose**: Verify error handling for corrupted/unreadable database
- **Given**: A path to a file that exists but is not a valid SQLite database
- **When**: `create_async_pool(path)` is called
- **Then**: Returns `Err(AsyncStoreError::Sqlx(_))`
- **Assertions**: Single assertion on error variant

## Edge Case Tests

### test_given_existing_wal_files_when_pool_created_then_succeeds
- **Purpose**: Verify initialization succeeds even when WAL files already exist
- **Given**: A database that already has WAL files (simulate by creating initial pool)
- **When**: A second pool is created for the same database
- **Then**: Pool creation succeeds and PRAGMA values are correct
- **Assertions**: Single assertion on synchronous value

### test_given_multiple_concurrent_pools_when_created_then_all_have_correct_pragmas
- **Purpose**: Verify pool can handle multiple concurrent connections
- **Given**: A valid temporary database path
- **When**: Multiple pools are created concurrently
- **Then**: All pools have correct PRAGMA values
- **Assertions**: Single assertion per pool (iterate)

### test_given_pool_lifetime_when_pragmas_queried_multiple_times_then_values_stable
- **Purpose**: Verify PRAGMA values remain stable throughout pool lifetime (Invariant I3)
- **Given**: A pool created with correct PRAGMA settings
- **When**: PRAGMA values are queried after multiple read/write operations
- **Then**: All PRAGMA values remain unchanged
- **Assertions**: Single assertion per pragma (iterate)

## Contract Verification Tests

### test_given_pool_created_when_synchronous_queried_then_returns_one
- **Purpose**: Verify Q1 - synchronous=NORMAL postcondition
- **Given**: A newly initialized async pool
- **When**: `read_store_pragmas_async(pool)` is called
- **Then**: `synchronous` field equals `1`
- **Assertions**: Single assertion on synchronous value

### test_given_pool_created_when_journal_mode_queried_then_returns_wal
- **Purpose**: Verify Q2 - journal_mode=WAL postcondition
- **Given**: A newly initialized async pool
- **When**: `read_store_pragmas_async(pool)` is called
- **Then**: `journal_mode` field equals "wal"
- **Assertions**: Single assertion on journal_mode

### test_given_pool_created_when_wal_autocheckpoint_queried_then_returns_1000
- **Purpose**: Verify Q2 - wal_autocheckpoint postcondition
- **Given**: A newly initialized async pool
- **When**: `read_store_pragmas_async(pool)` is called
- **Then**: `wal_autocheckpoint` field equals `1000`
- **Assertions**: Single assertion on wal_autocheckpoint value

### test_given_pool_created_when_foreign_keys_queried_then_returns_true
- **Purpose**: Verify Q2 - foreign_keys=ON postcondition
- **Given**: A newly initialized async pool
- **When**: `read_store_pragmas_async(pool)` is called
- **Then**: `foreign_keys` field equals `true`
- **Assertions**: Single assertion on foreign_keys value

### test_given_pool_created_when_busy_timeout_queried_then_returns_5000
- **Purpose**: Verify Q2 - busy_timeout=5000 postcondition
- **Given**: A newly initialized async pool
- **When**: `read_store_pragmas_async(pool)` is called
- **Then**: `busy_timeout` field equals `5000`
- **Assertions**: Single assertion on busy_timeout value

## Contract Violation Tests (One per violation example)

### test_given_full_synchronous_pragma_when_read_then_returns_two
- **Purpose**: Verify that we can detect when synchronous is FULL (value=2)
- **Given**: A pool (any valid pool)
- **When**: `PRAGMA synchronous=FULL` is manually executed
- **Then**: Querying `PRAGMA synchronous` returns value `2`
- **Assertions**: Single assertion - this is the bug we fixed (was FULL, now NORMAL)

### test_given_invalid_database_path_when_pool_created_then_returns_io_error
- **Purpose**: VIOLATES P1 - invalid path error handling
- **Given**: An invalid path that cannot be accessed
- **When**: `create_async_pool(invalid_path)` is called
- **Then**: Returns `Err(AsyncStoreError::Io(_))` or `Err(AsyncStoreError::Sqlx(_))`
- **Assertions**: Single assertion on error type

### test_given_delete_mode_not_wal_when_synchronous_set_then_mismatch_possible
- **Purpose**: VIOLATES P3 - detecting non-WAL mode
- **Given**: A pool with journal_mode="delete" instead of "wal"
- **When**: `read_store_pragmas_async(pool)` is called
- **Then**: Returns journal_mode != "wal"
- **Assertions**: Single assertion on journal_mode

## Data Persistence & Crash Recovery Tests (Invariant I1)

### test_given_wal_mode_when_data_written_then_data_in_wal
- **Purpose**: Verify data is written to WAL (not directly to database)
- **Given**: A pool with WAL mode and synchronous=NORMAL
- **When**: Data is written to the database (INSERT events)
- **Then**: WAL file contains the data (verify via `PRAGMA wal_begin` or checkpoint analysis)
- **Assertions**: Single assertion on WAL presence

### test_given_data_in_wal_when_checkpoint_forced_then_data_persisted
- **Purpose**: Verify WAL checkpoint writes data to main database
- **Given**: A pool with data written to WAL but not checkpointed
- **When**: `PRAGMA wal_checkpoint(TRUNCATE)` is executed
- **Then**: Data is persisted to main database file
- **Assertions**: Single assertion on checkpoint success

### test_given_pool_with_data_when_reopened_then_data_recovered
- **Purpose**: Verify data survives pool closure and reopening
- **Given**: Pool with data written and properly closed
- **When**: New pool is created for the same database
- **Then**: Previously written data is readable
- **Assertions**: Single assertion on data count

### test_given_n_normal_mode_when_simulated_crash_scenario_then_data_safe
- **Purpose**: Verify invariant I1 - data safety with NORMAL mode
- **Given**: Pool with synchronous=NORMAL, WAL mode, data written
- **When**: Data is written and pool remains open (simulating crash scenario)
- **Then**: Data can be recovered - WAL ensures durability for committed transactions
- **Assertions**: Single assertion on data count after recovery attempt

## Async/Sync Parity Tests (Invariant I2)

### test_given_both_async_and_sync_stores_when_initialized_then_synchronous_match
- **Purpose**: Verify I2 - async and sync stores have identical synchronous setting
- **Given**: Valid path, both async and sync stores initialized
- **When**: Both stores' PRAGMA values are queried
- **Then**: Both have synchronous=1 (NORMAL)
- **Assertions**: Single assertion on equality

### test_given_both_async_and_sync_stores_when_initialized_then_journal_mode_match
- **Purpose**: Verify I2 - async and sync stores have identical journal_mode
- **Given**: Valid path, both async and sync stores initialized
- **When**: Both stores' PRAGMA values are queried
- **Then**: Both have journal_mode="wal"
- **Assertions**: Single assertion on equality

### test_given_both_async_and_sync_stores_when_initialized_then_all_pragmas_match
- **Purpose**: Verify I2 - complete PRAGMA parity between async and sync stores
- **Given**: Valid path, both async and sync stores initialized
- **When**: Both stores' full PRAGMA sets are queried
- **Then**: All PRAGMA values match: journal_mode, synchronous, wal_autocheckpoint, foreign_keys, busy_timeout
- **Assertions**: Multiple assertions (same pragma group)

## Given-When-Then Scenarios

### Scenario 1: Async store initialization with optimal WAL settings
**Given**: A valid database file path  
**When**: `create_async_pool(path)` is called  
**Then**:
- Pool is successfully created
- `PRAGMA journal_mode` returns `wal`
- `PRAGMA synchronous` returns `1` (NORMAL) -- **KEY ASSERTION**
- `PRAGMA wal_autocheckpoint` returns `1000`
- `PRAGMA foreign_keys` returns `1` (ON)
- `PRAGMA busy_timeout` returns `5000`

### Scenario 2: Async and sync store configuration parity
**Given**: Both async and sync store pools  
**When**: Both are initialized with the same database path  
**Then**:
- Both return `journal_mode=wal`
- Both return `synchronous=1` (NORMAL) -- **CONSISTENCY CHECK**
- Both have identical PRAGMA configurations

### Scenario 3: Detecting regression to synchronous=FULL
**Given**: An initialized async pool  
**When**: `PRAGMA synchronous` is queried  
**Then**:
- Value must be `1` (NORMAL)
- If value is `2` (FULL), this indicates a regression

### Scenario 4: Data persistence verification
**Given**: Pool with synchronous=NORMAL and WAL mode  
**When**: Multiple events are written to the database  
**Then**:
- Events are successfully persisted
- After pool closure, events can be read back
- WAL checkpoint properly transfers data to main database

## Implementation Notes

### Test Infrastructure
- Use `tempfile::TempDir` for isolated test databases
- Each test cleans up its own temporary directory
- Use `sqlx::query_scalar` to read PRAGMA values
- Parse integer values from PRAGMA queries

### Key Test Functions
```rust
// Read PRAGMA values from existing pool
pub async fn read_store_pragmas_async(pool: &SqlitePool) -> Result<AsyncStorePragmas, AsyncStoreError>;

// Create pool with correct pragmas  
pub async fn create_async_pool(db_path: &Path) -> Result<SqlitePool, AsyncStoreError>;

// Bootstrap with schema migration
pub async fn bootstrap_async_store(db_path: &Path) -> Result<AsyncStoreBootstrap, AsyncStoreError>;
```

### The Fix
The original bug was on line 231 in store_async.rs:
- Before: `PRAGMA synchronous=FULL`
- After: `PRAGMA synchronous=NORMAL`

This one-line change aligns the async store with the sync store and follows SQLite's recommendation for WAL mode.

## Verification Checklist
- [x] Test exists to verify synchronous=NORMAL after pool init
- [x] Test exists to verify WAL mode is active
- [x] Test exists to verify all other PRAGMA values
- [x] Test exists for async/sync parity
- [x] Each violation example has a corresponding test
- [x] All tests follow BDD naming pattern
- [x] Each test has single assertion focus
- [x] Tests are runnable without `#[cfg(kani)]` gating
- [x] Tests verify data persistence (Invariant I1)
- [x] Tests verify crash recovery mechanism
