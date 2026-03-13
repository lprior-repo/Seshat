# Contract Specification: seshat-1pz

## Context
- **Feature**: Optimize PRAGMA synchronous for WAL mode in async store
- **Bead**: seshat-1pz
- **Domain terms**:
  - WAL: Write-Ahead Logging - SQLite's default journaling mode
  - synchronous: SQLite PRAGMA controlling durability vs performance tradeoff
    - `FULL` (value=2): Maximum durability, slowest write performance
    - `NORMAL` (value=1): Optimal balance for WAL mode, recommended by SQLite docs
    - `OFF` (value=0): No durability guarantee
  - PRAGMA: SQLite configuration statements executed at connection time
  - Data persistence: The ability to recover data after application crash
  - WAL checkpoint: The process of writing WAL data back to the main database
- **Assumptions**:
  - The database is initialized with WAL journal_mode
  - This is a one-time configuration at pool creation
  - The fix has been applied: line 231 changed from FULL to NORMAL
- **Open questions**: None - the fix is clear from the comparison between store_async.rs and store_sqlx.rs

## Preconditions
- **P1**: The async pool initialization function must receive a valid database path
  - Type enforcement: `Path` type, validated at runtime
- **P2**: The SQLite connection must be successfully established before PRAGMA execution
  - Type enforcement: Result error variant `AsyncStoreError::Sqlx`
- **P3**: The database must be using WAL journal_mode before setting synchronous
  - Type enforcement: Runtime check, query journal_mode first

## Postconditions
- **Q1**: After `create_async_pool` completes, `PRAGMA synchronous` must return value `1` (NORMAL)
  - Enforcement: Runtime verification via `read_store_pragmas_async()`
- **Q2**: The following PRAGMA values must be set correctly:
  - `journal_mode` = WAL
  - `synchronous` = NORMAL (1) -- THIS IS THE FIX
  - `wal_autocheckpoint` = 1000
  - `foreign_keys` = ON
  - `busy_timeout` = 5000

## Invariants
- **I1**: Data remains safe against application crashes when using WAL mode with synchronous=NORMAL
  - This is guaranteed by SQLite's WAL mechanism: data is either in WAL or committed to database
  - WAL autocheckpoint ensures periodic persistence
- **I2**: The async store and sync store should use identical PRAGMA configurations for consistency
  - Verified by comparing `AsyncStorePragmas` with `SyncStorePragmas`
- **I3**: After pool creation, all PRAGMA settings remain stable throughout pool lifetime
  - Verified by querying PRAGMA values at multiple points

## Error Taxonomy
- **Error::InvalidPath**: Returned if the database path is invalid or inaccessible
  - Trigger: Path does not exist, permission denied, or not a valid file path
- **Error::ConnectionFailed**: Returned if SQLite connection cannot be established
  - Trigger: Database locked, corrupt database, or resource exhaustion
- **Error::PragmaExecutionFailed**: Returned if a PRAGMA statement fails
  - Trigger: Invalid PRAGMA value, SQLite version incompatibility
- **Error::ConfigurationMismatch**: Returned if synchronous PRAGMA returns unexpected value after initialization
  - Trigger: Race condition, unexpected PRAGMA override, or bug in pool creation

