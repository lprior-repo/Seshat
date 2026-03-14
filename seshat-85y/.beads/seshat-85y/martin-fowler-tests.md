# Martin Fowler Test Plan

## Metadata
- bead_id: seshat-85y
- bead_title: CAM-001 to CAM-004: Zoom limits
- phase: test_plan
- updated_at: 2026-03-14T12:30:00Z

## Overview
This test plan covers zoom limit enforcement (0.1x to 4.0x) for the ViewportState. Tests verify clamping behavior, boundary handling, invalid input recovery, and idempotency at limits.

Tests are written for `ViewportState` methods which can run with `cargo test`:
- `viewport.set_zoom(zoom: f64) -> bool`
- `viewport.zoom_in() -> bool`
- `viewport.zoom_out() -> bool`
- `viewport.zoom_around_point(zoom: f64, screen_x: f64, screen_y: f64) -> bool`

---

## Happy Path Tests

### given_viewport_at_default_zoom_when_set_zoom_to_middle_value_then_zoom_changes
- Given: ViewportState at default zoom (1.0)
- When: set_zoom(2.0) is called
- Then: returns true

### given_viewport_at_default_zoom_when_set_zoom_to_middle_value_then_zoom_is_2_0
- Given: ViewportState at default zoom (1.0)
- When: set_zoom(2.0) is called
- Then: viewport.zoom() equals 2.0

### given_viewport_when_zoom_in_from_middle_then_zoom_increases
- Given: ViewportState at zoom 1.0
- When: zoom_in() is called
- Then: viewport.zoom() equals 1.25

### given_viewport_when_zoom_out_from_middle_then_zoom_decreases
- Given: ViewportState at zoom 1.0
- When: zoom_out() is called
- Then: viewport.zoom() equals 0.8

### given_viewport_at_zoom_2_0_when_set_zoom_to_1_0_then_returns_true
- Given: ViewportState at zoom 2.0
- When: set_zoom(1.0) is called
- Then: returns true

### given_viewport_at_default_zoom_when_set_zoom_to_1_0_then_returns_false
- Given: ViewportState at zoom 1.0
- When: set_zoom(1.0) is called
- Then: returns false

---

## Happy Path Tests: zoom_around_point

### given_viewport_at_default_zoom_when_zoom_around_point_to_valid_zoom_then_returns_true
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(2.0, 100.0, 100.0) is called
- Then: returns true

### given_viewport_at_default_zoom_when_zoom_around_point_to_valid_zoom_then_zoom_changes
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(2.0, 100.0, 100.0) is called
- Then: viewport.zoom() equals 2.0

### given_viewport_at_default_zoom_when_zoom_around_point_at_different_point_then_returns_true
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(1.5, 50.0, 200.0) is called
- Then: returns true

---

## Error Path Tests

### given_viewport_when_set_zoom_to_nan_then_returns_false
- Given: ViewportState at zoom 1.0
- When: set_zoom(f64::NAN) is called
- Then: returns false

### given_viewport_when_set_zoom_to_nan_then_zoom_unchanged
- Given: ViewportState at zoom 1.0
- When: set_zoom(f64::NAN) is called
- Then: viewport.zoom() still equals 1.0

### given_viewport_when_set_zoom_to_infinity_then_returns_false
- Given: ViewportState at zoom 1.0
- When: set_zoom(f64::INFINITY) is called
- Then: returns false

### given_viewport_when_set_zoom_to_infinity_then_zoom_unchanged
- Given: ViewportState at zoom 1.0
- When: set_zoom(f64::INFINITY) is called
- Then: viewport.zoom() still equals 1.0

### given_viewport_when_set_zoom_to_negative_then_returns_false
- Given: ViewportState at zoom 1.0
- When: set_zoom(-5.0) is called
- Then: returns false

### given_viewport_when_set_zoom_to_zero_then_returns_false
- Given: ViewportState at zoom 1.0
- When: set_zoom(0.0) is called
- Then: returns false

### given_viewport_when_zoom_around_point_to_nan_then_returns_false
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(f64::NAN, 100.0, 100.0) is called
- Then: returns false

