# Martin Fowler Test Plan: Straight-Line Edge Routing (seshat-q79)

## Happy Path Tests
- `test_compute_straight_line_route_between_centers_successfully`
- `test_compute_straight_line_route_between_named_ports_successfully`
- `test_compute_straight_line_route_between_custom_ports_successfully`

## Error Path Tests
- `test_returns_error_when_source_node_missing`
- `test_returns_error_when_target_node_missing`

## Edge Case Tests
- `test_compute_straight_line_route_for_self_loop_returns_same_points`
- `test_compute_straight_line_route_with_zero_sized_nodes`

## Contract Verification Tests
- `test_precondition_source_node_exists`
- `test_precondition_target_node_exists`
- `test_postcondition_points_match_port_computations`

## Contract Violation Tests
- `test_p1_violation_returns_source_not_found`
  Given: A document missing node "N1" and an edge with source "N1".
  When: `compute_straight_line_route` is called.
  Then: Returns `Err(RoutingError::SourceNotFound("N1"))`.

- `test_p2_violation_returns_target_not_found`
  Given: A document missing node "N2" and an edge with target "N2".
  When: `compute_straight_line_route` is called.
  Then: Returns `Err(RoutingError::TargetNotFound("N2"))`.

## Given-When-Then Scenarios
### Scenario 1: Routing between two nodes with default ports (centers)
Given: Document with Node A at (0,0) size (100,100) and Node B at (200,0) size (100,100).
Given: Edge from A to B with no ports specified.
When: `compute_straight_line_route` is called.
Then: Start point is (50, 50) and End point is (250, 50).

### Scenario 2: Routing with explicit Top and Bottom ports
Given: Document with Node A at (0,0) size (100,100) and Node B at (0,200) size (100,100).
Given: Edge from A (source_port: Bottom) to B (target_port: Top).
When: `compute_straight_line_route` is called.
Then: Start point is (50, 100) and End point is (50, 200).
