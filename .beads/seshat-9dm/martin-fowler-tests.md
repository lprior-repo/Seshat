# Martin Fowler Test Plan

## Happy Path Tests
- test_apply_update_node_style_success_with_valid_node
- test_apply_update_node_style_returns_ok_with_updated_style
- test_apply_update_node_style_preserves_all_other_node_fields
- test_apply_update_node_style_does_not_affect_other_nodes
- test_apply_update_node_style_functional_update_returns_new_projection

## Error Path Tests
- test_apply_update_node_style_returns_node_not_found_for_missing_node
- test_apply_update_node_style_returns_error_when_node_does_not_exist

## Edge Case Tests
- test_apply_update_node_style_all_four_style_variants
- test_apply_update_node_style_idempotent_same_style_twice
- test_apply_update_node_style_multiple_nodes_unaffected
- test_apply_update_node_style_preserves_edges

## Contract Verification Tests
- test_precondition_p1_operation_is_update_nodestyle
- test_precondition_p2_node_exists_runtime_check
- test_precondition_p3_valid_nodestyle_compile_time
- test_postcondition_q1_style_updated_correctly
- test_postcondition_q2_other_fields_unchanged
- test_postcondition_q3_other_nodes_unchanged
- test_postcondition_q4_returns_ok_on_success
- test_invariant_node_count_preserved
- test_invariant_edge_integrity_maintained

## Contract Violation Tests
- `test_precondition_p2_violation_node_not_found_returns_error`
  Given: apply_update_node_style called with non-existent node id
  When: apply_update_node_style(state, "n999", NodeStyle::Box) where n999 not in state.nodes
  Then: Returns `Err(ReplayError::NodeNotFound)`

- `test_postcondition_q1_violation_style_not_updated`
  Given: UpdateNodeStyle operation applied to existing node
  When: DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Cloud }
  After: state.nodes["n1"].style != NodeStyle::Cloud
  Then: FAIL - style should equal Cloud

- `test_postcondition_q2_violation_other_fields_changed`
  Given: UpdateNodeStyle operation applied to node at position (10, 20)
  When: DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Dashed }
  After: state.nodes["n1"].x != 10 OR state.nodes["n1"].y != 20
  Then: FAIL - position fields should remain unchanged

## Given-When-Then Scenarios

### Scenario 1: Apply UpdateNodeStyle to existing node
Given: DiagramProjection with node "n1" having NodeStyle::Box
When: Applying DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Cloud }
Then:
- Returns Ok with new DiagramProjection
- projection.nodes["n1"].style equals NodeStyle::Cloud
- projection.nodes["n1"].x equals original x
- projection.nodes["n1"].y equals original y
- projection.nodes["n1"].width equals original width
- projection.nodes["n1"].height equals original height
- projection.nodes["n1"].label equals original label
- All other nodes unchanged

### Scenario 2: Apply UpdateNodeStyle to non-existent node
Given: DiagramProjection with nodes "n1", "n2" (no "n999")
When: Applying DomainOp::UpdateNodeStyle { id: "n999", style: NodeStyle::Cylinder }
Then:
- Returns Err(ReplayError::NodeNotFound)
- Original projection unchanged (functional update pattern)

### Scenario 3: Apply UpdateNodeStyle preserves edges
Given: DiagramProjection with node "n1" connected to node "n2" via edge "e1"
When: Applying DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Box }
Then:
- Edge "e1" style unchanged
- Edge "e1" source/target unchanged
- Edge "e1" label unchanged

### Scenario 4: UpdateNodeStyle is idempotent
Given: DiagramProjection with node "n1" having NodeStyle::Box
When: Applying UpdateNodeStyle twice with same style
Then:
- First apply returns Ok with style = Box
- Second apply returns Ok with style = Box
- Both return equivalent projections

### Scenario 5: Multiple style variants work correctly
Given: DiagramProjection with node "n1"
When: Applying UpdateNodeStyle with each NodeStyle variant
Then:
- NodeStyle::Box works
- NodeStyle::Cloud works
- NodeStyle::Cylinder works
- NodeStyle::Dashed works

### Scenario 6: Functional update pattern
Given: Original DiagramProjection with node "n1"
When: Calling apply_update_node_style
Then:
- Original projection is not mutated
- New projection is returned with updated style
- Can chain multiple operations
