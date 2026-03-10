# Martin Fowler Test Plan

## Happy Path Tests
- test_returns_aabb_for_simple_straight_edge
- test_includes_stroke_width_in_bounds
- test_encloses_curved_connector_bezier_extents
- test_includes_bend_points_in_bounds

## Error Path Tests
- test_returns_error_when_source_has_nan_coordinates
- test_returns_error_when_target_has_infinite_coordinates
- test_returns_error_when_thickness_is_zero
- test_returns_error_when_thickness_is_negative

## Edge Case Tests
- test_handles_zero_length_edge_gracefully
- test_handles_vertical_edge_correctly
- test_handles_horizontal_edge_correctly
- test_handles_diagonal_edge_correctly

## Contract Verification Tests
- test_precondition_p1_source_valid
- test_precondition_p2_thickness_positive
- test_precondition_p3_coordinates_finite
- test_postcondition_q1_encloses_all_segments
- test_postcondition_q2_expanded_by_stroke
- test_invariant_i1_aabb_valid
- test_invariant_i2_contains_control_points

## Contract Violation Tests
- `test_p1_violation_returns_invalid_node_position`
  Given: edge_bounds with source position containing NaN (Point::new(f64::NAN, 0.0))
  When: function is called
  Then: returns `Err(EdgeBoundsError::InvalidNodePosition)` -- NOT a panic, NOT an unwrap failure

- `test_p2_violation_returns_invalid_thickness`
  Given: edge with thickness = 0.0
  When: edge_bounds is called
  Then: returns `Err(EdgeBoundsError::InvalidThickness)` -- NOT a panic, NOT an unwrap failure

- `test_p3_violation_returns_invalid_node_position`
  Given: target position with infinite coordinate (Point::new(f64::INFINITY, 0.0))
  When: edge_bounds is called
  Then: returns `Err(EdgeBoundsError::InvalidNodePosition)` -- NOT a panic, NOT an unwrap failure

## Given-When-Then Scenarios

### Scenario 1: Simple straight edge bounds
Given: Edge from (0, 0) to (100, 50) with thickness 2.0
When: edge_bounds is calculated
Then:
- min_x is approximately -1.0 (accounting for stroke)
- min_y is approximately -1.0
- max_x is approximately 101.0
- max_y is approximately 51.0

### Scenario 2: Curved connector with Bezier
Given: Edge with ArrowType::Curved from (0, 0) to (100, 0) with control point at (50, 50)
When: edge_bounds is calculated using QuadraticBezier
Then:
- bounds include the curve's apex at y=25
- bounds are tighter than simple AABB of all points

### Scenario 3: Edge with bend points
Given: Edge from (0, 0) to (100, 100) with bend_points at [(25, 0), (25, 100)]
When: edge_bounds is calculated
Then:
- bounds include all three segments
- bounds span from min_x=0 to max_x=100
- bounds span from min_y=0 to max_y=100

### Scenario 4: Edge with arrowhead extends bounds
Given: Edge from (0, 0) to (100, 0) with ArrowType::Default
When: edge_bounds is calculated
Then:
- bounds extend backward from target for arrowhead
- min_x < 100.0 (arrowhead area included)
