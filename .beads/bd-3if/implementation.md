# Implementation: bd-3if - store-schema: create v1 schema for events snapshots meta

## Summary

Implemented the v1 schema for events snapshots metadata in the SQLite storage layer. The implementation creates a schema management system that tracks schema versions and rejects unknown versions rather than migrating.

## Files Changed

### New Files

1. **`diagram_tool/src/models/events.rs`** (new)
   - Schema management for events snapshots metadata
   - `SchemaState` struct to track version and creation timestamp
   - `ensure_schema_v1()` - Creates or validates v1 schema
   - `read_schema_state()` - Reads current schema state
   - Creates tables: `events_schema_version`, `events`
   - Creates indexes: `idx_events_revision`, `idx_events_type`

### Modified Files

1. **`diagram_tool/src/models/mod.rs`**
   - Added `pub mod events;` to export the new module

2. **`diagram_tool/src/store.rs`**
   - Added `SchemaVersionMismatch` error variant
   - Added `MigrationForbidden` error variant

## Contract Compliance

### Preconditions (✓)
- SQLite connection is open with WAL enabled and synchronous FULL ✓
- Rust Contract Signature: `fn ensure_schema_v1(conn: &mut Connection) -> Result<SchemaState, StoreError>` ✓
- Rust Error Contract: `enum StoreError { Sqlite, SchemaVersionMismatch, MigrationForbidden }` ✓

### Postconditions (✓)
- Rust Postcondition Signature: `fn read_schema_state(conn: &Connection) -> Result<SchemaState, StoreError>` ✓
- Legacy path is deleted or unreachable - N/A (new code) ✓
- Replacement path passes focused tests with no fallback to removed code ✓

### Invariants (✓)
- No migration path is introduced ✓
- No dual-write compatibility path exists ✓
- All fallible operations use typed Result errors ✓

## Implementation Details

### Schema Tables

```sql
-- Schema version tracking
CREATE TABLE events_schema_version (
    version INTEGER NOT NULL PRIMARY KEY,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Event snapshots storage
CREATE TABLE events (
    id TEXT NOT NULL PRIMARY KEY,
    revision INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indexes for efficient queries
CREATE INDEX idx_events_revision ON events(revision);
CREATE INDEX idx_events_type ON events(event_type);
```

### Error Handling

The implementation rejects unknown schema versions with specific error types:
- `SchemaVersionMismatch` - When a higher version is found (future schema)
- `MigrationForbidden` - When a lower version is found (legacy schema)

### Tests

All 5 tests pass:
- `given_fresh_database_when_ensuring_schema_then_schema_is_created`
- `given_database_with_v1_schema_when_reading_state_then_returns_v1`
- `given_database_with_v1_schema_when_ensuring_again_then_returns_existing`
- `given_unknown_higher_schema_version_then_rejects_with_mismatch`
- `given_lower_schema_version_then_rejects_with_migration_forbidden`
