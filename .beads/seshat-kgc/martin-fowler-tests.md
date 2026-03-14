# Martin Fowler Test Plan: Marquee Performance (seshat-kgc)

## Happy Path Tests
- test_marquee_query_returns_correct_nodes_for_small_set
- test_marquee_query_scales_to_3000_nodes_within_time_limit
- test_marquee_query_handles_rotated_nodes_correctly

## Error Path Tests
- test_returns_error_for_negative_marquee_dimensions
- test_returns_error_if_index_uninitialized

## Edge Case Tests
- test_handles_empty_document
- test_handles_nodes_exactly_on_grid_boundaries
- test_handles_extremely_large_diagram_bounds

## Performance Tests
- test_benchmark_marquee_3000_nodes
  Given: A document with 3000 nodes randomly distributed.
  When: Marquee selection is performed.
  Then: Execution time is < 16ms.

## Contract Verification Tests
- test_precondition_non_negative_marquee
- test_precondition_index_initialization
- test_postcondition_result_parity_with_linear_scan
- test_postcondition_contain_mode_strictness
- test_postcondition_rotated_node_handling

## Contract Violation Tests
- `test_invalid_marquee_bounds_violation_returns_error`
  Given: `Rect::new(0.0, 0.0, -10.0, 10.0)`
  When: function is called
  Then: returns `Err(InvalidMarqueeBounds)`
- `test_uninitialized_index_violation_returns_error`
  Given: An uninitialized `SpatialIndex`
  When: `query_spatial_index` is called
  Then: returns `Err(IndexNotInitialized)`
- `test_performance_target_violation`
  Given: 3000 nodes and a query that artificially hangs for 20ms
  When: `query_spatial_index` is called
  Then: returns `Err(PerformanceTargetViolated)` or triggers `debug_assert`
- `test_result_parity_violation`
  Given: A discrepancy between linear and indexed results
  When: queried
  Then: returns `Err(PostconditionViolated)`

## Given-When-Then Scenarios
### Scenario 1: Large Scale Selection
Given: A DiagramDocument with 3000 nodes in a 10000x10000 area.
When: A marquee covering 10% of the area is queried.
Then: 
- Result contains all nodes inside the area.
- Query takes less than 16ms.