## Contract Signatures
```rust
/// Initialize async SQLite pool with optimized WAL pragmas
/// Returns: Result<SqlitePool, AsyncStoreError>
pub async fn create_async_pool(db_path: &Path) -> Result<SqlitePool, AsyncStoreError>;

/// Read current PRAGMA values from the pool
/// Returns: Result<AsyncStorePragmas, AsyncStoreError>
pub async fn read_store_pragmas_async(pool: &SqlitePool) -> Result<AsyncStorePragmas, AsyncStoreError>;

/// Bootstrap async store with schema migration
/// Returns: Result<AsyncStoreBootstrap, AsyncStoreError>
pub async fn bootstrap_async_store(db_path: &Path) -> Result<AsyncStoreBootstrap, AsyncStoreError>;
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Valid db_path | Runtime-checked | `Path` type, file system validation via `std::fs::metadata` |
| Connection success | Result error | `Result<SqlitePool, AsyncStoreError::Sqlx>` |
| WAL mode active | Runtime verification | Query `PRAGMA journal_mode` before setting synchronous |

| Postcondition | Enforcement Level | Implementation |
|---|---|---|
| synchronous=NORMAL | Runtime verification | Query `PRAGMA synchronous` after pool init, assert value == 1 |
| All PRAGMA values correct | Runtime verification | Query each PRAGMA and assert expected values |

## Violation Examples (REQUIRED)

### Precondition Violations

- **VIOLATES P1**: Calling `create_async_pool(Path::new("/nonexistent/invalid/path.db"))`
  - Should produce `Err(AsyncStoreError::Sqlx(_))` or `Err(AsyncStoreError::Io(_))`
  - Concrete: Connection string "sqlite:/nonexistent/invalid/path.db?mode=rwc" fails to connect

- **VIOLATES P2**: Calling `create_async_pool` with an empty path
  - Should produce `Err(AsyncStoreError::Sqlx(_))`
  - Concrete: `Path::new("")` is invalid for SQLite connection

- **VIOLATES P3**: Setting synchronous=NORMAL without WAL mode first
  - Should still work but is not the recommended configuration
  - Concrete: Query journal_mode returns "delete" instead of "wal"

### Postcondition Violations

- **VIOLATES Q1**: Calling `create_async_pool` and then querying `PRAGMA synchronous` returns `2` (FULL) instead of `1` (NORMAL)
  - Should produce `Err(AsyncStoreError::ConfigurationMismatch)` if we add runtime validation
  - Concrete: After pool creation, execute `SELECT * FROM pragma_synchronous`; if value != 1, the contract is violated
  - **This was the original bug**: line 231 had `PRAGMA synchronous=FULL` instead of `NORMAL`

- **VIOLATES Q2**: Any of the required PRAGMA values are incorrect
  - Should produce `Err(AsyncStoreError::ConfigurationMismatch)` if we add validation
  - Concrete: `journal_mode` != "wal", `wal_autocheckpoint` != 1000, etc.

### Invariant Violations

- **VIOLATES I1**: Data loss after simulated crash
  - Test: Write data, force checkpoint, simulate crash, verify data exists
  - If synchronous=NORMAL and WAL mode is used, data must be recoverable

- **VIOLATES I2**: Async and sync stores have different configurations
  - Test: Initialize both stores, compare PRAGMA values
  - Must have identical `synchronous`, `journal_mode`, `wal_autocheckpoint`, etc.

- **VIOLATES I3**: PRAGMA values change during pool lifetime
  - Test: Query PRAGMA values immediately after pool creation and after 100 operations
  - Values must remain stable

## Ownership Contracts (Rust-specific)

- `db_path: &Path` - shared borrow, read-only, no mutation, caller retains ownership
- Return `SqlitePool` - ownership transfer to caller, caller is responsible for pool shutdown
- `pool: &SqlitePool` - shared borrow, read-only access for pragma queries
- `AsyncStoreBootstrap` contains owned `PathBuf` and `SqlitePool`

## Non-goals
- [ ] Changing any other PRAGMA settings beyond synchronous
- [ ] Modifying the sync store (store_sqlx.rs already correct)
- [ ] Adding runtime configuration for synchronous level (hardcoded for now)
- [ ] Implementing actual crash simulation (we verify the mechanism, not actual crashes)

## CI Execution Requirements
- [ ] Tests must run with `cargo test` without special feature flags
- [ ] Tests must NOT be gated behind `#[cfg(kani)]`
- [ ] Tests must verify PRAGMA values in actual SQLite database
- [ ] Tests must be isolated (use temp directories, cleanup after)
- [ ] Tests must use BDD naming pattern: `given_X_when_Y_then_Z`
- [ ] Each test must have exactly ONE assertion focus