### given_viewport_when_zoom_around_point_to_nan_then_zoom_unchanged
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(f64::NAN, 100.0, 100.0) is called
- Then: viewport.zoom() still equals 1.0

### given_viewport_when_zoom_around_point_to_infinity_then_returns_false
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(f64::INFINITY, 100.0, 100.0) is called
- Then: returns false

### given_viewport_when_zoom_around_point_to_infinity_then_zoom_unchanged
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(f64::INFINITY, 100.0, 100.0) is called
- Then: viewport.zoom() still equals 1.0

### given_viewport_when_zoom_around_point_to_negative_infinity_then_returns_false
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(f64::NEG_INFINITY, 100.0, 100.0) is called
- Then: returns false

### given_viewport_when_zoom_around_point_to_negative_value_then_returns_false
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(-5.0, 100.0, 100.0) is called
- Then: returns false

### given_viewport_when_zoom_around_point_to_negative_value_then_zoom_unchanged
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(-5.0, 100.0, 100.0) is called
- Then: viewport.zoom() still equals 1.0

### given_viewport_when_zoom_around_point_to_zero_then_returns_false
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(0.0, 100.0, 100.0) is called
- Then: returns false

### given_viewport_when_zoom_around_point_to_zero_then_zoom_unchanged
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(0.0, 100.0, 100.0) is called
- Then: viewport.zoom() still equals 1.0

### given_viewport_when_zoom_around_point_with_negative_screen_x_then_returns_true
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(2.0, -50.0, 100.0) is called
- Then: returns true

### given_viewport_when_zoom_around_point_with_negative_screen_y_then_returns_true
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(2.0, 100.0, -50.0) is called
- Then: returns true

### given_viewport_when_zoom_around_point_with_negative_screen_coords_then_zoom_changes
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(2.0, -50.0, -50.0) is called
- Then: viewport.zoom() equals 2.0

---

## Edge Case Tests

### given_viewport_when_set_zoom_above_max_then_clamps_to_max
- Given: ViewportState at zoom 1.0
- When: set_zoom(100.0) is called
- Then: viewport.zoom() equals 4.0

### given_viewport_when_set_zoom_below_min_then_clamps_to_min
- Given: ViewportState at zoom 1.0
- When: set_zoom(0.01) is called
- Then: viewport.zoom() equals 0.1

### given_viewport_at_max_zoom_when_zoom_in_then_returns_false
- Given: ViewportState at zoom 4.0 (MAX_ZOOM)
- When: zoom_in() is called
- Then: returns false

### given_viewport_at_max_zoom_when_zoom_in_then_zoom_unchanged
- Given: ViewportState at zoom 4.0 (MAX_ZOOM)
- When: zoom_in() is called
- Then: viewport.zoom() still equals 4.0

### given_viewport_at_min_zoom_when_zoom_out_then_returns_false
- Given: ViewportState at zoom 0.1 (MIN_ZOOM)
- When: zoom_out() is called
- Then: returns false

### given_viewport_at_min_zoom_when_zoom_out_then_zoom_unchanged
- Given: ViewportState at zoom 0.1 (MIN_ZOOM)
- When: zoom_out() is called
- Then: viewport.zoom() still equals 0.1

### given_viewport_at_max_zoom_when_set_zoom_to_max_then_returns_false
- Given: ViewportState at zoom 4.0 (MAX_ZOOM)
- When: set_zoom(4.0) is called
- Then: returns false

### given_viewport_at_min_zoom_when_set_zoom_to_min_then_returns_false
- Given: ViewportState at zoom 0.1 (MIN_ZOOM)
- When: set_zoom(0.1) is called
- Then: returns false

### given_viewport_when_zoom_in_repeatedly_then_converges_to_max
- Given: ViewportState at zoom 1.0
- When: zoom_in() is called 10 times
- Then: viewport.zoom() equals 4.0

### given_viewport_when_zoom_out_repeatedly_then_converges_to_min
- Given: ViewportState at zoom 1.0
- When: zoom_out() is called 10 times
- Then: viewport.zoom() equals 0.1

