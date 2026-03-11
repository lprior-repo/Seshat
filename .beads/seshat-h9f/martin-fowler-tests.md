# Martin Fowler Test Plan

## Happy Path Tests
- test_update_edge_style_serialization_produces_correct_json
- test_update_edge_style_deserialization_parses_valid_json
- test_update_edge_style_roundtrip_preserves_equivalent_data
- test_update_edge_style_projection_updates_style_field
- test_update_edge_style_projection_preserves_other_fields
- test_update_edge_style_all_variants_work

## Error Path Tests
- test_update_edge_style_deserialization_fails_on_unknown_op_type
- test_update_edge_style_deserialization_fails_on_missing_fields
- test_update_edge_style_projection_returns_edge_not_found_for_missing_edge
- test_update_edge_style_projection_fails_on_invalid_json

## Edge Case Tests
- test_update_edge_style_solid_variant_serializes_correctly
- test_update_edge_style_dashed_variant_serializes_correctly
- test_update_edge_style_dotted_variant_serializes_correctly
- test_update_edge_style_idempotent_apply_twice
- test_update_edge_style_preserves_connected_nodes

## Contract Verification Tests
- test_precondition_p1_test_infrastructure_imports_available
- test_precondition_p2_fixture_data_valid
- test_postcondition_q1_serialization_format_correct
- test_postcondition_q2_deserialization_format_correct
- test_postcondition_q3_roundtrip_preserves_data
- test_postcondition_q4_projection_applies_style
- test_postcondition_q5_error_returned_for_missing_edge
- test_invariant_test_isolation
- test_invariant_deterministic_execution

## Contract Violation Tests
- `test_postcondition_q1_violation_wrong_json_format`
  Given: DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Solid }
  When: Serializing to JSON
  Expected JSON: `{"op_type":"update_edge_style","id":"e1","style":"solid"}`
  Actual: Different JSON format
  Then: Test fails - serialization format incorrect

- `test_postcondition_q4_violation_style_not_applied`
  Given: apply_operation called with UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed }
  When: Projection after apply
  Expected: projection.edges["e1"].style == EdgeStyle::Dashed
  Actual: style unchanged
  Then: Test fails - style not applied

- `test_postcondition_q5_violation_no_error_for_missing_edge`
  Given: apply_operation called with UpdateEdgeStyle { id: "e999", style: EdgeStyle::Dotted }
  When: Operation applied
  Expected: Err(ReplayError::EdgeNotFound)
  Actual: Ok(projection) - silently succeeded
  Then: Test fails - error not returned

## Given-When-Then Scenarios

### Scenario 1: Serialization produces correct format
Given: DomainOp::UpdateEdgeStyle { id: "edge1", style: EdgeStyle::Solid }
When: Serializing to JSON via serde_json::to_string
Then:
- Output string is valid JSON
- Contains "op_type" key with value "update_edge_style"
- Contains "id" key with value "edge1"
- Contains "style" key with value "solid"

### Scenario 2: Deserialization parses correctly
Given: Valid JSON string `{"op_type":"update_edge_style","id":"edge1","style":"dashed"}`
When: Deserializing to DomainOp via serde_json::from_str
Then:
- Result is Ok(DomainOp::UpdateEdgeStyle)
- id field equals "edge1"
- style field equals EdgeStyle::Dashed

### Scenario 3: Roundtrip preserves data
Given: Original DomainOp::UpdateEdgeStyle { id: "test", style: EdgeStyle::Dotted }
When: Serializing to JSON, then deserializing back
Then:
- Deserialized DomainOp equals original
- No data loss in either direction

### Scenario 4: Projection applies style change
Given: DiagramProjection with edge "e1" having EdgeStyle::Solid
When: Applying apply_operation(DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed })
Then:
- Result is Ok
- New projection.edges["e1"].style equals EdgeStyle::Dashed

### Scenario 5: Projection preserves other fields
Given: DiagramProjection with edge "e1" source "n1" target "n2" label "connects" thickness 2
When: Applying UpdateEdgeStyle to change style to Dotted
Then:
- source remains "n1"
- target remains "n2"
- label remains "connects"
- thickness remains 2
- arrow_type unchanged
- Only style changes

### Scenario 6: Missing edge returns error
Given: DiagramProjection without edge "e999"
When: Applying UpdateEdgeStyle { id: "e999", style: EdgeStyle::Solid }
Then:
- Returns Err(ReplayError::EdgeNotFound)
- Error message indicates edge not found

### Scenario 7: All EdgeStyle variants serialize/deserialize
Given: Each EdgeStyle variant (Solid, Dashed, Dotted)
When: Serializing and deserializing each
Then:
- All variants roundtrip correctly
- No variant loses data

### Scenario 8: Idempotent operation
Given: DiagramProjection with edge "e1" at EdgeStyle::Solid
When: Applying UpdateEdgeStyle twice with EdgeStyle::Solid
Then:
- First result Ok, style = Solid
- Second result Ok, style = Solid
- Both results equal
