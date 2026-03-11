# Martin Fowler Test Plan

## Happy Path Tests
- `test_MUL_006_translate_single_node_updates_coordinates`
- `test_MUL_007_translate_multiple_nodes_updates_all_coordinates`

## Error Path Tests
- `test_MUL_008_translate_empty_selection_returns_error`
- `test_MUL_009_translate_with_locked_node_returns_error_and_does_not_translate`

## Edge Case Tests
- `test_translate_by_zero_delta_succeeds_without_modifying_coordinates`
- `test_translate_negative_delta_moves_nodes_up_and_left`

## Contract Verification Tests
- `test_precondition_selection_not_empty`
- `test_precondition_no_locked_nodes`
- `test_postcondition_unselected_nodes_unmodified`
- `test_postcondition_ancestor_containers_recomputed`
- `test_invariant_node_count_remains_unchanged`
- `test_invariant_selection_remains_unchanged`

## Contract Violation Tests
- `test_P1_violation_empty_selection_returns_empty_selection_error`
  Given: A document with no nodes selected
  When: `translate_selection(doc, 10.0, 10.0)` is called
  Then: returns `Err(TransformError::EmptySelection)`

- `test_P2_violation_locked_node_returns_locked_node_error`
  Given: A document with two selected nodes, one of which is locked
  When: `translate_selection(doc, 10.0, 10.0)` is called
  Then: returns `Err(TransformError::LockedNode(locked_id))` and NO nodes are translated

- `test_P3_violation_nan_delta_returns_invalid_delta_error`
  Given: A document with a selected node
  When: `translate_selection(doc, f64::NAN, 10.0)` is called
  Then: returns `Err(TransformError::InvalidDelta)`

## Given-When-Then Scenarios
### Scenario 1: Group Translate with Container Bounds Update (MUL-010)
Given: A document with 3 nodes (A, B, C), where A and B are selected and C is not. A is a child of Container D.
When: `translate_selection` is called with dx=20.0, dy=-10.0
Then:
- Node A's x is increased by 20.0, y is decreased by 10.0
- Node B's x is increased by 20.0, y is decreased by 10.0
- Node C's coordinates are exactly the same
- Container D's bounds are recomputed based on A's new position
- The function returns `Ok(())`