### given_viewport_with_nan_zoom_when_zoom_in_then_zoom_recovers
- Given: ViewportState with zoom set to f64::NAN
- When: zoom_in() is called
- Then: viewport.zoom() is finite

### given_viewport_with_nan_zoom_when_zoom_in_then_zoom_within_bounds
- Given: ViewportState with zoom set to f64::NAN
- When: zoom_in() is called
- Then: viewport.zoom() >= 0.1 and viewport.zoom() <= 4.0

### given_viewport_with_inf_zoom_when_zoom_out_then_zoom_recovers
- Given: ViewportState with zoom set to f64::INFINITY
- When: zoom_out() is called
- Then: viewport.zoom() is finite

### given_viewport_with_zero_zoom_when_zoom_in_then_zoom_recovers
- Given: ViewportState with zoom set to 0.0
- When: zoom_in() is called
- Then: viewport.zoom() >= 0.1

### given_viewport_when_zoom_around_point_above_max_then_clamps_to_max
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(100.0, 100.0, 100.0) is called
- Then: viewport.zoom() equals 4.0

### given_viewport_when_zoom_around_point_below_min_then_clamps_to_min
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(0.01, 100.0, 100.0) is called
- Then: viewport.zoom() equals 0.1

### given_viewport_when_zoom_around_point_to_max_boundary_then_returns_true
- Given: ViewportState at zoom 2.0
- When: zoom_around_point(4.0, 100.0, 100.0) is called
- Then: returns true

### given_viewport_when_zoom_around_point_to_min_boundary_then_returns_true
- Given: ViewportState at zoom 2.0
- When: zoom_around_point(0.1, 100.0, 100.0) is called
- Then: returns true

---

## Contract Verification Tests

### given_any_viewport_when_set_zoom_then_result_in_bounds
- Given: ViewportState at any zoom
- When: set_zoom is called with any value
- Then: viewport.zoom() always in [0.1, 4.0]

### given_any_viewport_when_zoom_in_then_result_in_bounds
- Given: ViewportState at any zoom
- When: zoom_in() is called
- Then: viewport.zoom() always in [0.1, 4.0]

### given_any_viewport_when_zoom_out_then_result_in_bounds
- Given: ViewportState at any zoom
- When: zoom_out() is called
- Then: viewport.zoom() always in [0.1, 4.0]

### given_any_viewport_when_any_zoom_operation_then_zoom_finite
- Given: ViewportState with any zoom value (including invalid)
- When: Any zoom operation is performed
- Then: viewport.zoom().is_finite() equals true

### given_viewport_at_max_when_zoom_in_idempotent
- Given: ViewportState at zoom 4.0
- When: zoom_in() is called 5 times
- Then: viewport.zoom() still equals 4.0

### given_viewport_at_min_when_zoom_out_idempotent
- Given: ViewportState at zoom 0.1
- When: zoom_out() is called 5 times
- Then: viewport.zoom() still equals 0.1

### given_viewport_with_nan_camera_when_zoom_then_camera_finite
- Given: ViewportState with camera_x=NaN, camera_y=NaN
- When: zoom_in() is called
- Then: camera_x().is_finite() equals true

### given_viewport_with_nan_camera_when_zoom_then_camera_y_finite
- Given: ViewportState with camera_x=NaN, camera_y=NaN
- When: zoom_in() is called
- Then: camera_y().is_finite() equals true

### given_any_viewport_when_zoom_around_point_then_result_in_bounds
- Given: ViewportState at any zoom
- When: zoom_around_point is called with any zoom value
- Then: viewport.zoom() always in [0.1, 4.0]

### given_any_viewport_when_zoom_around_point_then_result_finite
- Given: ViewportState at any zoom
- When: zoom_around_point is called with any zoom value
- Then: viewport.zoom().is_finite() equals true

### given_viewport_at_nonzero_camera_when_zoom_around_point_then_camera_x_unchanged
- Given: ViewportState at zoom 1.0 with camera_x=100.0, camera_y=50.0
- When: zoom_around_point(2.0, 100.0, 100.0) is called
- Then: viewport.camera_x() equals 100.0

