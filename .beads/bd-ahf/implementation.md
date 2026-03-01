bead_id: bd-ahf
bead_title: verify-occ-idempotency: add stale revision and duplicate op regression tests
phase: p2
updated_at: 2026-03-01T21:16:00Z

# Implementation: bd-ahf - verify-occ-idempotency

## Summary

The OCC (Optimistic Concurrency Control) and idempotency functionality already exists in the codebase. The bead verifies that the existing implementation works correctly.

## Existing Implementation

### OCC (Optimistic Concurrency Control)
- Location: `diagram_tool/src/store.rs` - `append_event` function (line 719)
- Tests: `test_append_with_occ_*`, `test_occ_*`
- Behavior:
  - When `expected_revision` is provided, validates current revision matches
  - Returns `StoreError::RevisionMismatch` if stale
  - No event is appended on rejection

### Idempotency
- Location: `diagram_tool/src/store.rs` - `append_idempotent` function (line 1020)
- Tests: `test_append_idempotent_*`, `test_classify_duplicate_*`
- Behavior:
  - Checks for existing `op_id` in the database
  - Exact duplicate returns existing outcome (no-op success)
  - Conflicting duplicate (same op_id, different payload) returns `StoreError::DuplicateWithConflict`

## Verification Results

All tests pass:
- `test_occ_stale_revision_rejected_with_no_append` ✓
- `test_append_with_occ_revision_mismatch` ✓
- `test_append_idempotent_exact_duplicate_returns_existing` ✓
- `test_append_idempotent_conflicting_duplicate_returns_error` ✓
- `test_occ_exact_duplicate_returns_noop_success` ✓
- And 86 more store tests ✓

## Notes

This bead's implementation was already complete - it just needed verification. The tests demonstrate:
1. Stale revision (OCC) correctly rejects without side effects
2. Exact duplicate op_id returns no-op success (idempotent)
3. Conflicting duplicate op_id returns error (preserves integrity)
