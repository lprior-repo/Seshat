# Martin Fowler Test Plan

## Happy Path Tests
- test_apply_update_edge_style_success_with_valid_edge
- test_apply_update_edge_style_returns_ok_with_updated_style
- test_apply_update_edge_style_preserves_all_other_edge_fields
- test_apply_update_edge_style_does_not_affect_other_edges
- test_apply_update_edge_style_functional_update_returns_new_projection

## Error Path Tests
- test_apply_update_edge_style_returns_edge_not_found_for_missing_edge
- test_apply_update_edge_style_returns_error_when_edge_does_not_exist

## Edge Case Tests
- test_apply_update_edge_style_all_three_style_variants
- test_apply_update_edge_style_idempotent_same_style_twice
- test_apply_update_edge_style_multiple_edges_unaffected
- test_apply_update_edge_style_preserves_connected_nodes

## Contract Verification Tests
- test_precondition_p1_operation_is_update_edgestyle
- test_precondition_p2_edge_exists_runtime_check
- test_precondition_p3_valid_edgestyle_compile_time
- test_postcondition_q1_style_updated_correctly
- test_postcondition_q2_other_fields_unchanged
- test_postcondition_q3_other_edges_unchanged
- test_postcondition_q4_returns_ok_on_success
- test_invariant_edge_count_preserved
- test_invariant_node_integrity_maintained

## Contract Violation Tests
- `test_precondition_p2_violation_edge_not_found_returns_error`
  Given: apply_update_edge_style called with non-existent edge id
  When: apply_update_edge_style(state, "e999", EdgeStyle::Dashed) where e999 not in state.edges
  Then: Returns `Err(ReplayError::EdgeNotFound)`

- `test_postcondition_q1_violation_style_not_updated`
  Given: UpdateEdgeStyle operation applied to existing edge
  When: DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dotted }
  After: state.edges["e1"].style != EdgeStyle::Dotted
  Then: FAIL - style should equal Dotted

- `test_postcondition_q2_violation_other_fields_changed`
  Given: UpdateEdgeStyle operation applied to edge with source "n1", target "n2"
  When: DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed }
  After: state.edges["e1"].source != "n1" OR state.edges["e1"].target != "n2"
  Then: FAIL - connectivity fields should remain unchanged

## Given-When-Then Scenarios

### Scenario 1: Apply UpdateEdgeStyle to existing edge
Given: DiagramProjection with edge "e1" having EdgeStyle::Solid
When: Applying DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed }
Then:
- Returns Ok with new DiagramProjection
- projection.edges["e1"].style equals EdgeStyle::Dashed
- projection.edges["e1"].source equals original source
- projection.edges["e1"].target equals original target
- projection.edges["e1"].label equals original label
- projection.edges["e1"].thickness equals original thickness
- projection.edges["e1"].arrow_type equals original arrow_type
- All other edges unchanged

### Scenario 2: Apply UpdateEdgeStyle to non-existent edge
Given: DiagramProjection with edges "e1", "e2" (no "e999")
When: Applying DomainOp::UpdateEdgeStyle { id: "e999", style: EdgeStyle::Dotted }
Then:
- Returns Err(ReplayError::EdgeNotFound)
- Original projection unchanged (functional update pattern)

### Scenario 3: Apply UpdateEdgeStyle preserves nodes
Given: DiagramProjection with edge "e1" connecting node "n1" to node "n2"
When: Applying DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Solid }
Then:
- Node "n1" unchanged
- Node "n2" unchanged
- Node positions unchanged

### Scenario 4: UpdateEdgeStyle is idempotent
Given: DiagramProjection with edge "e1" having EdgeStyle::Solid
When: Applying UpdateEdgeStyle twice with same style
Then:
- First apply returns Ok with style = Solid
- Second apply returns Ok with style = Solid
- Both return equivalent projections

### Scenario 5: Multiple style variants work correctly
Given: DiagramProjection with edge "e1"
When: Applying UpdateEdgeStyle with each EdgeStyle variant
Then:
- EdgeStyle::Solid works
- EdgeStyle::Dashed works
- EdgeStyle::Dotted works

### Scenario 6: Functional update pattern
Given: Original DiagramProjection with edge "e1"
When: Calling apply_update_edge_style
Then:
- Original projection is not mutated
- New projection is returned with updated style
- Can chain multiple operations
