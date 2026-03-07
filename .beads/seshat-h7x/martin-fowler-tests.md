# Martin Fowler Test Plan

## Overview

This test plan covers the async migration of `snapshot.rs` and `events.rs` from `rusqlite` to `sqlx`. All tests must be `#[tokio::test]` async tests using `SqlitePool` with either in-memory databases or temporary files.

---

## Module: `models/snapshot.rs` (Async)

### Happy Path Tests

#### `test_write_and_load_snapshot_happy_path`
**Given**: Fresh database with one event at revision 1  
**When**: Write snapshot at revision 1, then load projection  
**Then**: 
- `write_snapshot` returns `SnapshotMeta` with revision 1
- `load_projection` returns projection at revision 1

#### `test_latest_snapshot_returns_metadata_after_write`
**Given**: Fresh database with event at revision 1  
**When**: Write snapshot, then call `latest_snapshot`  
**Then**: Returns `Some(SnapshotMeta)` matching written snapshot

#### `test_load_projection_replays_events_after_snapshot`
**Given**: Database with snapshot at revision 0, then event added at revision 1  
**When**: Call `load_projection`  
**Then**: Returns projection at revision 1 (replayed from snapshot + tail)

#### `test_load_projection_with_no_snapshot_falls_back_to_full_replay`
**Given**: Database with event at revision 1, but no snapshot  
**When**: Call `load_projection`  
**Then**: Returns projection at revision 1 (full replay from empty)

#### `test_load_tail_events_returns_events_after_revision`
**Given**: Database with events at revisions 1, 2, 3  
**When**: Call `load_tail_events(1)`  
**Then**: Returns events with op_id "op-2" and "op-3"

#### `test_load_projection_preserves_snapshot_data`
**Given**: 
- Event 1 adds node-1
- Snapshot at revision 1 (contains node-1)
- Event 2 adds node-2
**When**: Call `load_projection`  
**Then**: Projection at revision 2 contains both node-1 and node-2

#### `test_latest_snapshot_returns_highest_revision`
**Given**: Database with snapshots at revisions 1, 2, 3  
**When**: Call `latest_snapshot`  
**Then**: Returns snapshot metadata with revision 3

---

### Error Path Tests

#### `test_snapshot_stale_error_when_revision_behind`
**Given**: Database with event at revision 1  
**When**: Try to write snapshot with `projection.revision = 0`  
**Then**: Returns `Err(SnapshotError::SnapshotStale { expected: 1, found: 0 })`

#### `test_latest_snapshot_returns_none_when_no_snapshots_exist`
**Given**: Fresh database with no snapshots  
**When**: Call `latest_snapshot`  
**Then**: Returns `Ok(None)`

---

### Edge Case Tests

#### `test_given_stale_snapshot_when_load_projection_then_replays_tail`
**Given**: 
- Snapshot written at revision 1
- Then events added to revision 3
**When**: Call `load_projection`  
**Then**: Gracefully handles "stale" snapshot, returns projection at revision 3

#### `test_given_corrupted_payload_when_load_projection_then_returns_serialization_error`
**Given**: Snapshots table contains `'this is not valid json at all'`  
**When**: Call `load_projection`  
**Then**: Returns `Err(SnapshotError::Serialization(...))`, no panic

#### `test_given_truncated_json_payload_when_load_projection_then_returns_serialization_error`
**Given**: Snapshots table contains `'{"version":1,"revision":1,"nodes":{'`  
**When**: Call `load_projection`  
**Then**: Returns `Err(SnapshotError::Serialization(...))`

#### `test_given_semantically_invalid_payload_when_load_projection_then_returns_error`
**Given**: Valid JSON but wrong types: `{"nodes": "should be a map not string", ...}`  
**When**: Call `load_projection`  
**Then**: Returns `Err(SnapshotError::Serialization(...))`

#### `test_given_incompatible_snapshot_format_when_load_projection_then_handles_gracefully`
**Given**: Snapshot with old schema format: `{"schema_version": "v0.1.0-legacy", ...}`  
**When**: Call `load_projection`  
**Then**: Returns `Err(SnapshotError::Serialization(...))`

#### `test_given_snapshot_with_missing_metadata_fields_when_load_then_returns_serialization_error`
**Given**: Snapshot missing required fields: `{"nodes": {}, "edges": {}}`  
**When**: Call `load_projection`  
**Then**: Returns `Err(SnapshotError::Serialization(...))`

---

### Contract Verification Tests

#### `test_command_flow_uses_replacement_implementation_without_legacy_calls`
**Given**: Fresh database with event at revision 1  
**When**: 
- Write snapshot via `write_snapshot`
- Load via `latest_snapshot`
- Load projection via `load_projection`
**Then**: All operations succeed, no rusqlite code paths used

#### `test_invalid_input_returns_typed_error_without_partial_mutation`
**Given**: Database with event at revision 1  
**When**: Try to write snapshot with stale revision  
**Then**: 
- Returns `SnapshotError::SnapshotStale`
- No snapshot row created (transaction rollback)

---

### Async Transaction Tests

#### `test_write_snapshot_uses_transaction_atomically`
**Given**: Fresh database  
**When**: Write snapshot operation  
**Then**: Uses single transaction, commits only on success

#### `test_concurrent_snapshot_writes_succeed_with_proper_isolation`
**Given**: Database at revision 1  
**When**: Two concurrent `write_snapshot` calls for revision 1  
**Then**: Both succeed (INSERT OR REPLACE handles idempotency)

