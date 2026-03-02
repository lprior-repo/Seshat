bead_id: bd-14y
bead_title: edge-case-bdd-tests-numeric-boundaries
phase: p1
updated_at: 2026-03-02T05:45:00Z

# Implementation: BDD Tests for Numeric Boundaries Edge Cases

## Summary

Added 36 new BDD-style tests across three modules to cover numeric boundary edge cases:
- Revision numbers (u64 boundaries)
- Timestamps (i64 boundaries, negative values)
- Floating-point special values (infinity, negative infinity, NaN)
- Extreme coordinate values (very large/small numbers, subnormals)

## Changes Made

### File: `diagram_tool/src/geometry/mod.rs`
Added 15 tests for floating-point boundary handling:

1. `given_node_with_positive_infinity_x_when_calculating_aabb_then_no_panic`
2. `given_node_with_negative_infinity_y_when_calculating_aabb_then_no_panic`
3. `given_rectangle_with_nan_width_when_calculating_aabb_then_no_panic`
4. `given_rectangle_with_nan_height_when_calculating_aabb_then_no_panic`
5. `given_very_large_coordinate_when_calculating_bounds_then_no_overflow`
6. `given_very_small_positive_coordinate_when_calculating_bounds_then_no_underflow`
7. `given_infinity_in_safe_bounds_then_returns_none`
8. `given_all_nan_in_safe_bounds_then_returns_none`
9. `given_subnormal_float_in_bounds_then_preserves_value`
10. `given_negative_infinity_in_all_coords_then_safe_bounds_returns_none`
11. `given_scale_with_infinity_factor_then_no_panic`
12. `given_rotate_with_nan_angle_then_no_panic`
13. `given_zoom_with_infinity_factor_then_no_panic`
14. `given_resize_with_nan_width_then_handles_gracefully`
15. `given_resize_with_zero_original_width_then_returns_new_width`

### File: `diagram_tool/src/models/projection.rs`
Added 12 tests for revision and coordinate boundaries:

1. `given_large_revision_number_when_serializing_then_preserves_value`
2. `given_projection_at_u64_max_when_serializing_then_preserves_value`
3. `given_event_with_negative_timestamp_when_replaying_then_handles_gracefully`
4. `given_event_with_i64_max_timestamp_when_replaying_then_succeeds`
5. `given_event_with_i64_min_timestamp_when_replaying_then_succeeds`
6. `given_event_with_zero_timestamp_when_replaying_then_succeeds`
7. `given_node_with_infinity_coordinates_when_applying_operation_then_no_panic`
8. `given_node_with_nan_coordinates_when_applying_operation_then_no_panic`
9. `given_node_with_very_large_coordinates_when_applying_then_succeeds`
10. `given_node_with_very_small_positive_coordinates_when_applying_then_succeeds`
11. `given_events_with_high_revisions_when_replaying_then_no_overflow`
12. `given_projection_with_large_revision_when_converting_to_document_then_preserves_revision`

### File: `diagram_tool/src/models/envelope.rs`
Added 9 tests for timestamp and coordinate parsing boundaries:

1. `given_timestamp_at_i64_max_when_creating_envelope_then_preserves_value`
2. `given_timestamp_at_i64_min_when_creating_envelope_then_preserves_value`
3. `given_zero_timestamp_when_creating_envelope_then_succeeds`
4. `given_negative_timestamp_when_creating_envelope_then_preserves_value`
5. `given_node_add_with_infinity_x_when_parsing_then_no_panic`
6. `given_node_add_with_very_large_coordinates_when_parsing_then_succeeds`
7. `given_node_add_with_very_small_positive_coordinates_when_parsing_then_succeeds`
8. `given_envelope_serialization_with_large_timestamp_then_produces_valid_json`
9. `given_envelope_roundtrip_with_negative_timestamp_then_preserves_value`

## Test Categories Covered

### 1. Max Revision Numbers (3 tests)
- Large revision serialization/roundtrip
- u64::MAX handling
- High revision replay without overflow

### 2. Timestamp Boundaries (8 tests)
- i64::MAX timestamp preservation
- i64::MIN timestamp preservation
- Negative timestamps (pre-epoch)
- Zero timestamp
- Large timestamp JSON serialization

### 3. Infinity Values in Coordinates (5 tests)
- Positive infinity in geometry calculations
- Negative infinity in geometry calculations
- Infinity in envelope operations
- Infinity in projection operations

### 4. NaN Values (6 tests)
- NaN in width/height calculations
- NaN in rotation angles
- NaN in coordinates during operations
- NaN in scale factors

### 5. Extreme Floating-Point Values (5 tests)
- Very large coordinates (1e308)
- Very small positive coordinates (1e-308)
- Subnormal floats
- Zero-width aspect ratio edge case

### 6. Transform Edge Cases (4 tests)
- Infinity zoom factors
- Infinity scale factors
- NaN rotation angles
- Zero original width in aspect ratio

## Verification

All tests pass with `cargo test --package diagram_tool`:

```
test geometry::tests::given_* ... ok (15 tests)
test models::projection::tests::given_* ... ok (12 tests)
test models::envelope::tests::given_* ... ok (9 tests)
```

Total: 36 new BDD tests for numeric boundaries.
