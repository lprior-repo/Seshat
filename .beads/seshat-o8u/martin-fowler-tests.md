# Martin Fowler Test Plan

## Happy Path Tests
- test_update_node_style_serialization_produces_correct_json
- test_update_node_style_deserialization_parses_valid_json
- test_update_node_style_roundtrip_preserves_equivalent_data
- test_update_node_style_projection_updates_style_field
- test_update_node_style_projection_preserves_other_fields
- test_update_node_style_all_variants_work

## Error Path Tests
- test_update_node_style_deserialization_fails_on_unknown_op_type
- test_update_node_style_deserialization_fails_on_missing_fields
- test_update_node_style_projection_returns_node_not_found_for_missing_node
- test_update_node_style_projection_fails_on_invalid_json

## Edge Case Tests
- test_update_node_style_box_variant_serializes_correctly
- test_update_node_style_cloud_variant_serializes_correctly
- test_update_node_style_cylinder_variant_serializes_correctly
- test_update_node_style_dashed_variant_serializes_correctly
- test_update_node_style_idempotent_apply_twice
- test_update_node_style_preserves_connected_edges

## Contract Verification Tests
- test_precondition_p1_test_infrastructure_imports_available
- test_precondition_p2_fixture_data_valid
- test_postcondition_q1_serialization_format_correct
- test_postcondition_q2_deserialization_format_correct
- test_postcondition_q3_roundtrip_preserves_data
- test_postcondition_q4_projection_applies_style
- test_postcondition_q5_error_returned_for_missing_node
- test_invariant_test_isolation
- test_invariant_deterministic_execution

## Contract Violation Tests
- `test_postcondition_q1_violation_wrong_json_format`
  Given: DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Box }
  When: Serializing to JSON
  Expected JSON: `{"op_type":"update_node_style","id":"n1","style":"box"}`
  Actual: Different JSON format
  Then: Test fails - serialization format incorrect

- `test_postcondition_q4_violation_style_not_applied`
  Given: apply_operation called with UpdateNodeStyle { id: "n1", style: NodeStyle::Cloud }
  When: Projection after apply
  Expected: projection.nodes["n1"].style == NodeStyle::Cloud
  Actual: style unchanged
  Then: Test fails - style not applied

- `test_postcondition_q5_violation_no_error_for_missing_node`
  Given: apply_operation called with UpdateNodeStyle { id: "n999", style: NodeStyle::Dashed }
  When: Operation applied
  Expected: Err(ReplayError::NodeNotFound)
  Actual: Ok(projection) - silently succeeded
  Then: Test fails - error not returned

## Given-When-Then Scenarios

### Scenario 1: Serialization produces correct format
Given: DomainOp::UpdateNodeStyle { id: "node1", style: NodeStyle::Box }
When: Serializing to JSON via serde_json::to_string
Then:
- Output string is valid JSON
- Contains "op_type" key with value "update_node_style"
- Contains "id" key with value "node1"
- Contains "style" key with value "box"

### Scenario 2: Deserialization parses correctly
Given: Valid JSON string `{"op_type":"update_node_style","id":"node1","style":"cloud"}`
When: Deserializing to DomainOp via serde_json::from_str
Then:
- Result is Ok(DomainOp::UpdateNodeStyle)
- id field equals "node1"
- style field equals NodeStyle::Cloud

### Scenario 3: Roundtrip preserves data
Given: Original DomainOp::UpdateNodeStyle { id: "test", style: NodeStyle::Cylinder }
When: Serializing to JSON, then deserializing back
Then:
- Deserialized DomainOp equals original
- No data loss in either direction

### Scenario 4: Projection applies style change
Given: DiagramProjection with node "n1" having NodeStyle::Box
When: Applying apply_operation(DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Dashed })
Then:
- Result is Ok
- New projection.nodes["n1"].style equals NodeStyle::Dashed

### Scenario 5: Projection preserves other fields
Given: DiagramProjection with node "n1" at position (100, 200) size (50, 30) label "Test"
When: Applying UpdateNodeStyle to change style to Cloud
Then:
- x remains 100
- y remains 200
- width remains 50
- height remains 30
- label remains "Test"
- Only style changes

### Scenario 6: Missing node returns error
Given: DiagramProjection without node "n999"
When: Applying UpdateNodeStyle { id: "n999", style: NodeStyle::Box }
Then:
- Returns Err(ReplayError::NodeNotFound)
- Error message indicates node not found

### Scenario 7: All NodeStyle variants serialize/deserialize
Given: Each NodeStyle variant (Box, Cloud, Cylinder, Dashed)
When: Serializing and deserializing each
Then:
- All variants roundtrip correctly
- No variant loses data

### Scenario 8: Idempotent operation
Given: DiagramProjection with node "n1" at NodeStyle::Box
When: Applying UpdateNodeStyle twice with NodeStyle::Box
Then:
- First result Ok, style = Box
- Second result Ok, style = Box
- Both results equal
