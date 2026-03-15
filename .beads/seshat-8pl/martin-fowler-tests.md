# Martin Fowler Test Plan

## Happy Path Tests
- test_coordinates_are_snapped_to_grid_when_enabled
- test_coordinates_remain_unchanged_when_snapping_disabled
- test_grid_size_is_created_with_valid_dimensions

## Error Path Tests
- test_rejects_out_of_bounds_grid_sizes
- test_rejects_non_finite_grid_sizes
- test_rejects_non_finite_coordinates_x_nonfinite
- test_rejects_non_finite_coordinates_y_nonfinite
- test_rejects_non_finite_coordinates_both_nonfinite

## Edge Case Tests
- test_snapping_handles_exact_grid_multiples
- test_snapping_resolves_midway_ties_deterministically_positive
- test_snapping_resolves_midway_ties_deterministically_negative
- test_snapping_handles_negative_coordinates
- test_snapping_works_at_minimum_and_maximum_grid_boundaries
- test_snapping_handles_zero_coordinate

## Property-Based Tests
- proptest_q1_disabled_snap_returns_exact_raw_coordinate
- proptest_q2_enabled_snap_is_multiple_of_grid
- proptest_q3_snap_distance_never_exceeds_half_grid
- proptest_q4_midway_ties_always_round_away_from_zero
- proptest_q5_finite_inputs_always_yield_finite_outputs

## Contract Verification Tests
- test_precondition_finite_coordinates_are_required
- test_precondition_grid_size_must_be_within_bounds
- test_snapping_is_ignored_when_disabled
- test_snapping_aligns_to_grid_multiples
- test_snapping_distance_never_exceeds_half_grid

## Contract Violation Tests
- `test_finite_coordinate_violation_returns_not_finite_error_x`
  Given: `snap_node_coordinates((f64::NAN, 15.0), SnapMode::Enabled, valid_grid)`
  When: function is called with a non-finite X coordinate value
  Then: returns `Err(GridSnapError::NotFinite)`

- `test_finite_coordinate_violation_returns_not_finite_error_y`
  Given: `snap_node_coordinates((15.0, f64::NAN), SnapMode::Enabled, valid_grid)`
  When: function is called with a non-finite Y coordinate value
  Then: returns `Err(GridSnapError::NotFinite)`

- `test_finite_coordinate_violation_returns_not_finite_error_both`
  Given: `snap_node_coordinates((f64::NAN, f64::INFINITY), SnapMode::Enabled, valid_grid)`
  When: function is called with both non-finite coordinate values
  Then: returns `Err(GridSnapError::NotFinite)`

- `test_grid_size_out_of_bounds_violation_returns_error_low`
  Given: `try_grid_size(9.9)`
  When: function is called with a value below the 10.0 minimum
  Then: returns `Err(GridError::OutOfRange)`

- `test_grid_size_out_of_bounds_violation_returns_error_high`
  Given: `try_grid_size(100.1)`
  When: function is called with a value above the 100.0 maximum
  Then: returns `Err(GridError::OutOfRange)`

- `test_grid_size_non_finite_violation_returns_error`
  Given: `try_grid_size(f64::NAN)`
  When: function is called with a non-finite value
  Then: returns `Err(GridError::NotFinite)`

## Given-When-Then Scenarios
### Scenario 1: Free coordinate movement with snapping disabled
Given: A coordinate and snapping is disabled
When: The coordinate is evaluated for snapping at `(15.7, -42.3)`
Then:
- The returned coordinate exactly matches the raw coordinate `(15.7, -42.3)`
- No grid alignment is applied

### Scenario 2: Precise grid alignment for coordinates
Given: A grid size of `20.0` and snapping is enabled
When: A coordinate is evaluated for snapping at `(29.0, 41.0)`
Then:
- The X coordinate aligns to the nearest grid step at `20.0`
- The Y coordinate aligns to the nearest grid step at `40.0`
- The final snapped coordinate is `(20.0, 40.0)`

### Scenario 3: Deterministic tie-breaking at midpoint boundaries
Given: A grid size of `20.0` and snapping is enabled
When: A coordinate is placed exactly at the midpoint raw value of `10.0`
Then:
- The tie-break logic consistently rounds away from zero
- The final snapped coordinate aligns to `20.0`