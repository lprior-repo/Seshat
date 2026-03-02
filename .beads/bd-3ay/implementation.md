bead_id: bd-3ay
bead_title: edge-case-bdd-tests-geometry-boundaries
phase: p1
updated_at: 2026-03-02T04:50:00Z

# Implementation: Edge-Case BDD Tests for Geometry Boundary Conditions

## Summary

Added comprehensive BDD (Behavior-Driven Development) tests for geometry boundary
conditions in `/home/lewis/src/seshat/diagram_tool/src/geometry/mod.rs`.

## Changes

### GEO-EDGE-001: Zero Dimensions (8 tests)
- `test_edge_zero_width_rectangle` - Zero width produces zero-width AABB
- `test_edge_zero_height_rectangle` - Zero height produces zero-height AABB
- `test_edge_zero_both_dimensions_rectangle` - Degenerate point at position
- `test_edge_zero_dimensions_at_origin` - Origin degenerate case
- `test_edge_zero_width_with_rotation` - Rotation of line segment
- `test_edge_zero_dimensions_image` - Image with zero dimensions
- `test_edge_zero_area_aabb_operations` - Operations on zero-area AABB
- `test_edge_zero_dimensions_expand` - Expansion creates area

### GEO-EDGE-002: Maximum Rotation Values (11 tests)
- `test_edge_rotation_full_circle` - 2*pi returns to original
- `test_edge_rotation_beyond_2pi` - Rotation mod 2*pi equivalence
- `test_edge_rotation_negative_angle` - Negative rotation equivalence
- `test_edge_rotation_pi_half_boundary` - 90 degree boundary
- `test_edge_rotation_pi_boundary` - 180 degree produces same AABB
- `test_edge_rotation_3pi_half_boundary` - 270 degree boundary
- `test_edge_rotation_very_large_angle` - 100 full circles + offset
- `test_edge_rotation_consistency_across_multiples` - Multiple of 2*pi
- `test_edge_rotate_point_full_circle` - Point rotation by 2*pi
- `test_edge_rotate_point_negative_full_circle` - Point rotation by -2*pi

### GEO-EDGE-003: Negative Dimensions (6 tests)
- `test_edge_negative_width_aabb_calculation` - Documents inverted bounds
- `test_edge_negative_height_aabb_calculation` - Documents inverted bounds
- `test_edge_both_dimensions_negative` - Both axes inverted
- `test_edge_negative_dimensions_with_rotation` - Rotation still works
- `test_edge_safe_bounds_with_swapped_coords` - Normalization by safe_bounds
- `test_edge_scale_to_negative_factor` - Flip across anchor
- `test_edge_scale_to_negative_preserves_anchor` - Anchor invariance

### GEO-EDGE-004: Infinite Coordinates (12 tests)
- `test_edge_safe_bounds_rejects_positive_infinity` - Rejects +inf
- `test_edge_safe_bounds_rejects_negative_infinity` - Rejects -inf
- `test_edge_safe_bounds_rejects_nan` - Rejects NaN
- `test_edge_safe_bounds_rejects_nan_in_max` - Rejects NaN in any coord
- `test_edge_safe_bounds_accepts_large_finite` - Large finite OK
- `test_edge_point_at_infinity_rotation` - Infinity rotation propagation
- `test_edge_aabb_infinity_min` - Infinity in AABB
- `test_edge_aabb_expand_infinity` - Expansion by infinity
- `test_edge_scale_with_infinity_factor` - Infinite scale
- `test_edge_scale_infinity_point` - Scaling infinity point
- `test_edge_point_origin_is_finite` - Origin is always finite

### GEO-EDGE-005: Stroke Width Boundaries (9 tests)
- `test_edge_stroke_width_zero` - Zero stroke = no expansion
- `test_edge_stroke_width_negative` - Negative contracts bounds
- `test_edge_stroke_width_very_large` - Stroke larger than shape
- `test_edge_stroke_width_with_zero_dimension_shape` - Creates area from point
- `test_edge_stroke_width_with_rotated_shape` - Works with rotation
- `test_edge_stroke_width_infinity` - Infinite stroke
- `test_edge_stroke_width_nan` - NaN propagation
- `test_edge_stroke_width_tiny` - Very small stroke

### Property-Based Tests (7 tests)
- `prop_edge_zero_width_any_height` - Proptest for zero width
- `prop_edge_zero_height_any_width` - Proptest for zero height
- `prop_edge_rotation_equivalence` - Proptest for rotation mod 2*pi
- `prop_edge_negative_dimensions_aabb_valid` - Proptest for negative dims
- `prop_edge_safe_bounds_finite_always_succeeds` - Proptest for safe_bounds
- `prop_edge_stroke_width_finite` - Proptest for finite stroke
- `prop_edge_rotation_corners_within_aabb` - Proptest for AABB containment

## Test Count

- Total new tests: 49 unit tests + 7 property-based tests = 56 tests
- All tests follow Given-When-Then BDD pattern
- All tests use existing TOLERANCE constant (1e-10)
- No use of unwrap(), expect(), or panic!() in test assertions

## Files Modified

- `/home/lewis/src/seshat/diagram_tool/src/geometry/mod.rs` - Added tests to existing test module
