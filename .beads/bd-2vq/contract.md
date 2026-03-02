bead_id: bd-2vq
bead_title: edge-case-bdd-tests-snapshot-recovery
phase: p0
updated_at: 2026-03-02T04:45:30Z

# Contract: BDD Tests for Snapshot Recovery Edge Cases

## Scope

Add comprehensive BDD-style tests for snapshot recovery edge cases in
`diagram_tool/src/models/snapshot.rs`. Tests must follow the existing
`given_*_when_*_then_*` naming convention used throughout the codebase.

## Required Test Cases

### 1. Staleness Detection

```rust
#[test]
fn given_stale_snapshot_when_load_projection_then_returns_stale_error()
```

- A snapshot at revision N exists in the database
- Events have been appended up to revision M where M > N
- The snapshot payload claims revision N
- Verify that attempting to load with stale detection returns appropriate error
- Note: Current implementation loads snapshot + replays tail, so this test
  verifies behavior when snapshot revision < stored revision in metadata

### 2. Deserialization Failures

```rust
#[test]
fn given_corrupted_payload_when_load_projection_then_returns_serialization_error()
```

- A snapshot exists with invalid JSON payload (e.g., truncated, malformed)
- Verify `load_projection` returns `SnapshotError::Serialization`
- Verify no panic, graceful error handling

### 3. Corrupted Payload (Structurally Valid but Semantically Invalid)

```rust
#[test]
fn given_semantically_invalid_payload_when_load_projection_then_returns_error()
```

- A snapshot exists with valid JSON but missing required fields
- Or fields have wrong types (e.g., nodes is a string instead of a map)
- Verify `load_projection` returns appropriate error
- Verify no partial state corruption

### 4. Incompatible Format (Schema Version Mismatch)

```rust
#[test]
fn given_incompatible_snapshot_format_when_load_projection_then_handles_gracefully()
```

- A snapshot exists with an unexpected structure (e.g., old schema version)
- The DiagramProjection struct cannot deserialize from it
- Verify graceful error handling with typed error

### 5. Missing Metadata

```rust
#[test]
fn given_snapshot_with_missing_metadata_fields_when_load_then_returns_serialization_error()
```

- A snapshot row exists but payload lacks required metadata (revision, nodes, etc.)
- Verify `SnapshotError::Serialization` is returned
- Verify fallback or clean error, no panic

## Acceptance Criteria

1. All 5 test cases implemented in `diagram_tool/src/models/snapshot.rs`
2. Tests follow BDD naming convention: `given_*_when_*_then_*`
3. Each test verifies both:
   - Correct error type is returned
   - No panic or unhandled exception occurs
4. Tests use tempfile for isolated test databases
5. Tests are deterministic and can run in parallel
6. All existing tests continue to pass
7. `moon run :test` passes after implementation

## Non-Goals

- Modifying production code (only adding tests)
- Changing error types or adding new error variants
- Testing performance characteristics

## Dependencies

- Existing snapshot.rs infrastructure
- tempfile crate (already in use)
- serde_json for payload manipulation

## Verification

```bash
moon run :quick
moon run :test
```

All tests must pass including new edge case tests.
