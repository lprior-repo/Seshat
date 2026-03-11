# Martin Fowler Test Plan: Subgraph Events (SUB-019 to SUB-024)

## Happy Path Tests
- `test_subgraph_bounds_expand_when_child_added` (SUB-019)
- `test_subgraph_z_index_orders_children_above_container` (SUB-020)
- `test_add_node_updates_parent_reference` (SUB-021)
- `test_remove_node_clears_parent_reference` (SUB-022)
- `test_batch_add_updates_multiple_nodes_and_bounds_once` (SUB-023)
- `test_remove_all_nodes_leaves_empty_container` (SUB-024)

## Error Path Tests
- `test_add_node_returns_error_when_subgraph_not_found`
- `test_add_node_returns_error_when_child_not_found`
- `test_add_node_returns_error_on_cycle_detection`

## Edge Case Tests
- `test_subgraph_bounds_contract_when_outlier_child_removed`
- `test_batch_add_with_empty_list_does_nothing`
- `test_remove_node_that_has_no_parent_is_noop_or_error`

## Contract Verification Tests
- `test_invariant_child_has_only_one_parent`
- `test_invariant_bounds_contain_all_children`

## Contract Violation Tests
- `test_p1_violation_returns_node_not_found_error`
  Given: A `DiagramState` without `subgraph_id`.
  When: `add_node_to_subgraph(child_id, subgraph_id, &mut state)` is called.
  Then: returns `Err(Error::NodeNotFound(subgraph_id))`

- `test_p2_violation_returns_node_not_found_error`
  Given: A `DiagramState` without `child_id`.
  When: `add_node_to_subgraph(child_id, subgraph_id, &mut state)` is called.
  Then: returns `Err(Error::NodeNotFound(child_id))`

- `test_p3_violation_returns_cycle_detected_error`
  Given: A `DiagramState` where `node_a` is the parent of `node_b`.
  When: `add_node_to_subgraph(node_a, node_b, &mut state)` is called.
  Then: returns `Err(Error::CycleDetected(node_a, node_b))`

## Given-When-Then Scenarios

### Scenario 1: SUB-019 Subgraph bounds calculation
Given: A subgraph `S1` containing node `N1` at `(0,0)` with size `(10,10)`.
When: A new node `N2` at `(20,20)` with size `(10,10)` is added to `S1`.
Then: 
- `N2`'s `parent_id` is `S1`.
- `S1`'s bounds are re-calculated to `(0,0, 30,30)` (plus padding).

### Scenario 2: SUB-023 Add multiple nodes in batch
Given: A subgraph `S1` and three unparented nodes `N1`, `N2`, `N3`.
When: `batch_add_nodes_to_subgraph(&[N1, N2, N3], S1, &mut state)` is called.
Then:
- `N1`, `N2`, and `N3` all have `parent_id == S1`.
- The bounds calculation function is invoked only once after all nodes are parented.

### Scenario 3: SUB-024 Remove all nodes preserves container
Given: A subgraph `S1` containing nodes `N1` and `N2`.
When: `remove_all_nodes_from_subgraph(S1, &mut state)` is called.
Then:
- `N1` and `N2` have `parent_id == None`.
- `S1` still exists in the diagram state.
- `S1`'s bounds shrink to its defined minimum empty size.
