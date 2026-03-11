# Martin Fowler Test Plan

## Happy Path Tests
- `edg_026_test_default_edge_style_is_solid`
- `edg_027_test_set_edge_style_to_dashed_updates_successfully`
- `edg_028_test_set_edge_style_to_dotted_updates_successfully`
- `edg_030_test_set_edge_style_to_same_style_is_idempotent`

## Error Path Tests
- `edg_029_test_returns_error_when_setting_style_on_missing_edge`

## Edge Case Tests
- `test_handles_applying_style_to_newly_connected_edge`
- `test_style_persists_across_multiple_operations`

## Contract Verification Tests
- `test_precondition_target_edge_must_exist`
- `test_postcondition_edge_style_is_updated`
- `test_postcondition_other_properties_unchanged`
- `test_postcondition_other_edges_unchanged`
- `test_invariant_projection_structural_integrity_preserved`

## Contract Violation Tests
- `test_edge_exists_violation_returns_not_found_error`
  Given: `apply_edge_style(state, "missing-edge", EdgeStyle::Dashed)`
  When: function is called with a non-existent edge ID
  Then: returns `Err(EdgeOpsError::EdgeNotFound("missing-edge".to_string()))` -- NOT a panic, NOT an unwrap failure

## Given-When-Then Scenarios
### Scenario 1: Applying Dashed Style to an Existing Edge
Given: A `DiagramProjection` with an existing edge "edge-1" connecting "node-a" and "node-b".
When: `apply_edge_style` is invoked with id "edge-1" and `EdgeStyle::Dashed`.
Then:
- The operation returns `Ok(new_state)`.
- The edge "edge-1" in `new_state` has its `style` set to `EdgeStyle::Dashed`.
- All other properties of "edge-1" remain exactly the same as in the original state.

### Scenario 2: Applying Style to a Non-existent Edge
Given: A `DiagramProjection` that does not contain an edge with ID "ghost-edge".
When: `apply_edge_style` is invoked with id "ghost-edge" and `EdgeStyle::Dotted`.
Then:
- The operation returns an error.
- The error is strictly `EdgeOpsError::EdgeNotFound("ghost-edge")`.