# Martin Fowler Test Plan: EDG-006 to EDG-010

## Happy Path Tests
- `test_returns_success_when_points_are_vertically_aligned` (EDG-006)
- `test_returns_success_when_points_are_horizontally_aligned` (EDG-006)
- `test_returns_success_l_shape_when_points_are_diagonal` (EDG-007)
- `test_returns_success_detour_when_avoiding_obstacle` (EDG-008)
- `test_returns_symmetric_route_when_endpoints_are_swapped`

## Error Path Tests
- `test_returns_error_when_start_point_is_nan`
- `test_returns_error_when_end_point_is_infinity`
- `test_returns_error_when_start_and_end_are_identical`
- `test_returns_error_when_start_point_inside_obstacle` (EDG-009)
- `test_returns_error_when_end_point_inside_obstacle` (EDG-009)

## Edge Case Tests
- `test_handles_points_with_sub_pixel_differences_as_identical`
- `test_detour_margin_calculation_near_boundaries`
- `test_obstacle_avoidance_when_route_barely_touches_edge` (Tangent check)

## Contract Verification Tests
- `test_postcondition_route_has_minimum_two_points`
- `test_postcondition_all_segments_are_strictly_orthogonal` (EDG-010)
- `test_postcondition_start_and_end_points_match_input`
- `test_postcondition_route_never_intersects_obstacle_interior`

## Contract Violation Tests
- `test_p1_violation_returns_invalid_endpoint`
  Given: `compute_orthogonal_route(Point::new(f64::NAN, 0.0), Point::new(10.0, 10.0))`
  When: function is called with invalid endpoints
  Then: returns `Err(RoutingError::InvalidEndpoint)`

- `test_p2_violation_returns_degenerate_route`
  Given: `compute_orthogonal_route(Point::new(5.0, 5.0), Point::new(5.0, 5.0))`
  When: function is called with identical points
  Then: returns `Err(RoutingError::DegenerateRoute)`

- `test_p3_violation_returns_endpoint_inside_obstacle`
  Given: `compute_orthogonal_route_avoiding(Point::new(50.0, 50.0), Point::new(200.0, 200.0), &AABB::new(0.0, 0.0, 100.0, 100.0))`
  When: function is called with start point inside the obstacle
  Then: returns `Err(RoutingError::EndpointInsideObstacle)`

## Given-When-Then Scenarios

### Scenario 1: Basic Orthogonal Route (EDG-006/007)
Given: An empty canvas
When: `compute_orthogonal_route` is requested from (0, 0) to (100, 50)
Then:
- It returns an `Ok(OrthogonalRoute)`
- The route contains 3 points (0, 0) -> (0, 50) -> (100, 50)
- All segments are perfectly orthogonal.

### Scenario 2: Obstacle Avoidance Detour (EDG-008)
Given: An obstacle at (50, 0) with width 50, height 50
When: `compute_orthogonal_route_avoiding` is requested from (0, 25) to (150, 25)
Then:
- It returns an `Ok(OrthogonalRoute)`
- The route includes detour points correctly avoiding the obstacle with a 10.0 margin
- No segment intersects the obstacle (50..100, 0..50).

### Scenario 3: Endpoint Inside Obstacle Rejection (EDG-009)
Given: An obstacle at (0, 0) to (100, 100)
When: `compute_orthogonal_route_avoiding` is requested from (50, 50) to (200, 200)
Then:
- It immediately returns `Err(RoutingError::EndpointInsideObstacle)`
- No partial route is returned.

### Scenario 4: Validating Strict Orthogonality (EDG-010)
Given: A successful route calculation
When: The resulting `OrthogonalRoute` is verified
Then:
- Every pair of sequential points has either identical X coordinates or identical Y coordinates
- No diagonal segments are allowed.