### given_viewport_at_nonzero_camera_when_zoom_around_point_then_camera_y_unchanged
- Given: ViewportState at zoom 1.0 with camera_x=100.0, camera_y=50.0
- When: zoom_around_point(2.0, 100.0, 100.0) is called
- Then: viewport.camera_y() equals 50.0

### given_viewport_when_zoom_around_point_then_screen_point_remains_fixed
- Given: ViewportState at zoom 1.0, camera at (0.0, 0.0), viewport 800x600
- When: zoom_around_point(2.0, 400.0, 300.0) is called
- Then: The screen point (400.0, 300.0) maps to same world coordinates before and after
- Then: world_to_screen(screen_to_world(400.0, 300.0)) equals (400.0, 300.0)

### given_viewport_when_zoom_around_point_at_corner_then_screen_point_remains_fixed
- Given: ViewportState at zoom 1.0, camera at (0.0, 0.0), viewport 800x600
- When: zoom_around_point(2.0, 0.0, 0.0) is called
- Then: The screen point (0.0, 0.0) maps to same world coordinates before and after

### given_viewport_when_zoom_around_point_at_corner_then_screen_point_remains_fixed_opposite
- Given: ViewportState at zoom 1.0, camera at (0.0, 0.0), viewport 800x600
- When: zoom_around_point(2.0, 800.0, 600.0) is called
- Then: The screen point (800.0, 600.0) maps to same world coordinates before and after

---

## Property-Based Tests (proptest)

### property_zoom_around_point_always_in_bounds
- Given: Arbitrary valid zoom in (0.0, 100.0), arbitrary screen coordinates
- When: zoom_around_point is called
- Then: Result zoom always in [0.1, 4.0]
- Note: Use proptest to test thousands of random inputs

### property_zoom_around_point_camera_unchanged
- Given: Arbitrary camera position (camera_x, camera_y), arbitrary valid zoom, arbitrary screen point
- When: zoom_around_point is called
- Then: camera_x and camera_y remain unchanged
- Note: Use proptest with arbitrary camera positions

### property_zoom_around_point_screen_point_fixed
- Given: Arbitrary camera position, arbitrary valid zoom, arbitrary screen point (sx, sy)
- When: zoom_around_point is called
- Then: screen_to_world(sx, sy) returns same world coordinates before and after zoom
- Note: Use proptest to verify coordinate transform reversibility (I4)

### property_set_zoom_always_in_bounds
- Given: Arbitrary zoom value (including extreme values)
- When: set_zoom is called
- Then: Result always in [0.1, 4.0]
- Note: Use proptest with float_strategy

### property_zoom_in_always_in_bounds
- Given: Arbitrary current zoom value
- When: zoom_in is called
- Then: Result always in [0.1, 4.0]
- Note: Use proptest to test zoom sequences

### property_zoom_out_always_in_bounds
- Given: Arbitrary current zoom value
- When: zoom_out is called
- Then: Result always in [0.1, 4.0]
- Note: Use proptest to test zoom sequences

### property_zoom_around_point_zero_zoom_returns_false
- Given: Arbitrary viewport state, arbitrary screen coordinates
- When: zoom_around_point(0.0, sx, sy) is called
- Then: returns false and zoom unchanged

### property_zoom_around_point_negative_zoom_returns_false
- Given: Arbitrary viewport state, arbitrary screen coordinates
- When: zoom_around_point(-1.0, sx, sy) is called
- Then: returns false and zoom unchanged

---

## Contract Violation Tests

### given_viewport_when_set_zoom_nan_then_violation_returns_false
- Given: ViewportState at zoom 1.0
- When: set_zoom(f64::NAN) is called
- Then: returns false (not panic, not invalid state)

### given_viewport_when_set_zoom_inf_then_violation_returns_false
- Given: ViewportState at zoom 1.0
- When: set_zoom(f64::INFINITY) is called
- Then: returns false (not panic, not invalid state)

