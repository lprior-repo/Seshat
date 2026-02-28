bead_id: bd-l79
bead_title: qa-matrix: build integration and failure regression suite
phase: p0
updated_at: 2026-02-28T21:54:46Z

# Contract: qa-matrix integration test suite

## Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - Input JSON parses into the expected command payload
  - Diagram identifier resolves to a known or creatable single-diagram slot

## Postconditions
- state_changes:
  - Command exits with deterministic exit code and JSONL finish event
  - Persisted diagram remains schema-valid and DAG-valid on success
- return_guarantees: []

## Invariants
- Revision is monotonic and only server-owned
- Invalid mutations never reach UI broadcast

## Acceptance Criteria
1. Integration tests for stale revisions
2. Integration tests for DAG failures
3. Integration tests for lock contention
4. Integration tests for rollback safety
5. Integration tests for redb/file consistency under concurrent writes

## Research Requirements
- Read: diagram_tool/src/cli.rs
- Read: diagram_tool/src/backend.rs
- Read: diagram_tool/src/patch.rs
- Read: diagram_tool/src/models/document.rs

## Implementation Tasks
1. Write failing integration test for command JSONL format and exit code map
2. Write failing test for rejection path preserving last-known-good state
3. Add integration tests for stale revisions, DAG failures, lock contention, rollback safety, and redb/file consistency under concurrent writes
4. Add structured error-code mapping and JSONL serializer
