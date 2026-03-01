bead_id: bd-2qg
bead_title: append-idempotency-behavior: return no-op success for exact duplicates
phase: p0
updated_at: 2026-03-01T00:00:00Z

# Contract: append-idempotency-behavior

## Summary

Implement idempotent append behavior in the event store that returns no-op success for exact duplicate operations, while rejecting duplicates with conflicting payloads.

## Preconditions

1. **Rust Contract Signature:**
   ```rust
   fn append_idempotent(conn: &mut Connection, op: EventEnvelope) -> Result<AppendOutcome, StoreError>
   ```

2. **Rust Error Contract:**
   ```rust
   enum StoreError {
       DuplicateWithConflict(String),  // Duplicate op_id with different payload
       Serialization(String),
       Sqlite(rusqlite::Error),
       // ... existing variants
   }
   ```

3. **Rust Classification Signature:**
   ```rust
   fn classify_duplicate(existing: &EventRecord, incoming: &EventEnvelope) -> Result<DuplicateKind, StoreError>
   ```

4. **Legacy code path for this slice is identified and removable in one commit**

5. **Dependency:** `bd-1ua` (append-idempotency-index) must be closed - unique index on `operation_id` exists

## Postconditions

1. **Exact Duplicate Handling:**
   - If an operation with the same `op_id` AND identical payload already exists, return the existing `AppendOutcome` (no-op success)
   - No new database row is created for exact duplicates
   - The revision number remains unchanged

2. **Conflicting Duplicate Handling:**
   - If an operation with the same `op_id` but DIFFERENT payload exists, return `StoreError::DuplicateWithConflict`
   - The error message must include the conflicting `op_id`

3. **Classification Helper:**
   - `classify_duplicate` must return `DuplicateKind::Exact` for identical payloads
   - `classify_duplicate` must return `DuplicateKind::Conflict` for different payloads

4. **Legacy path is deleted or unreachable by compile-time guarantees**

5. **Replacement path passes focused tests with no fallback to removed code**

## Invariants

1. No migration path is introduced
2. No dual-write compatibility path exists
3. All fallible operations use typed Result errors
4. Zero unwrap or expect calls in implementation
5. `op_id` uniqueness is enforced at the storage layer

## DuplicateKind Enum

```rust
pub enum DuplicateKind {
    Exact,     // Same op_id, same payload -> return existing outcome
    Conflict,  // Same op_id, different payload -> return error
}
```

## Behavior Specification

### Happy Path: New Operation
```
GIVEN: No existing operation with op_id "op-123"
WHEN: append_idempotent is called with EventEnvelope { op_id: "op-123", ... }
THEN: Returns Ok(AppendOutcome { revision: N+1, op_id: "op-123", timestamp: T })
AND:  New row is inserted into events table
```

### Happy Path: Exact Duplicate
```
GIVEN: Existing operation with op_id "op-123" and payload P
WHEN: append_idempotent is called with EventEnvelope { op_id: "op-123", payload: P, ... }
THEN: Returns Ok(AppendOutcome { revision: N, op_id: "op-123", timestamp: existing_timestamp })
AND:  No new row is inserted
AND:  Revision number unchanged
```

### Error Path: Conflicting Duplicate
```
GIVEN: Existing operation with op_id "op-123" and payload P1
WHEN: append_idempotent is called with EventEnvelope { op_id: "op-123", payload: P2, ... }
      WHERE P1 != P2
THEN: Returns Err(StoreError::DuplicateWithConflict("op-123"))
AND:  No new row is inserted
AND:  Database state unchanged
```

## Acceptance Tests

### Test 1: test_append_idempotent_new_operation
- Given: Empty database
- When: append_idempotent with new op_id
- Then: Returns Ok with revision 1

### Test 2: test_append_idempotent_exact_duplicate_returns_existing
- Given: Database with operation op-123 at revision 1
- When: append_idempotent with same op_id and identical payload
- Then: Returns Ok with revision 1 (unchanged)

### Test 3: test_append_idempotent_conflicting_duplicate_returns_error
- Given: Database with operation op-123 with payload P1
- When: append_idempotent with same op_id but different payload P2
- Then: Returns Err(DuplicateWithConflict("op-123"))

### Test 4: test_classify_duplicate_exact
- Given: EventRecord and EventEnvelope with identical payloads
- When: classify_duplicate is called
- Then: Returns Ok(DuplicateKind::Exact)

### Test 5: test_classify_duplicate_conflict
- Given: EventRecord and EventEnvelope with different payloads
- When: classify_duplicate is called
- Then: Returns Ok(DuplicateKind::Conflict)

### Test 6: test_append_idempotent_preserves_revision_on_duplicate
- Given: Database at revision 5 with operation op-123
- When: append_idempotent with exact duplicate of op-123
- Then: Revision remains 5

### Test 7: test_append_idempotent_multiple_different_ops
- Given: Database with operations op-1, op-2 at revisions 1, 2
- When: append_idempotent with new op-3
- Then: Returns revision 3

## Implementation Tasks

1. Add `DuplicateKind` enum to store.rs
2. Implement `classify_duplicate` function
3. Implement `append_idempotent` function that:
   - Looks up existing operation by op_id
   - If not found, delegates to `append_event`
   - If found with exact match, returns existing outcome
   - If found with conflict, returns `DuplicateWithConflict` error
4. Write comprehensive tests for all paths
5. Ensure no unwrap/expect in implementation

## Files to Modify

- `diagram_tool/src/store.rs` - Add `DuplicateKind`, `classify_duplicate`, `append_idempotent`

## Constraints

- MUST NOT use unwrap or expect
- MUST use Result<T, StoreError> for all fallible operations
- MUST NOT introduce panic!, todo!, or unimplemented!
- MUST pass `moon run :ci`
