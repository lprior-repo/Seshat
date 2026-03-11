# Implementation: seshat-fwm - Inline Text Blur Dispatch

## Contract Summary
Wired inline text editor onBlur event to dispatch DomainOp::UpdateLabel.

## Changes Made

### 1. `diagram_tool/src/ui/dispatch.rs`
- Added `create_update_label_envelope()` function
- Added `dispatch_update_label()` function

### 2. `diagram_tool/src/ui/canvas/interaction_reducer.rs`
- Modified `commit_inline_edit()` to accept `db_tx` parameter and return `Result<(), CommitError>`
- Added `CommitError` enum with variants: `DispatchFailed`, `TargetNotFound`, `NoEditActive`
- Split function to stay under 25 lines (now 17 lines)
- Added helper functions: `commit_edit_to_target`, `commit_node_label_edit`, `commit_edge_label_edit`, `current_node_label`, `current_edge_label`, `build_updated_node_doc`, `build_updated_edge_doc`, `ensure_target_exists`
- Added test module `commit_inline_edit_tests` with 4 tests

### 3. `diagram_tool/src/ui/canvas.rs`
- Updated all 10 call sites of `commit_inline_edit()` to pass `db_tx.clone()` and handle `Result` with `.ok()`

### 4. `diagram_tool/src/ui/mod.rs`
- Added `pub mod dispatch;` to export the dispatch module

## Contract Verification

- P1 (Input valid UTF-8): String type guarantees
- P2 (Text differs): Comparison before dispatch
- P3 (Node in EditMode): Checked before edit
- Q1 (Dispatch on blur): onblur handler calls commit_inline_edit with db_tx
- Q2 (No dispatch if unchanged): Label comparison before dispatch

## Constraint Adherence

| Constraint | Implementation |
|------------|----------------|
| Zero panics | All return Result<T, CommitError> |
| Zero mut | Uses functional patterns |
| Result<T, E> | CommitError for errors |
| Function < 25 lines | commit_inline_edit is 17 lines |
| Tests added | 4 tests for CommitError type |
