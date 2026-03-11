# Martin Fowler Test Plan

## Happy Path Tests
- `test_mul_011_scale_around_group_center`: Verifies that multiple nodes scale their positions and dimensions uniformly relative to a shared anchor point.
- `test_mul_012_mixed_node_types_scale`: Verifies that scaling a group containing shapes, lines, and text properly applies the scale factor to the relevant geometric properties of each type.
- `test_mul_015_scale_undo_redo`: Verifies that a group scale operation can be perfectly reversed via an undo action, restoring exact original bounds and positions.

## Error Path Tests
- `test_returns_error_when_selection_is_empty`
- `test_returns_error_when_node_not_found`
- `test_returns_error_when_selection_contains_locked_node`
- `test_returns_error_when_scale_exceeds_canvas_bounds`

## Edge Case Tests
- `test_mul_013_scale_clamps_to_minimum_dimension`: Scaling down towards zero clamps node dimensions to `MIN_DIMENSION` and prevents negative scale inversion from corrupting geometry.
- `test_mul_014_inverse_scale_no_drift`: Scaling up by a factor, then scaling down by the inverse factor repeatedly causes zero drift (loss of precision < epsilon).
- `test_handles_single_item_group_scale`: Group scale logic correctly reduces to single-item scale when selection count is exactly 1.

## Contract Verification Tests
- `test_precondition_scale_is_positive` (Compile-time validated via `PositiveScale`)
- `test_postcondition_relative_distances_scaled`
- `test_postcondition_dimensions_scaled`
- `test_postcondition_unselected_nodes_unmutated`
- `test_invariant_node_count_remains_unchanged`
- `test_invariant_node_types_remain_unchanged`

## Contract Violation Tests

- `test_p1_empty_selection_violation_returns_error`
  Given: `scale_group(&mut subgraph, &[], scale, anchor)`
  When: function is called with an empty selection slice
  Then: returns `Err(GroupTransformError::EmptySelection)` -- NOT a panic, NOT an unwrap failure

- `test_p3_node_not_found_violation_returns_error`
  Given: `scale_group(&mut subgraph, &[missing_id], scale, anchor)`
  When: function is called with a NodeId that does not exist in the subgraph
  Then: returns `Err(GroupTransformError::NodeNotFound(missing_id))`

- `test_p4_node_locked_violation_returns_error`
  Given: `scale_group(&mut subgraph, &[locked_id], scale, anchor)`
  When: function is called with a selection containing a locked node
  Then: returns `Err(GroupTransformError::NodeLocked(locked_id))`

- `test_p5_exceeds_max_bounds_violation_returns_error`
  Given: `scale_group(&mut subgraph, &[id], huge_scale, anchor)` where the resulting coordinate exceeds canvas limits
  When: function is called with a massive scale factor
  Then: returns `Err(GroupTransformError::OutOfBounds)`

## Given-When-Then Scenarios

### Scenario 1: MUL-011 Group Scale Around Common Center
**Given**: A subgraph with Node A at `(10, 10)` size `10x10` and Node B at `(30, 30)` size `10x10`. Selection is `[A, B]`. Anchor is `(25, 25)` (group center).
**When**: `scale_group` is called with `scale_factor = 2.0`.
**Then**:
- Node A moves to `(25 + (10 - 25)*2, 25 + (10 - 25)*2)` = `(-5, -5)`.
- Node B moves to `(25 + (30 - 25)*2, 25 + (30 - 25)*2)` = `(35, 35)`.
- Node A size becomes `20x20`.
- Node B size becomes `20x20`.
- Function returns `Ok(())`.

### Scenario 2: MUL-013 Scale Clamps to Minimum Dimension
**Given**: A subgraph with Node A of size `10x10`. Selection is `[A]`. Anchor is its center. `MIN_DIMENSION` is `2.0`.
**When**: `scale_group` is called with `scale_factor = 0.1` (which would mathematically yield `1.0x1.0`).
**Then**:
- Node A's position scales correctly towards the anchor.
- Node A's size is clamped to exactly `2.0x2.0`.
- Function returns `Ok(())`.

### Scenario 3: MUL-014 Inverse Scale No Drift (Repeated Scale)
**Given**: A subgraph with Node A at `(100, 100)` size `50x50`. Selection is `[A]`. Anchor is `(0, 0)`.
**When**: `scale_group` is called with `scale_factor = 1.001`, followed by `scale_factor = 1.0 / 1.001`, repeated 100 times.
**Then**:
- Node A returns to `(100, 100)` with size `50x50`.
- The delta between the original state and final state is `< 1e-6` in both position and dimensions.

### Scenario 4: Postcondition Unselected Nodes Ignored
**Given**: A subgraph with Node A (selected) and Node B (unselected).
**When**: `scale_group` is called on `[A]` with `scale_factor = 2.0`.
**Then**:
- Node A's position and size are updated.
- Node B's exact memory representation remains bit-for-bit identical to the initial state.
