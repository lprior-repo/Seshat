bead_id: bd-l79
bead_title: qa-matrix: build integration and failure regression suite
phase: p3
updated_at: 2026-02-28T21:56:00Z

# QA Verification Report

## Test Execution Results

Command: `cargo test --manifest-path diagram_tool/Cargo.toml --test cli_e2e`

### Existing Tests (Passing)
| Test | Status |
|------|--------|
| given_valid_document_when_validate_command_runs_then_it_succeeds | PASS |
| given_valid_patch_when_patch_command_runs_then_it_writes_updated_document | PASS |
| given_valid_document_when_layout_command_runs_then_output_contains_nodes_and_edges | PASS |
| given_valid_document_when_render_svg_command_runs_then_svg_file_is_generated | PASS |
| given_validate_command_when_run_then_it_outputs_jsonl_start_and_finish_events | PASS |
| given_invalid_v2_document_when_validate_runs_then_schema_violation_is_emitted | PASS |
| given_legacy_edge_alias_when_validate_runs_then_parse_error_is_emitted | PASS |
| given_patch_without_first_revision_test_when_patch_runs_then_fail_closed_error_events_are_emitted | PASS |

### New Tests (Intentionally Failing - TDD)
| Test | Expected Failure Reason |
|------|------------------------|
| given_stale_revision_document_when_patch_runs_then_it_fails_with_stale_revision_error | Feature not implemented: stale_revision error code |
| given_dag_cycle_document_when_validate_runs_then_it_fails_with_dag_error | Feature not implemented: dag_violation error code |
| given_dangling_edge_document_when_validate_runs_then_it_fails_with_dangling_error | Feature not implemented: dangling_reference error code |
| given_self_loop_edge_when_validate_runs_then_it_fails_with_dag_error | Feature not implemented: dag_violation error code |
| given_failed_patch_when_last_known_good_exists_then_original_is_preserved | Feature not implemented: LKG preservation |

## QA Assessment

- **Test Execution**: All 13 tests executed (8 pass, 5 fail intentionally)
- **Code Compilation**: Tests compile successfully
- **Test Design**: Tests follow TDD pattern - failing tests represent acceptance criteria

## Notes
The 5 failing tests represent features that need to be implemented:
1. Error code mapping for stale_revision, dag_violation, dangling_reference
2. Last-known-good state preservation for failed patches

This is the expected TDD outcome - tests written first, implementation follows.
