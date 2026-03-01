# Implementation: bd-3ve - storage-bootstrap

## Summary

Implemented SQLite WAL schema and pragma bootstrap functionality for the diagram tool storage layer.

## Changes Made

### New Types Added (`diagram_tool/src/store.rs`)

1. **`CURRENT_SCHEMA_VERSION`** - Public constant (i32 = 1) for schema versioning

2. **`StoreError`** - Error enum (already existed, verified):
   - `Io(std::io::Error)` - I/O errors
   - `Sqlite(rusqlite::Error)` - SQLite errors  
   - `InvalidPragma(String)` - Invalid PRAGMA configuration
   - `SchemaVersionMismatch { expected: i32, found: i32 }` - Schema version mismatch
   - `MigrationForbidden { version: i32 }` - Migration not allowed

3. **`StorePragmas`** - Struct holding SQLite pragma settings:
   - `journal_mode: String`
   - `synchronous: i32`
   - `wal_autocheckpoint: i32`

4. **`StoreBootstrap`** - Result of bootstrapping a new store:
   - `conn: Connection` - SQLite connection
   - `db_path: PathBuf` - Path to database
   - `schema_version: i32` - Current schema version

5. **`StoreConfig`** - Current store configuration:
   - `pragmas: StorePragmas` - Current pragma settings
   - `schema_version: i32` - Current schema version

### New Functions Added

1. **`bootstrap_store(db_path: &Path) -> Result<StoreBootstrap, StoreError>`**
   - Opens/creates database at given path
   - Enforces WAL journal mode and FULL synchronous (2)
   - Runs deterministic schema migration v1
   - Returns bootstrap result with connection and metadata
   - Idempotent on repeated calls

2. **`run_schema_migration(conn: &Connection) -> Result<(), StoreError>`**
   - Internal function for deterministic schema setup
   - Creates `schema_version` table if not exists
   - Creates `events` table for append-only event log if not exists

3. **`current_store_config(conn: &Connection) -> Result<StoreConfig, StoreError>`**
   - Returns current pragma settings and schema version
   - Does not require a reference to the connection in return type

### Tests Added (8 tests)

1. `test_bootstrap_store_creates_database_with_schema` - Happy path bootstrap
2. `test_bootstrap_store_enforces_wal_mode` - WAL mode enforcement
3. `test_bootstrap_store_enforces_synchronous_full` - FULL synchronous enforcement
4. `test_bootstrap_store_with_invalid_path` - Error path for invalid paths
5. `test_bootstrap_store_creates_schema_tables` - Schema table creation
6. `test_current_store_config_returns_pragmas_and_version` - Config retrieval
7. `test_bootstrap_idempotent_on_existing_schema` - Idempotency
8. `test_open_store_with_existing_wal_database` - Interop with open_store

## Requirements Met

- ✅ All fallible functions return `Result<T, Error>`
- ✅ Zero `.unwrap()` or `.expect()` calls in implementation
- ✅ Uses functional patterns (`?`, `map`, `and_then`)
- ✅ Enforces `PRAGMA journal_mode=WAL`
- ✅ Enforces `PRAGMA synchronous=FULL`
- ✅ Schema version management
- ✅ All tests pass (8/8)
- ✅ Zero clippy warnings in new code

## Additional Notes

- Fixed unrelated pre-existing issue in `envelope.rs` (removed `Eq` derive on enum with `f64` fields)
- The `StoreConfig` does not store a clone of the connection to avoid lifetime complexity
- The module is wired for future integration with the rest of the application
