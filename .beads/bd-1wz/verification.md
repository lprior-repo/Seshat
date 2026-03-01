# Verification: bd-1wz - cli: fix dangling_reference and stale_revision codes

## Metadata
- bead_id: bd-1wz
- bead_title: cli: fix dangling_reference and stale_revision codes
- phase: p2
- updated_at: 2026-03-01T23:10:00Z

## Acceptance Criteria Verification

### 1. Given stale revision error, when `error_code` called, returns "stale_revision"

**Status**: PASS

Unit test `given_stale_revision_error_when_error_code_called_then_returns_stale_revision` verifies:
```rust
let err = anyhow!("stale_revision: test failed at /revision: expected Some(999) but got Some(1)");
let code = error_code(&err);
assert_eq!(code, "stale_revision");
```

### 2. Given stale revision error, when CLI runs, emits error event with "stale_revision"

**Status**: PASS

Manual verification shows:
```json
{"event":"error","command":"patch","ok":false,"code":"stale_revision","message":"stale_revision: test failed at /revision: expected Some(Number(999)) but got Some(Number(1))"}
```

E2E test `given_stale_revision_document_when_patch_runs_then_it_fails_with_stale_revision_error` passes.

### 3. Given stale revision error, when CLI runs, finish event has "stale_revision" code

**Status**: PASS

Manual verification shows:
```json
{"event":"finish","command":"patch","ok":false,"code":"stale_revision","message":null}
```

### 4. Given stale revision error, exit code is 1 (not 2)

**Status**: PASS

Manual verification shows:
```
Exit code: 1
```

Unit test `given_stale_revision_error_when_exit_code_called_then_returns_1` verifies:
```rust
let err = anyhow!("stale_revision: test failed at /revision");
let code = exit_code(&err);
assert_eq!(code, 1);
```

### 5. All existing e2e tests pass

**Status**: PASS

```
running 13 tests
test given_valid_document_when_validate_command_runs_then_it_succeeds ... ok
test given_dangling_edge_document_when_validate_runs_then_it_fails_with_dangling_error ... ok
test given_patch_without_first_revision_test_when_patch_runs_then_fail_closed_error_events_are_emitted ... ok
test given_invalid_v2_document_when_validate_runs_then_schema_violation_is_emitted ... ok
test given_valid_document_when_layout_command_runs_then_output_contains_nodes_and_edges ... ok
test given_legacy_edge_alias_when_validate_runs_then_parse_error_is_emitted ... ok
test given_self_loop_edge_when_validate_runs_then_it_fails_with_dag_error ... ok
test given_stale_revision_document_when_patch_runs_then_it_fails_with_stale_revision_error ... ok
test given_dag_cycle_document_when_validate_runs_then_it_fails_with_dag_error ... ok
test given_validate_command_when_run_then_it_outputs_jsonl_start_and_finish_events ... ok
test given_valid_document_when_render_svg_command_runs_then_svg_file_is_generated ... ok
test given_failed_patch_when_last_known_good_exists_then_original_is_preserved ... ok
test given_valid_patch_when_patch_command_runs_then_it_writes_updated_document ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Invariant Verification

- [x] No changes to error event structure or JSONL format
- [x] No changes to existing error patterns (dangling, dag, schema, semantic, parse)
- [x] Pattern order maintained: specific patterns before general ones
- [x] Exit code policy preserved: business logic errors return 1, command/parse errors return 2

## Test Summary

| Test Suite | Tests | Passed | Failed |
|------------|-------|--------|--------|
| cli_event_tests (unit) | 16 | 16 | 0 |
| cli_e2e (integration) | 13 | 13 | 0 |
| **Total** | **29** | **29** | **0** |
