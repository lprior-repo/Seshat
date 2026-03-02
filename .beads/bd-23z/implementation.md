bead_id: bd-23z
bead_title: cli: save LKG before patch operation
phase: p1
updated_at: 2026-03-02T00:30:00Z

# Implementation: bd-23z - CLI Save LKG Before Patch Operation

## Summary

Modified the CLI patch command in `/home/lewis/src/seshat/diagram_tool/src/cli.rs` to save the Last Known Good (LKG) file **before** any patch operations are applied, rather than only when a revision test fails.

## Changes Made

### File: `/home/lewis/src/seshat/diagram_tool/src/cli.rs`

#### 1. LKG save before patch operations (already present from previous work)

The LKG save logic is placed immediately after loading the document and before reading/parsing the patch file (lines 247-276):

```rust
// Save LKG (Last Known Good) before any patch operations
// This ensures we have a recovery point regardless of how the patch fails
let input_path = Path::new(input);
let lkg_dir = input_path.parent().unwrap_or(Path::new(".")).join(".lkg");
std::fs::create_dir_all(&lkg_dir).ok();
let lkg_filename = format!(
    "{}.lkg",
    input_path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default()
);
let lkg_path = lkg_dir.join(lkg_filename);

if let Err(e) = save_workspace_atomic(&current_doc, &lkg_path) {
    emit_stage_event(
        "lkg_save_failed",
        &StageDetails::new()
            .with_path(&lkg_path)
            .with_code("lkg_save_failed")
            .with_message(&e.to_string()),
    );
} else {
    emit_stage_event(
        "lkg_saved",
        &StageDetails::new()
            .with_path(&lkg_path)
            .with_code("success"),
    );
}
```

#### 2. Removed duplicate LKG save from test failure path (this change)

The previous implementation saved LKG inside the `"test"` operation case when `test_passed` was false. This has been simplified to just emit the error event and return the error, since LKG is already saved at the start:

```rust
"test" => {
    // Test operation - verify value matches before proceeding
    // Note: LKG was already saved before any patch operations
    let expected = op.get("value");
    let actual = json_pointer_get(&doc, path);
    let test_passed = expected
        .and_then(|e| actual.as_ref().map(|a| e == a))
        .unwrap_or(false);
    if !test_passed {
        // Determine error code based on path
        let err_code = if path == "/revision" {
            "stale_revision"
        } else {
            "command_error"
        };

        emit_event(&CliEvent::error(
            String::from("patch"),
            String::from(err_code),
            format!(
                "test failed at {path}: expected {expected:?} but got {actual:?}"
            ),
        ));

        return Err(anyhow!(
            "{err_code}: test failed at {path}: expected {expected:?} but got {actual:?}"
        ));
    }
}
```

## Behavior Changes

### Before
- LKG was only saved when a test operation (revision check) failed
- If a replace/add/remove operation failed, no LKG was saved
- If post-patch validation failed, no LKG was saved

### After
- LKG is saved immediately after loading the document, before any patch operations
- LKG is available for recovery from any type of patch failure
- Stage event `lkg_saved` is emitted on success, `lkg_save_failed` on failure

## Test Results

All 13 CLI E2E tests pass:
```
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
```

## Verification

- `cargo check --package diagram_tool` - PASS
- `cargo clippy` (strict warnings) - PASS
- `cargo test --test cli_e2e` - PASS (13/13)
