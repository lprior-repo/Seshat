bead_id: bd-338
bead_title: tests: Implement GEO geometry tests - transforms
phase: p1
updated_at: 2026-03-02T00:42:00Z

# Implementation: GEO Geometry Transform Tests

## Summary
Implemented 5 comprehensive transform test suites in `/home/lewis/src/seshat/diagram_tool/src/geometry/mod.rs` covering scale around anchor points, rotation around selection center, rotation around custom pivot, minimum size clamping, and negative scaling behavior.

## Test Implementations

### 1. GEO-TRN-001: Scale Around Anchor Point (NW/NE/SE/SW)

**Tests added:**
- `test_scale_around_anchor_nw` - Scale 2x around NorthWest corner, anchor stays fixed
- `test_scale_around_anchor_ne` - Scale 2x around NorthEast corner, anchor stays fixed
- `test_scale_around_anchor_se` - Scale 2x around SouthEast corner, anchor stays fixed
- `test_scale_around_anchor_sw` - Scale 2x around SouthWest corner, anchor stays fixed
- `test_scale_around_anchor_shrink_nw` - Scale 0.5x (shrink) around NW corner

**Helper functions:**
- `Corner` enum - Defines the four corner anchor points
- `get_corner_point()` - Returns the Point for a given rectangle corner
- `scale_rect_around_corner()` - Scales a rectangle keeping specified corner fixed

### 2. GEO-TRN-002: Rotate Around Selection Center

**Tests added:**
- `test_rotate_around_selection_center_single_item` - Single item rotation
- `test_rotate_around_selection_center_multiple_items` - Multiple items rotate as group
- `test_rotate_around_selection_center_45_degrees` - 45-degree rotation preserves distances

**Key behaviors verified:**
- Selection center (centroid) remains invariant under rotation
- Distances from items to center are preserved
- Items rotate as a coherent group

### 3. GEO-TRN-003: Rotate Around Custom Pivot

**Tests added:**
- `test_rotate_around_custom_pivot_origin` - Rotation around origin
- `test_rotate_around_custom_pivot_offset` - Rotation around offset pivot point
- `test_rotate_around_custom_pivot_270_degrees` - 270-degree rotation (equivalent to -90)
- `test_rotate_around_custom_pivot_preserves_distance` - Distance preservation at various angles

**Key behaviors verified:**
- Pivot point remains fixed during rotation
- Distance from point to pivot is invariant
- Various rotation angles work correctly

### 4. GEO-TRN-004: Minimum Size Clamp

**Tests added:**
- `test_min_size_clamp_below_minimum` - Both dimensions below min are clamped
- `test_min_size_clamp_one_below_minimum` - Only small dimension is clamped
- `test_min_size_clamp_at_minimum` - Dimensions at minimum stay unchanged
- `test_min_size_clamp_above_minimum` - Large dimensions unchanged
- `test_min_size_clamp_with_scaling` - Combined scaling and clamping

**Helper function:**
- `clamp_to_min_size(width, height, min_size)` - Clamps both dimensions to minimum

**Constants:**
- `MIN_SIZE = 1.0` - Minimum allowed geometry dimension

### 5. GEO-TRN-005: Negative Scaling Flip vs Clamp

**Tests added:**
- `test_negative_scaling_flip_x` - Horizontal flip via negative scale
- `test_negative_scaling_flip_y` - Vertical flip via negative scale
- `test_negative_scaling_flip_both` - Both axes flip
- `test_negative_scaling_clamp_x` - Negative scale clamped to minimum
- `test_negative_scaling_clamp_y` - Negative vertical scale clamped
- `test_negative_scaling_clamp_both` - Both negative scales clamped
- `test_negative_scaling_zero_transition` - Behavior near zero crossing

**Helper functions:**
- `scale_with_flip(width, height, scale_x, scale_y)` - Takes absolute value (mirror behavior)
- `scale_with_clamp(width, height, scale_x, scale_y, min_size)` - Clamps negative to minimum

## Code Quality
- All tests follow Given/When/Then structure
- TOLERANCE constant used for floating-point comparisons
- No unwrap/expect in test code (uses assert with tolerance)
- Tests are deterministic and independent

## File Modified
- `/home/lewis/src/seshat/diagram_tool/src/geometry/mod.rs` (lines ~3334-3620)

## Total Tests Added
- 26 new test functions covering the 5 transform test categories
