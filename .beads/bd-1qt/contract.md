bead_id: bd-1qt
bead_title: tests: Implement MUL multi-select tests 4/4
phase: p0
updated_at: 2026-03-01T22:25:00Z

# Contract: MUL Multi-Select Rotation Tests

## Purpose

Implement 5 multi-select tests focusing on rotation operations in the diagram tool.

## Scope

Add tests to `diagram_tool/src/geometry/mod.rs` (or appropriate test module) covering multi-selection rotation scenarios:

### MUL-001: Rotate Around Center
- Test that multi-selected items rotate around their collective center
- When rotating a selection of multiple nodes, the rotation center is the centroid of the selection bounds
- All selected items should maintain their relative positions to each other
- Use `rotate_around_center` from geometry module

### MUL-002: Mixed Rotation Combine
- Test combining multiple rotation operations on a multi-selection
- Sequential rotations should compose correctly
- Rotating by angle A then angle B equals rotating by angle (A + B)
- Rotation composition is additive

### MUL-003: Rotate Bound Edges Survive
- Test that selection bounds edges remain valid after rotation
- After rotation, the selection bounds should encompass all rotated items
- No item should fall outside the computed selection bounds
- AABB must contain all corners of all selected items

### MUL-004: Rotate 360 No Drift
- Test that rotating by 360 degrees returns items to original position
- Full rotation should have minimal drift due to floating-point precision
- Drift should be bounded (< 1e-9 for f64 operations)
- Tests numerical stability of rotation implementation

### MUL-005: Rotate Undo/Redo
- Test that rotation operations can be undone and redone
- Using History module, verify rotation can be undone
- After undo, items return to pre-rotation positions
- After redo, items return to rotated positions

## Preconditions

1. Project must compile (`moon run :quick` passes)
2. `rotate_around_center` function exists in `diagram_tool/src/geometry/mod.rs`
3. `History` module exists in `diagram_tool/src/history.rs`
4. Existing test patterns can be used as reference
5. No unsafe code allowed

## Postconditions

1. All 5 MUL tests pass (`moon run :test` passes)
2. Tests added to geometry module test section
3. Each test has clear given/when/then structure
4. Property-based tests use proptest where appropriate

## Acceptance Criteria

- [ ] MUL-001: Rotate around center test passes
- [ ] MUL-002: Mixed rotation combine test passes
- [ ] MUL-003: Rotate bound edges survive test passes
- [ ] MUL-004: Rotate 360 no drift test passes
- [ ] MUL-005: Rotate undo/redo test passes
- [ ] All tests pass with `moon run :test`
- [ ] CI passes with `moon run :ci`

## Test Mapping

| Test ID | Test Function Name | Category |
|---------|-------------------|----------|
| MUL-001 | test_mul_rotate_around_center | multi-select |
| MUL-002 | test_mul_mixed_rotation_combine | multi-select |
| MUL-003 | test_mul_rotate_bound_edges_survive | multi-select |
| MUL-004 | test_mul_rotate_360_no_drift | multi-select |
| MUL-005 | test_mul_rotate_undo_redo | multi-select |

## Implementation Notes

- Tests should use the existing geometry primitives (`Point`, `AABB`, `Rectangle`)
- Use `rotate_around_center(point, center, angle)` for rotation calculations
- Use `History::push/undo/redo` for undo/redo testing
- Floating-point comparisons should use `TOLERANCE = 1e-10`
- Multi-selection is simulated by tracking multiple points representing node centers