### given_viewport_when_set_zoom_100_then_violation_clamped_to_max
- Given: ViewportState at zoom 1.0
- When: set_zoom(100.0) is called
- Then: viewport.zoom() equals 4.0 (clamped, not 100.0)

### given_viewport_when_set_zoom_001_then_violation_clamped_to_min
- Given: ViewportState at zoom 1.0
- When: set_zoom(0.01) is called
- Then: viewport.zoom() equals 0.1 (clamped, not 0.01)

### given_viewport_at_max_when_zoom_in_then_violation_no_change
- Given: ViewportState at zoom 4.0
- When: zoom_in() is called
- Then: returns false (no change made)

### given_viewport_when_zoom_around_point_nan_then_violation_returns_false
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(f64::NAN, 100.0, 100.0) is called
- Then: returns false (not panic, not invalid state)

### given_viewport_when_zoom_around_point_inf_then_violation_returns_false
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(f64::INFINITY, 100.0, 100.0) is called
- Then: returns false (not panic, not invalid state)

### given_viewport_when_zoom_around_point_100_then_violation_clamped_to_max
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(100.0, 100.0, 100.0) is called
- Then: viewport.zoom() equals 4.0 (clamped, not 100.0)

### given_viewport_when_zoom_around_point_001_then_violation_clamped_to_min
- Given: ViewportState at zoom 1.0
- When: zoom_around_point(0.01, 100.0, 100.0) is called
- Then: viewport.zoom() equals 0.1 (clamped, not 0.01)

---

## Given-When-Then Scenarios

### Scenario 1: User zooms in past maximum
- Given: Viewport at zoom 3.0
- When: User triggers zoom_in (factor 1.25)
- Then: New zoom is 4.0 (clamped to max)
- Then: Returns false (no actual change)

### Scenario 2: User zooms out past minimum
- Given: Viewport at zoom 0.2
- When: User triggers zoom_out (factor 0.8)
- Then: New zoom is 0.1 (clamped to min)
- Then: Returns false (no actual change)

### Scenario 3: Rapid zoom in sequence
- Given: Viewport at zoom 1.0
- When: User zooms in 10 times
- Then: Zoom values: 1.25, 1.56, 1.95, 2.44, 3.05, 3.81, 4.0, 4.0, 4.0, 4.0
- Then: All values within [0.1, 4.0]

### Scenario 4: Set zoom directly to boundary
- Given: Viewport at zoom 1.0
- When: set_zoom(4.0) is called
- Then: Returns true
- Then: viewport.zoom() equals 4.0

### Scenario 5: Set zoom to invalid value
- Given: Viewport at zoom 2.0
- When: set_zoom(f64::NAN) is called
- Then: Returns false
- Then: viewport.zoom() still equals 2.0

### Scenario 6: Zoom around point to valid value
- Given: Viewport at zoom 1.0
- When: zoom_around_point(2.0, 100.0, 100.0) is called
- Then: Returns true
- Then: viewport.zoom() equals 2.0

### Scenario 7: Zoom around point past maximum
- Given: Viewport at zoom 2.0
- When: zoom_around_point(10.0, 100.0, 100.0) is called
- Then: Returns true (clamped to max)
- Then: viewport.zoom() equals 4.0

### Scenario 8: Zoom around point to NaN
- Given: Viewport at zoom 2.0
- When: zoom_around_point(f64::NAN, 100.0, 100.0) is called
- Then: Returns false
- Then: viewport.zoom() still equals 2.0

---

## Traceability Matrix

