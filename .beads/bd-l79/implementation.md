bead_id: bd-l79
bead_title: qa-matrix: build integration and failure regression suite
phase: p1
updated_at: 2026-02-28T21:55:30Z

# Implementation: qa-matrix integration tests

## Files Changed
- `diagram_tool/tests/cli_e2e.rs` - Added integration tests

## Tests Added

### 1. Stale Revision Tests
- `given_stale_revision_document_when_patch_runs_then_it_fails_with_stale_revision_error` - Tests that patch command fails with `stale_revision` error code when the revision in test op doesn't match current document revision

### 2. DAG Failure Tests
- `given_dag_cycle_document_when_validate_runs_then_it_fails_with_dag_error` - Tests that validate fails with `dag_violation` error for cyclic graphs
- `given_self_loop_edge_when_validate_runs_then_it_fails_with_dag_error` - Tests that validate fails with `dag_violation` error for self-loop edges
- `given_dangling_edge_document_when_validate_runs_then_it_fails_with_dangling_error` - Tests that validate fails with `dangling_reference` error for edges pointing to non-existent nodes

### 3. Rollback Safety Tests
- `given_failed_patch_when_last_known_good_exists_then_original_is_preserved` - Tests that failed patches preserve the last-known-good state in `.lkg/` directory

## Error Codes Required
The tests require the following error codes to be implemented in the JSONL event output:
- `stale_revision` - When patch revision test fails
- `dag_violation` - When DAG constraints are violated (cycles, self-loops)
- `dangling_reference` - When edge references non-existent node

## Acceptance Criteria Coverage
| Criteria | Test Coverage |
|----------|---------------|
| Integration tests for stale revisions | ✓ stale_revision test |
| Integration tests for DAG failures | ✓ dag_violation tests (3 variants) |
| Integration tests for lock contention | TODO: Requires server mode + concurrent CLI |
| Integration tests for rollback safety | ✓ lkg preservation test |
| Integration tests for redb/file consistency | TODO: Requires server feature |
