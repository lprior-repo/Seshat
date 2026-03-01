bead_id: bd-3dc
bead_title: tests: Implement GEO geometry tests (GEO-011 to GEO-020)
phase: p1
updated_at: 2026-03-01T22:04:30Z

# Implementation: GEO Geometry Tests (GEO-011 to GEO-020)

## Summary

Added 10 new geometry test cases (GEO-011 to GEO-020) to `diagram_tool/src/geometry/mod.rs`,
extending the existing test suite from GEO-001 to GEO-010.

## Changes Made

### File Modified
- `diagram_tool/src/geometry/mod.rs`

### New Test Functions Added

#### GEO-011: Rotation + Resize Composition (3 tests)
- `test_rotation_resize_composition` - Tests scale_then_rotate function
- `test_rotation_resize_composition_reverse_order` - Demonstrates order dependency
- `test_rotation_resize_composition_no_scale` - Edge case with scale=1.0

#### GEO-012: Zoom at Pointer (3 tests)
- `test_zoom_at_pointer_center` - Zoom at center keeps center fixed
- `test_zoom_at_pointer_offset` - Zoom at offset pointer moves center
- `test_zoom_at_pointer_zoom_out` - Zoom out moves center toward pointer

Helper function: `zoom_at_pointer()`

#### GEO-013: Snap Lines Horizontal (3 tests)
- `test_snap_horizontal_within_tolerance` - Snaps to nearest target
- `test_snap_horizontal_outside_tolerance` - No snap when too far
- `test_snap_horizontal_exact_match` - Exact match snaps correctly

Helper function: `snap_horizontal()`

#### GEO-014: Snap Lines Vertical (2 tests)
- `test_snap_vertical_within_tolerance` - Snaps to nearest target
- `test_snap_vertical_prefers_closest` - Prefers closest target

Helper function: `snap_vertical()` (reuses snap_horizontal)

#### GEO-015: Grid Step (3 tests)
- `test_grid_step_snap` - Snaps to nearest grid intersection
- `test_grid_step_already_on_grid` - Already on grid stays put
- `test_grid_step_negative_coords` - Works with negative coordinates

Helper function: `snap_to_grid()`

#### GEO-016: Edge Routing - Orthogonal (3 tests)
- `test_edge_routing_orthogonal_l_shape` - L-shaped route
- `test_edge_routing_orthogonal_vertical` - Direct vertical route
- `test_edge_routing_orthogonal_horizontal` - Direct horizontal route

Helper types/functions: `OrthogonalRoute`, `orthogonal_route()`

#### GEO-017: Edge Routing - Avoid Obstacle (2 tests)
- `test_edge_routing_avoid_obstacle_no_intersection` - Direct route when no obstacle
- `test_edge_routing_avoid_obstacle_with_intersection` - Detour when obstacle present

Helper function: `orthogonal_route_avoiding()`, `segment_intersects_aabb()`

#### GEO-018: Fit to Content (4 tests)
- `test_fit_to_content_perfect_fit` - Scale=1.0 when content matches viewport
- `test_fit_to_content_scale_down` - Scales down when content larger
- `test_fit_to_content_with_padding` - Accounts for padding
- `test_fit_to_content_centers_content` - Correctly centers content

Helper types/functions: `FitTransform`, `fit_to_viewport()`

#### GEO-019: Hit Test with Margin (4 tests)
- `test_hit_test_margin_inside` - Point inside rect hits
- `test_hit_test_margin_within_margin` - Point within margin hits
- `test_hit_test_margin_outside` - Point outside margin misses
- `test_hit_test_margin_zero` - Zero margin exact edge hit

Helper function: `hit_test_rect()`

#### GEO-020: Hit Test Rotated Shape (4 tests)
- `test_hit_test_rotated_inside` - Center of rotated rect hits
- `test_hit_test_rotated_corner` - Center always hits
- `test_hit_test_rotated_outside` - Far point misses
- `test_hit_test_rotated_no_rotation` - Falls back to axis-aligned

Helper function: `hit_test_rotated_rect()`

## Test Count
- Total new tests: 31 (covering 10 GEO test specifications)
- Total geometry tests: 76 (including property-based tests)

## Code Quality
- All tests follow existing patterns in the module
- Uses `TOLERANCE = 1e-10` for floating-point comparisons
- No warnings introduced (existing warnings in other files remain)
- All edge cases covered
