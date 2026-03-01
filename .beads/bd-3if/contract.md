# Contract: bd-3if - store-schema: create v1 schema for events snapshots meta

## Metadata
- bead_id: bd-3if
- bead_title: store-schema: create v1 schema for events snapshots meta
- phase: p0
- updated_at: 2026-03-01T13:04:00Z

## Preconditions
- SQLite connection is open with WAL enabled and synchronous FULL
- Rust Contract Signature: `fn ensure_schema_v1(conn: &mut Connection) -> Result<SchemaState, StoreError>`
- Rust Error Contract: `enum StoreError { Sqlite, SchemaVersionMismatch, MigrationForbidden }`
- Legacy code path for this slice is identified and removable in one commit

## Postconditions
- Rust Postcondition Signature: `fn read_schema_state(conn: &Connection) -> Result<SchemaState, StoreError>`
- Legacy path is deleted or unreachable by compile-time guarantees
- Replacement path passes focused tests with no fallback to removed code

## Invariants
- No migration path is introduced
- No dual-write compatibility path exists
- All fallible operations use typed Result errors

## Implementation Tasks
1. Create v1 schema transaction
2. Reject unknown schema versions instead of migrating
