bead_id: bd-23z
bead_title: cli: save LKG before patch operation
phase: p2
updated_at: 2026-03-02T00:30:00Z

# Verification: bd-23z - CLI Save LKG Before Patch Operation

## Contract Acceptance Criteria Verification

### AC1: LKG Saved Before Patch Operations
- **Status**: PASS
- **Evidence**: Code review shows LKG save logic is placed immediately after loading `current_doc` and before reading/parsing the patch file (lines 247-276 in cli.rs)

### AC2: LKG Contains Pre-Patch State
- **Status**: PASS
- **Evidence**: The `save_workspace_atomic(&current_doc, &lkg_path)` call saves the document state immediately after loading, before any modifications

### AC3: LKG Saved Regardless of Patch Outcome
- **Status**: PASS
- **Evidence**:
  - LKG is saved before any patch operations are attempted
  - Test `given_failed_patch_when_last_known_good_exists_then_original_is_preserved` passes
  - LKG will exist even if:
    - Revision test fails
    - Replace/add/remove operation fails
    - Post-patch validation fails

### AC4: Stage Event Emitted for LKG Save
- **Status**: PASS
- **Evidence**: Code emits `lkg_saved` stage event with path and code="success" on successful save, or `lkg_save_failed` with error message on failure

### AC5: Backward Compatibility
- **Status**: PASS
- **Evidence**: All 13 existing CLI E2E tests pass without modification

## Test Execution Results

### Compilation
```
$ cargo check --package diagram_tool
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.25s
```

### CLI E2E Tests
```
$ cargo test --package diagram_tool --test cli_e2e

running 13 tests
test given_valid_patch_when_patch_command_runs_then_it_writes_updated_document ... ok
test given_self_loop_edge_when_validate_runs_then_it_fails_with_dag_error ... ok
test given_stale_revision_document_when_patch_runs_then_it_fails_with_stale_revision_error ... ok
test given_patch_without_first_revision_test_when_patch_runs_then_fail_closed_error_events_are_emitted ... ok
test given_invalid_v2_document_when_validate_runs_then_schema_violation_is_emitted ... ok
test given_dag_cycle_document_when_validate_runs_then_it_fails_with_dag_error ... ok
test given_validate_command_when_run_then_it_outputs_jsonl_start_and_finish_events ... ok
test given_dangling_edge_document_when_validate_runs_then_it_fails_with_dangling_error ... ok
test given_valid_document_when_layout_command_runs_then_output_contains_nodes_and_edges ... ok
test given_failed_patch_when_last_known_good_exists_then_original_is_preserved ... ok
test given_legacy_edge_alias_when_validate_runs_then_parse_error_is_emitted ... ok
test given_valid_document_when_validate_command_runs_then_it_succeeds ... ok
test given_valid_document_when_render_svg_command_runs_then_svg_file_is_generated ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## Invariants Verification

1. **LKG must be saved before any modifications to the document** - VERIFIED
   - Code placement ensures LKG is saved before patch operations loop

2. **LKG must use the same atomic write pattern as `save_workspace_atomic`** - VERIFIED
   - Uses the same `save_workspace_atomic` function

3. **LKG directory must be `.lkg/` relative to the input file's parent** - VERIFIED
   - `let lkg_dir = input_path.parent().unwrap_or(Path::new(".")).join(".lkg");`

4. **LKG filename format must be `<original_filename>.lkg`** - VERIFIED
   - `let lkg_filename = format!("{}.lkg", input_path.file_name()...);`

## Notes

Pre-existing test compilation issues in `diagram_tool/src/geometry/mod.rs` (missing `prop_assert` macro import) prevent full `cargo test` from running. This is unrelated to the bd-23z changes and was present before this implementation.
