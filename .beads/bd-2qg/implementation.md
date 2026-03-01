bead_id: bd-2qg
bead_title: append-idempotency-behavior: return no-op success for exact duplicates
phase: p1
updated_at: 2026-03-01T00:00:00Z

# Implementation: append-idempotency-behavior

## Summary

Implemented idempotent append behavior in the event store (`diagram_tool/src/store.rs`) that returns no-op success for exact duplicate operations while rejecting duplicates with conflicting payloads.

## Changes Made

### 1. Added `DuplicateKind` Enum

```rust
/// Kind of duplicate detected during idempotent append
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    /// Same op_id with identical payload - return existing outcome (no-op)
    Exact,
    /// Same op_id with different payload - return error
    Conflict,
}
```

### 2. Implemented `classify_duplicate` Function

```rust
pub fn classify_duplicate(
    existing: &EventRecord,
    incoming: &EventEnvelope,
) -> Result<DuplicateKind, StoreError>
```

This function compares the payload of an existing event record with an incoming envelope to determine if the duplicate should be treated as a no-op (exact match) or rejected as a conflict.

### 3. Implemented `append_idempotent` Function

```rust
pub fn append_idempotent(
    conn: &mut Connection,
    op: EventEnvelope,
) -> Result<AppendOutcome, StoreError>
```

This function implements idempotent append semantics:
- If the op_id is new, appends the event and returns the new outcome
- If the op_id exists with an identical payload, returns the existing outcome (no-op)
- If the op_id exists with a different payload, returns `DuplicateWithConflict` error

## Implementation Details

The implementation follows the contract specification:

1. **Lookup First**: Uses `lookup_existing_op` to check if an operation with the given op_id already exists.

2. **New Operation Path**: If no existing operation is found, delegates to `append_event` for standard append behavior.

3. **Duplicate Classification**: If an existing operation is found, uses `classify_duplicate` to determine if it's an exact match or conflict.

4. **Exact Duplicate Handling**: Returns the existing `AppendOutcome` without modifying the database.

5. **Conflict Handling**: Returns `StoreError::DuplicateWithConflict` with the conflicting op_id.

## Files Modified

- `diagram_tool/src/store.rs`:
  - Added `DuplicateKind` enum (lines 786-793)
  - Added `classify_duplicate` function (lines 795-822)
  - Added `append_idempotent` function (lines 824-855)
  - Added 9 comprehensive tests (lines 2090-2328)

## Constraints Satisfied

- No use of `unwrap` or `expect` in implementation
- All fallible operations use `Result<T, StoreError>`
- No `panic!`, `todo!`, or `unimplemented!` macros
- All tests pass

## Test Coverage

1. `test_classify_duplicate_exact_match` - Verifies exact payload matching
2. `test_classify_duplicate_conflict` - Verifies conflict detection
3. `test_append_idempotent_new_operation` - New operation creates revision
4. `test_append_idempotent_exact_duplicate_returns_existing` - Exact duplicate returns existing
5. `test_append_idempotent_conflicting_duplicate_returns_error` - Conflict returns error
6. `test_append_idempotent_preserves_revision_on_duplicate` - Revision unchanged on duplicate
7. `test_append_idempotent_multiple_different_ops` - Multiple distinct operations work
8. `test_duplicate_kind_equality` - Enum equality works correctly
9. `test_append_idempotent_with_different_operation_types` - Works with various DomainOp types
