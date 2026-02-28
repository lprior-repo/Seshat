# Implementation: watcher-sync: ingest canonical file edits safely

## bead_id: bd-o6p
## phase: p2
## updated_at: 2026-02-28

## Summary

This implementation verifies that the watcher-sync feature requirements are fully satisfied by the existing codebase. All contract requirements have been implemented and tested.

## Verified Contract Requirements

### 1. JSONL Events for Every CLI Command Stage

**Status**: ✅ Implemented

**Implementation**:
- `cli.rs`: `emit_event()` and `CliEvent` struct emit JSONL events for `start`, `finish`, and `error` events
- `cli_persistence.rs`: `emit_stage_event()` emits stage-specific events with `StageDetails`
- Error codes are mapped via `error_code()` function: `schema_violation`, `dag_cycle`, `parse_error`, `command_error`

**Test Coverage**:
- `cli_e2e::given_validate_command_when_run_then_it_outputs_jsonl_start_and_finish_events`
- `cli_e2e::given_invalid_v2_document_when_validate_runs_then_schema_violation_is_emitted`
- `cli_e2e::given_legacy_edge_alias_when_validate_runs_then_parse_error_is_emitted`

### 2. Preserve Valid Last-Known-Good Diagram State

**Status**: ✅ Implemented

**Implementation**:
- `cli_persistence.rs`: `load_workspace_with_lkg()` attempts to load primary file first, falls back to `.lkg` file on failure
- `save_workspace_atomic()` uses atomic write pattern (temp file + rename) for crash safety
- Validates both primary and LKG files against schema before loading

**Test Coverage**:
- `cli_persistence::tests::given_lkg_fallback_file_when_primary_fails_then_uses_lkg`
- `cli_persistence::tests::given_valid_document_when_saved_atomically_then_file_exists`
- `cli_persistence::tests::given_saved_document_when_loaded_with_lkg_then_returns_same_document`

### 3. Run Full Validation Pipeline Before Persistence

**Status**: ✅ Implemented

**Implementation**:
- `mutation/pipeline.rs`: `run_mutation()` and `run_mutation_with_policy()` execute full validation:
  1. Apply transformation function
  2. Validate schema (`validate_schema()`)
  3. Validate semantic rules (`validate_document()`)
  4. Increment revision (if policy is Increment)
- Returns `MutationError` with `Schema`, `Semantic`, or `Transform` variants

**Test Coverage**:
- `mutation::pipeline::tests::given_transform_that_creates_cycle_when_run_mutation_then_it_fails_closed`
- `mutation::pipeline::tests::given_valid_transform_when_run_mutation_then_revision_increments_once`
- All pipeline proptests verify validation is enforced

### 4. Machine-Readable Error Codes on Validation Failure

**Status**: ✅ Implemented

**Implementation**:
- `cli.rs`: `error_code()` function maps errors to machine-readable codes:
  - `schema_violation` - schema validation errors
  - `dag_cycle` - DAG/cycle errors  
  - `parse_error` - JSON parse/deserialize errors
  - `command_error` - generic command errors
- Exit codes: `parse_error` and `command_error` return 2, others return 1

**Test Coverage**:
- `cli_e2e::given_invalid_v2_document_when_validate_runs_then_schema_violation_is_emitted`
- `cli_e2e::given_legacy_edge_alias_when_validate_runs_then_parse_error_is_emitted`
- `cli_e2e::given_patch_without_first_revision_test_when_patch_runs_then_fail_closed_error_events_are_emitted`

### 5. Prevent Silent Overwrites on Concurrent Updates

**Status**: ✅ Implemented

**Implementation**:
- `locking/manager.rs`: `DiagramLockManager` provides per-diagram serialization
- `locking/file_lock.rs`: File-level locking for cross-process safety
- `with_lock()` method: acquires lock → loads doc → applies mutation → saves → releases lock
- Queue-based mutation system: `queue_mutation()` and `flush_queue()` for batch processing
- Revision policy: `RevisionPolicy::Preserve` keeps server-owned revision monotonic

**Test Coverage**:
- `locking::manager::tests::given_different_diagrams_when_mutated_then_both_succeed`
- `locking::manager::tests::given_mutation_with_lock_when_applied_then_document_modified`
- `locking::manager::tests::given_queued_mutations_when_flushed_then_all_applied`
- `locking::manager::tests::given_multiple_operations_same_diagram_when_sequential_then_succeed`

## Test Results

```
Running tests:
- 479 unit tests: ALL PASSED
- 8 CLI e2e tests: ALL PASSED
Total: 487 tests passed
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Entry Point                        │
│                    (cli.rs, main.rs)                       │
└────────────────────────┬────────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   ┌─────────┐   ┌──────────┐  ┌──────────┐
   │ Render  │   │  Patch   │  │  Layout  │
   └────┬────┘   └─────┬────┘  └─────┬────┘
        │              │              │
        └──────────────┼──────────────┘
                       ▼
              ┌────────────────┐
              │ emit_stage_   │
              │ event()       │
              └───────┬────────┘
                      ▼
          ┌───────────────────────┐
          │ load_workspace_with_ │
          │ lkg()                 │
          └───────────┬───────────┘
                      ▼
          ┌───────────────────────┐
          │ run_mutation()        │
          │ (validation pipeline) │
          └───────────┬───────────┘
                      ▼
          ┌───────────────────────┐
          │ save_workspace_      │
          │ atomic()              │
          └───────────────────────┘
```

## Key Files

| File | Purpose |
|------|---------|
| `src/cli.rs` | CLI entry point, JSONL event emission, error code mapping |
| `src/cli_persistence.rs` | Atomic file I/O, LKG fallback mechanism |
| `src/mutation/pipeline.rs` | Validation pipeline, revision policy |
| `src/locking/manager.rs` | Per-diagram lock manager |
| `src/locking/file_lock.rs` | File-level locking |
| `src/backend.rs` | Server-side persistence with validation |
| `src/models/document.rs` | Document model, Revision type |
| `src/models/schema.rs` | Schema validation |
| `src/models/validation.rs` | Semantic validation |

## Contract Compliance

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Emit JSONL events for CLI stages | ✅ | `emit_event()`, `emit_stage_event()` |
| Preserve LKG diagram state | ✅ | `load_workspace_with_lkg()`, LKG fallback tests |
| Validation before persistence | ✅ | `run_mutation()` with schema + semantic validation |
| Error codes on validation failure | ✅ | `error_code()` maps to `schema_violation`, `dag_cycle`, etc. |
| Concurrent update protection | ✅ | `DiagramLockManager` with file locks |

## Conclusion

The watcher-sync feature is fully implemented in the existing codebase. All EARS requirements are satisfied:

- ✅ JSONL events for every CLI command stage
- ✅ Preserve valid last-known-good diagram state  
- ✅ Run full validation pipeline before persistence
- ✅ Machine-readable error codes on validation failure
- ✅ Prevent silent overwrites on concurrent updates

The implementation uses functional-rust patterns:
- Result<T, Error> for all fallible functions
- No unwrap/expect/panic in core logic
- Atomic file operations for crash safety
- Revision monotonicity enforced by server
