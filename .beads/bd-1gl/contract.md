bead_id: bd-1gl
bead_title: append-batch: support atomic multi-event gesture commits
phase: p0
updated_at: 2026-03-01T20:22:33Z

# Contract: append-batch

## Summary
Implement `append_batch` function to support atomic multi-event gesture commits with all-or-nothing semantics.

## Preconditions

### System State
- Rust Contract Signature: `fn append_batch(conn: &mut Connection, ops: Vec<EventEnvelope>) -> Result<BatchAppendResult, StoreError>`
- Rust Error Contract: `enum StoreError { EmptyBatch, RevisionMismatch, ValidationFailed, Sqlite }`
- Legacy code path for this slice is identified and removable in one commit

### Required Inputs
- `conn`: Mutable reference to database connection
- `ops`: Vector of EventEnvelope objects representing the batch of events to append

## Postconditions

### State Changes
- Rust Postcondition Signature: `fn verify_batch_atomicity(result: &BatchAppendResult) -> Result<(), StoreError>`
- Legacy path is deleted or unreachable by compile-time guarantees
- Replacement path passes focused tests with no fallback to removed code

### Return Guarantees
- Returns `BatchAppendResult` on success containing batch metadata
- Returns `StoreError` on failure with specific error variant

## Invariants
- No migration path is introduced
- No dual-write compatibility path exists
- All fallible operations use typed Result errors
- Batch operations are atomic (all-or-nothing)

## Error Contract

```rust
enum StoreError {
    EmptyBatch,         // Raised when ops vector is empty
    RevisionMismatch,   // Raised when optimistic concurrency check fails
    ValidationFailed,   // Raised when event validation fails
    Sqlite,             // Raised on database errors
}
```

## Acceptance Tests

### Happy Paths
1. Given: Valid batch of events
   When: append_batch is called
   Then: All events are persisted atomically, BatchAppendResult returned

2. Given: Single event in batch
   When: append_batch is called
   Then: Event is persisted, BatchAppendResult returned

### Error Paths
1. Given: Empty batch
   When: append_batch is called
   Then: Returns StoreError::EmptyBatch

2. Given: Batch with revision mismatch
   When: append_batch is called
   Then: Returns StoreError::RevisionMismatch, no partial writes

3. Given: Invalid event in batch
   When: append_batch is called
   Then: Returns StoreError::ValidationFailed, no partial writes

## Related Files
- diagram_tool/src/backend.rs
- diagram_tool/src/patch.rs
- diagram_tool/src/cli.rs
- diagram_tool/src/models/document.rs

## Dependencies
- bd-1nb (closed): append-occ: implement revision-guarded append transaction
