# Verification: bd-3if - store-schema: create v1 schema for events snapshots meta

## Test Results

```
$ cargo test --bin diagram_tool models::events::tests -- --nocapture

running 5 tests
test models::events::tests::given_unknown_higher_schema_version_then_rejects_with_mismatch ... ok
test models::events::tests::given_lower_schema_version_then_rejects_with_migration_forbidden ... ok
test models::events::tests::given_database_with_v1_schema_when_ensuring_again_then_returns_existing ... ok
test models::events::tests::given_database_with_v1_schema_when_reading_state_then_returns_v1 ... ok
test models::events::tests::given_fresh_database_when_ensuring_schema_then_schema_is_created ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 470 filtered out; finished in 0.00s
```

## Contract Verification

### Preconditions
- [x] SQLite connection is open with WAL enabled and synchronous FULL (handled by caller)
- [x] Function signature: `fn ensure_schema_v1(conn: &mut Connection) -> Result<SchemaState, StoreError>`
- [x] Error enum: `enum StoreError { Sqlite, SchemaVersionMismatch, MigrationForbidden }`

### Postconditions
- [x] Function signature: `fn read_schema_state(conn: &Connection) -> Result<SchemaState, StoreError>`
- [x] Legacy path deleted/unreachable: N/A (new code)
- [x] Replacement path passes tests: All 5 tests pass

### Invariants
- [x] No migration path introduced
- [x] No dual-write compatibility path
- [x] All fallible operations use typed Result errors

## Quality Gates

- [x] Cargo fmt passes
- [x] Tests pass (5/5)
- [x] Code compiles without errors (warnings are pre-existing in other files)

## Warnings

The following warnings exist in the codebase but are unrelated to this implementation:
- Pre-existing clippy warnings in `cli.rs`, `cli_events_tests.rs`, `store.rs`, `mutation/pipeline.rs`
- LSP errors in `ui/canvas.rs` and `ui/canvas/canvas_view.rs` (render errors, unrelated)
- Missing file `cli_events_tests.rs` referenced in main.rs (pre-existing issue)