| Test ID | Contract Clause | Type |
|---------|----------------|------|
| given_viewport_at_default_zoom_when_set_zoom_to_middle_value_then_zoom_changes | Q1 | Happy |
| given_viewport_when_zoom_in_from_middle_then_zoom_increases | Q1 | Happy |
| given_viewport_when_zoom_out_from_middle_then_zoom_decreases | Q1 | Happy |
| given_viewport_when_set_zoom_above_max_then_clamps_to_max | Q1 | Edge |
| given_viewport_when_set_zoom_below_min_then_clamps_to_min | Q1 | Edge |
| given_viewport_at_max_zoom_when_zoom_in_then_returns_false | Q3 | Edge |
| given_viewport_at_min_zoom_when_zoom_out_then_returns_false | Q3 | Edge |
| given_viewport_when_set_zoom_to_nan_then_returns_false | P1 | Error |
| given_viewport_when_set_zoom_to_infinity_then_returns_false | P1 | Error |
| given_viewport_when_set_zoom_to_negative_then_returns_false | P2 | Error |
| given_viewport_when_set_zoom_to_zero_then_returns_false | P2 | Error |
| given_any_viewport_when_any_zoom_operation_then_zoom_finite | Q2 | Contract |
| given_viewport_with_nan_camera_when_zoom_then_camera_finite | Q5 | Contract |
| given_viewport_at_max_when_zoom_in_idempotent | I1 | Contract |
| given_viewport_at_default_zoom_when_zoom_around_point_to_valid_zoom_then_returns_true | Q1 | Happy |
| given_viewport_at_default_zoom_when_zoom_around_point_to_valid_zoom_then_zoom_changes | Q1 | Happy |
| given_viewport_at_default_zoom_when_zoom_around_point_at_different_point_then_returns_true | Q1 | Happy |
| given_viewport_when_zoom_around_point_to_nan_then_returns_false | P1 | Error |
| given_viewport_when_zoom_around_point_to_infinity_then_returns_false | P1 | Error |
| given_viewport_when_zoom_around_point_above_max_then_clamps_to_max | Q1 | Edge |
| given_viewport_when_zoom_around_point_below_min_then_clamps_to_min | Q1 | Edge |
| given_any_viewport_when_zoom_around_point_then_result_in_bounds | Q2 | Contract |
| given_any_viewport_when_zoom_around_point_then_result_finite | Q2 | Contract |
| given_viewport_when_zoom_around_point_100_then_violation_clamped_to_max | Q1 | Violation |
| given_viewport_when_zoom_around_point_001_then_violation_clamped_to_min | Q1 | Violation |
| given_viewport_when_zoom_around_point_nan_then_violation_returns_false | P1 | Violation |
| given_viewport_when_zoom_around_point_inf_then_violation_returns_false | P1 | Violation |
| given_viewport_when_zoom_around_point_to_negative_value_then_returns_false | P2 | Error |
| given_viewport_when_zoom_around_point_to_negative_value_then_zoom_unchanged | P2 | Error |
| given_viewport_when_zoom_around_point_to_zero_then_returns_false | P2 | Error |
| given_viewport_when_zoom_around_point_to_zero_then_zoom_unchanged | P2 | Error |
| given_viewport_when_zoom_around_point_with_negative_screen_x_then_returns_true | Q1 | Edge |
| given_viewport_when_zoom_around_point_with_negative_screen_y_then_returns_true | Q1 | Edge |
| given_viewport_when_zoom_around_point_with_negative_screen_coords_then_zoom_changes | Q1 | Edge |
| given_viewport_at_nonzero_camera_when_zoom_around_point_then_camera_x_unchanged | I2 | Contract |
| given_viewport_at_nonzero_camera_when_zoom_around_point_then_camera_y_unchanged | I3 | Contract |
| given_viewport_when_zoom_around_point_then_screen_point_remains_fixed | I4 | Contract |
| given_viewport_when_zoom_around_point_at_corner_then_screen_point_remains_fixed | I4 | Contract |
| given_viewport_when_zoom_around_point_at_corner_then_screen_point_remains_fixed_opposite | I4 | Contract |
| property_zoom_around_point_always_in_bounds | Q1 | Property |
| property_zoom_around_point_camera_unchanged | I2, I3 | Property |
| property_zoom_around_point_screen_point_fixed | I4 | Property |
| property_set_zoom_always_in_bounds | Q1 | Property |
| property_zoom_in_always_in_bounds | Q1 | Property |
| property_zoom_out_always_in_bounds | Q1 | Property |
| property_zoom_around_point_zero_zoom_returns_false | P2 | Property |
| property_zoom_around_point_negative_zoom_returns_false | P2 | Property |
