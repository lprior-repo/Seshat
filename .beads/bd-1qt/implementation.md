bead_id: bd-1qt
bead_title: tests: Implement MUL multi-select tests 4/4
phase: p1
updated_at: 2026-03-01T22:45:00Z

# Implementation: MUL Multi-Select Rotation Tests

## Summary

Implemented 8 unit tests and 3 property-based tests for multi-select rotation operations in the diagram tool geometry module.

## Files Modified

- `diagram_tool/src/geometry/mod.rs` - Added MUL tests to the test module

## Tests Implemented

### MUL-001: Rotate Around Center
- `test_mul_rotate_around_center`
- Tests that multi-selected items rotate around their collective center (centroid)
- Verifies selection center is invariant under rotation

### MUL-002: Mixed Rotation Combine
- `test_mul_mixed_rotation_combine`
- `test_mul_mixed_rotation_combine_multiple`
- Tests that sequential rotations compose correctly
- Verifies rotation composition is additive (A + B = combined)

### MUL-003: Rotate Bound Edges Survive
- `test_mul_rotate_bound_edges_survive`
- Tests that selection bounds encompass all rotated items after rotation
- Verifies AABB contains all corners of all selected items

### MUL-004: Rotate 360 No Drift
- `test_mul_rotate_360_no_drift`
- `test_mul_rotate_360_no_drift_incremental`
- Tests numerical stability of rotation implementation
- Verifies drift is bounded (< 1e-9 for direct, < 1e-6 for incremental)

### MUL-005: Rotate Undo/Redo
- `test_mul_rotate_undo_redo`
- `test_mul_rotate_undo_redo_with_history`
- Tests that rotation operations can be undone and redone
- Verifies positions are correctly restored

## Property-Based Tests

- `prop_mul_rotation_preserves_distances` - Rotation preserves distances between points
- `prop_mul_full_rotation_returns_to_origin` - 360-degree rotation returns to original position
- `prop_mul_selection_center_unchanged_by_rotation` - Selection center is invariant under rotation

## Helper Functions

Added `selection_center(points: &[Point]) -> Point` - Calculates the centroid of multiple points representing selected items.

## Test Results

```
running 11 tests
test geometry::tests::test_mul_mixed_rotation_combine ... ok
test geometry::tests::test_mul_mixed_rotation_combine_multiple ... ok
test geometry::tests::test_mul_rotate_360_no_drift ... ok
test geometry::tests::test_mul_rotate_360_no_drift_incremental ... ok
test geometry::tests::test_mul_rotate_around_center ... ok
test geometry::tests::test_mul_rotate_bound_edges_survive ... ok
test geometry::tests::test_mul_rotate_undo_redo ... ok
test geometry::tests::test_mul_rotate_undo_redo_with_history ... ok
test geometry::tests::prop_mul_full_rotation_returns_to_origin ... ok
test geometry::tests::prop_mul_rotation_preserves_distances ... ok
test geometry::tests::prop_mul_selection_center_unchanged_by_rotation ... ok

test result: ok. 11 passed; 0 failed; 0 ignored
```

## Notes

- All tests use the existing `rotate_around_center` function from the geometry module
- Tests follow the given/when/then structure for clarity
- Property-based tests use proptest for comprehensive coverage
- Floating-point comparisons use `TOLERANCE = 1e-10`
