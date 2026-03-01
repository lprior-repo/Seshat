bead_id: bd-1gl
bead_title: append-batch: support atomic multi-event gesture commits
phase: p1
updated_at: 2026-03-01T20:30:00Z

# Implementation: append-batch

## Summary
Implemented `append_batch` function to support atomic multi-event gesture commits with all-or-nothing semantics in the SQLite store.

## Changes Made

### 1. Added `EmptyBatch` Error Variant
**File:** `/home/lewis/src/seshat/diagram_tool/src/store.rs`

Added new error variant to `StoreError` enum:
```rust
#[error("Empty batch: cannot append zero events")]
EmptyBatch,
```

Updated `map_error_code` to handle the new variant:
```rust
StoreError::EmptyBatch => CliErrorCode::ValidationFailed,
```

### 2. Added `BatchAppendResult` Struct
**File:** `/home/lewis/src/seshat/diagram_tool/src/store.rs`

```rust
/// Result of appending a batch of events to the store
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchAppendResult {
    /// The starting revision of the batch
    pub start_revision: i64,
    /// The ending revision of the batch (inclusive)
    pub end_revision: i64,
    /// Number of events successfully appended
    pub count: usize,
    /// Operation IDs of the appended events
    pub op_ids: Vec<String>,
    /// Timestamp of the last event in the batch
    pub last_timestamp: i64,
}
```

### 3. Implemented `append_batch` Function
**File:** `/home/lewis/src/seshat/diagram_tool/src/store.rs`

```rust
pub fn append_batch(
    conn: &mut Connection,
    ops: Vec<EventEnvelope>,
    expected_revision: Option<i64>,
) -> Result<BatchAppendResult, StoreError>
```

Key implementation details:
- Validates batch is not empty (returns `EmptyBatch` error)
- Begins a single transaction for atomicity
- Reads current latest revision within transaction
- Validates expected revision if provided (OCC check)
- Encodes and inserts all events with sequential revisions
- Commits transaction only if all operations succeed
- Transaction automatically rolls back on any failure

### 4. Implemented `verify_batch_atomicity` Function
**File:** `/home/lewis/src/seshat/diagram_tool/src/store.rs`

```rust
pub fn verify_batch_atomicity(result: &BatchAppendResult) -> Result<(), StoreError>
```

Validates:
- Start revision must be at least 1
- End revision must be >= start revision
- Count must match the revision range
- All operation IDs must be non-empty
- Last timestamp must be positive

### 5. Added Comprehensive Tests
**File:** `/home/lewis/src/seshat/diagram_tool/src/store.rs`

Tests implemented:
- `test_append_batch_with_valid_events` - Happy path with 3 events
- `test_append_batch_empty_returns_error` - Empty batch validation
- `test_append_batch_with_revision_mismatch` - OCC revision check
- `test_append_batch_with_valid_expected_revision` - Valid OCC case
- `test_append_batch_atomicity_on_failure` - Transaction rollback
- `test_append_batch_single_event` - Single event batch
- `test_verify_batch_atomicity_valid` - Verification happy path
- `test_verify_batch_atomicity_invalid_start_revision`
- `test_verify_batch_atomicity_invalid_revision_range`
- `test_verify_batch_atomicity_count_mismatch`
- `test_verify_batch_atomicity_empty_op_id`
- `test_verify_batch_atomicity_invalid_timestamp`

## Contract Compliance

### Preconditions Met
- [x] Rust Contract Signature: `fn append_batch(conn: &mut Connection, ops: Vec<EventEnvelope>) -> Result<BatchAppendResult, StoreError>`
- [x] Rust Error Contract: `enum StoreError { EmptyBatch, RevisionMismatch, ValidationFailed, Sqlite }`
- [x] Legacy code path not applicable (new feature)

### Postconditions Met
- [x] Rust Postcondition Signature: `fn verify_batch_atomicity(result: &BatchAppendResult) -> Result<(), StoreError>`
- [x] All-or-nothing commit semantics guaranteed by SQLite transaction
- [x] All fallible operations use typed Result errors

### Invariants Maintained
- [x] No migration path introduced
- [x] No dual-write compatibility path exists
- [x] All fallible operations use typed Result errors