#### `test_load_projection_handles_concurrent_event_appends`
**Given**: Database with snapshot at revision 1  
**When**: Concurrent event append + load_projection  
**Then**: Either sees consistent state or proper error, no corruption

---

## Module: `models/events.rs` (Async)

### Happy Path Tests

#### `test_given_fresh_database_when_ensuring_schema_then_schema_is_created`
**Given**: Fresh database (no tables)  
**When**: Call `ensure_schema_v1`  
**Then**: Returns `SchemaState` with version 1

#### `test_given_database_with_v1_schema_when_reading_state_then_returns_v1`
**Given**: Database with v1 schema already created  
**When**: Call `read_schema_state`  
**Then**: Returns `SchemaState { version: 1, ... }`

#### `test_given_database_with_v1_schema_when_ensuring_again_then_returns_existing`
**Given**: Database where `ensure_schema_v1` was already called  
**When**: Call `ensure_schema_v1` again  
**Then**: Returns same `SchemaState` without error

---

### Error Path Tests

#### `test_given_unknown_higher_schema_version_then_rejects_with_mismatch`
**Given**: Database with schema version 99 manually inserted  
**When**: Call `ensure_schema_v1`  
**Then**: Returns `Err(StoreError::SchemaVersionMismatch { expected: 1, found: 99 })`

#### `test_given_lower_schema_version_then_rejects_with_migration_forbidden`
**Given**: Database with schema version 0 manually inserted  
**When**: Call `ensure_schema_v1`  
**Then**: Returns `Err(StoreError::MigrationForbidden { version: 0 })`

---

### Contract Verification Tests

#### `test_schema_tables_created_correctly`
**Given**: Fresh database  
**When**: After `ensure_schema_v1` succeeds  
**Then**: 
- `events_schema_version` table exists
- `events` table exists with all columns
- `snapshots` table exists with all columns
- All indexes created
- Version record inserted

#### `test_schema_creation_is_idempotent`
**Given**: Database where `ensure_schema_v1` succeeded  
**When**: Call `ensure_schema_v1` multiple times  
**Then**: All calls succeed, only one version row exists

---

## Given-When-Then Scenarios

### Scenario 1: Snapshot Recovery from Corrupted State
**Given**: 
- Database with snapshot at revision 5
- Snapshot payload is corrupted JSON
- Events table has events 1-10

**When**: Call `load_projection`

**Then**:
- Returns `Err(SnapshotError::Serialization(...))`
- No panic
- Database remains in consistent state

---

### Scenario 2: Idempotent Snapshot Write
**Given**: 
- Database at revision 3
- Snapshot already exists for revision 3

**When**: Call `write_snapshot` with projection at revision 3

**Then**:
- Succeeds (INSERT OR REPLACE)
- Only one snapshot row for revision 3
- Returns valid `SnapshotMeta`

---

### Scenario 3: Tail Replay with Many Events
**Given**:
- Snapshot at revision 10
- 100 events added after snapshot (revisions 11-110)

**When**: Call `load_projection`

**Then**:
- Loads snapshot from revision 10
- Replays 100 events
- Returns projection at revision 110
- No memory issues

---

### Scenario 4: Schema Version Mismatch Protection
**Given**:
- Database with schema version 2 (future version)

**When**: Call `ensure_schema_v1`

**Then**:
- Returns `Err(StoreError::SchemaVersionMismatch { expected: 1, found: 2 })`
- No schema changes
- Database remains unmodified

---

### Scenario 5: Concurrent Load and Write Operations
**Given**:
- Database with snapshot at revision 5
- 10 events in queue

**When**:
- Thread A calls `load_projection`
- Thread B calls `write_snapshot` concurrently
- Thread C appends events concurrently

**Then**:
- All operations complete without deadlock
- Final state is consistent
- No data corruption

---

## Test Infrastructure Requirements

### Database Setup
- Use `tempfile::TempDir` for file-based tests
- Use `sqlite::memory:` for simple tests
- Bootstrap with `store::bootstrap_store` for schema setup
- Clean up temp directories after tests

### Async Runtime
- All tests marked `#[tokio::test]`
- Use `tokio::test` with default multi-threaded runtime
- No blocking calls in async context

### Assertion Patterns
```rust
// Happy path
let result = write_snapshot(&pool, &projection).await;
assert!(result.is_ok(), "Should succeed: {:?}", result.err());

// Error path
let result = write_snapshot(&pool, &stale).await;
assert!(matches!(result, Err(SnapshotError::SnapshotStale { .. })));

// No unwraps - use expect or pattern matching
let meta = result.expect("snapshot write should succeed");
```

### Functional Rust Constraints
- ❌ NO `.unwrap()` in tests (use `.expect()` with message or pattern match)
- ❌ NO `.expect()` in production code
- ✅ Railway-oriented: `result.map_err(|e| ...)?`
- ✅ Use `?` operator for error propagation
- ✅ Pattern match for specific error variants

---

## Coverage Matrix

| Function | Happy Path | Error Path | Edge Case | Contract | Async Tx |
|---|---|---|---|---|---|
| `write_snapshot` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `latest_snapshot` | ✅ | ✅ | - | ✅ | - |
| `load_projection` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `load_tail_events` | ✅ | - | - | ✅ | - |
| `ensure_schema_v1` | ✅ | ✅ | - | ✅ | - |
| `read_schema_state` | ✅ | - | - | ✅ | - |

**Total test count**: ~25 tests
**Expected coverage**: >95% of async code paths
