bead_id: bd-23z
bead_title: cli: save LKG before patch operation
phase: p0
updated_at: 2026-03-02T00:30:00Z

# Contract: bd-23z - CLI Save LKG Before Patch Operation

## Summary

The CLI patch command must save a Last Known Good (LKG) file **before** attempting any patch operations, not just when a revision test fails. This ensures recovery is possible from any patch failure scenario.

## Problem Statement

Currently, the `patch` command in `diagram_tool/src/cli.rs` only saves an LKG file when a test operation fails (specifically when `test_passed` is false). This is insufficient because:

1. If a `replace`, `add`, or `remove` operation fails, no LKG is saved
2. If post-patch validation fails, no LKG is saved
3. The user cannot recover the pre-patch state in these failure scenarios

## Acceptance Criteria

### AC1: LKG Saved Before Patch Operations
- GIVEN a valid input document
- WHEN the patch command is invoked
- THEN an LKG file must be saved to `.lkg/<filename>.lkg` BEFORE any patch operations are applied

### AC2: LKG Contains Pre-Patch State
- GIVEN the LKG file saved in AC1
- WHEN the LKG file is loaded
- THEN it must contain the exact document state before any patch operations

### AC3: LKG Saved Regardless of Patch Outcome
- GIVEN a patch command that fails for any reason (stale revision, validation error, etc.)
- WHEN the command completes with failure
- THEN an LKG file must exist containing the pre-patch state

### AC4: Stage Event Emitted for LKG Save
- GIVEN the LKG save operation
- WHEN the LKG is saved successfully
- THEN a `lkg_saved` stage event must be emitted with the LKG path

### AC5: Backward Compatibility
- GIVEN existing tests for LKG behavior
- WHEN the fix is applied
- THEN all existing tests must continue to pass

## Invariants

1. LKG must be saved before any modifications to the document
2. LKG must use the same atomic write pattern as `save_workspace_atomic`
3. LKG directory must be `.lkg/` relative to the input file's parent
4. LKG filename format must be `<original_filename>.lkg`

## Preconditions

- Input document must be loadable and valid
- Patch file must be parseable as JSON

## Postconditions

- On success: output document written, LKG preserved
- On failure: LKG preserved, original input unchanged

## Test Mappings

| Test File | Test Name | Maps To |
|-----------|-----------|---------|
| `diagram_tool/tests/cli_e2e.rs` | `given_failed_patch_when_last_known_good_exists_then_original_is_preserved` | AC3 |
| `diagram_tool/tests/cli_e2e.rs` | `given_stale_revision_document_when_patch_runs_then_it_fails_with_stale_revision_error` | AC3 |
