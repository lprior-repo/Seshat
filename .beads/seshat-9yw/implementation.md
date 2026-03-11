# Implementation: seshat-9yw - NodeResize Dispatch with Original Bounds

## Contract Summary
Fixed DomainOp::NodeResize to include original bounds (original_x, original_y, original_width, original_height) for proper undo/history tracking.

## Changes Made

### 1. `diagram_tool/src/models/envelope.rs`
- Updated `DomainOp::NodeResize` variant (lines 121-131) to include:
  - `original_x: f64`
  - `original_y: f64`
  - `original_width: f64`
  - `original_height: f64`
  - `x: f64` (new position)
  - `y: f64` (new position)
  - `width: f64` (new dimensions)
  - `height: f64` (new dimensions)
- Updated `parse_node_resize()` (lines 295-318) to extract all 8 fields from JSON

### 2. `diagram_tool/src/models/projection/replay.rs`
- Updated pattern match at line ~178 to extract `x, y, width, height` from NodeResize
- Changed: `DomainOp::NodeResize { id, width, height }` → `DomainOp::NodeResize { id, x, y, width, height, .. }`

### 3. `diagram_tool/src/models/projection/ops/node_ops.rs`
- Updated `apply_node_resize()` (lines 336-367) to accept x, y, width, height parameters
- Now updates all 4 bounds (x, y, width, height) instead of just width/height

### 4. `diagram_tool/src/ui/dispatch.rs`
- Already had correct 10-parameter signature: id, original_x, original_y, original_width, original_height, x, y, width, height
- No changes needed - already implemented correctly
- Added tests: test_node_resize_envelope_has_valid_metadata, test_resize_bounds_no_resize_detection, test_finalize_motion_release_resize_wiring

### 5. `diagram_tool/src/ui/canvas/interaction_reducer.rs` (NEW)
- Modified `finalize_motion_release` to accept `db_tx` parameter
- Added logic to call `dispatch_node_resize` when resize completes with `did_resize=true`
- Iterates over resized nodes from `originals` HashMap and dispatches resize events

### 6. `diagram_tool/src/ui/canvas.rs` (NEW)
- Updated all 4 call sites of `finalize_motion_release` to pass `db_tx.clone()`

### 7. `diagram_tool/src/ui/mod.rs` (NEW)
- Added `pub mod dispatch;` to export the dispatch module

## Files Modified
1. `diagram_tool/src/models/envelope.rs` - DomainOp enum + parse function
2. `diagram_tool/src/models/projection/replay.rs` - Pattern match
3. `diagram_tool/src/models/projection/ops/node_ops.rs` - apply_node_resize function
4. `diagram_tool/src/ui/canvas/interaction_reducer.rs` - finalize_motion_release wiring
5. `diagram_tool/src/ui/canvas.rs` - Call site updates
6. `diagram_tool/src/ui/mod.rs` - Export dispatch module
7. `diagram_tool/src/ui/dispatch.rs` - Added NodeResize dispatch tests
8. `diagram_tool/src/ui/properties.rs` - Fixed const fn issue with unwrap_or
9. `diagram_tool/src/ui/toast.rs` - Fixed mutable borrow issues

## Constraint Adherence

| Constraint | Implementation |
|------------|----------------|
| Zero panics | No unwrap/expect/panic in core logic |
| Zero mut | Uses functional update patterns (rpds) |
| Result<T, E> | DispatchError, ContractError for all errors |
| Data→Calc→Actions | Pure envelope creation, async dispatch |
| Make illegal states unrepresentable | Type-safe f64 fields |

## Contract Verification

- NodeResize now includes original bounds for undo/redo support
- dispatch.rs create_node_resize_envelope accepts all 10 params
- finalize_motion_release now dispatches resize events when resizing completes
- All tests pass (envelope module)
