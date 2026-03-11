# Martin Fowler Test Plan

## Happy Path Tests
- test_update_node_style_variant_constructable_with_valid_fields
- test_update_node_style_serializes_to_correct_json
- test_update_node_style_deserializes_from_valid_json
- test_update_node_style_kind_returns_node
- test_update_node_style_clone_produces_equivalent_copy

## Error Path Tests
- test_update_node_style_deserialization_fails_with_unknown_op_type
- test_update_node_style_deserialization_fails_with_missing_required_fields
- test_update_node_style_deserialization_fails_with_invalid_style_value

## Edge Case Tests
- test_update_node_style_all_four_style_variants
- test_update_node_style_with_various_valid_node_ids
- test_update_node_style_serialization_preserves_all_fields

## Contract Verification Tests
- test_precondition_valid_nodestyle_compile_time_enforced
- test_precondition_valid_nodeid_runtime_check
- test_postcondition_variant_exists_and_constructable
- test_postcondition_serialization_format_correct
- test_postcondition_deserialization_format_correct
- test_postcondition_kind_classification_correct
- test_invariant_domainop_exhaustiveness
- test_invariant_serialization_roundtrip

## Contract Violation Tests
- `test_precondition_p1_invalid_style_value_returns_compile_error`
  Given: Attempting to construct DomainOp::UpdateNodeStyle with invalid style value
  When: style = "invalid_shape" (not a NodeStyle variant)
  Then: Compile-time error - Rust enum prevents invalid variants

- `test_precondition_p2_empty_nodeid_returns_error`
  Given: DomainOp::UpdateNodeStyle with empty id field
  When: id = ""
  Then: Returns `Err(Error::InvalidInput)` or precondition check fails

- `test_postcondition_q3_unknown_op_type_deserialization_fails`
  Given: JSON with unknown op_type field
  When: JSON `{"op_type": "update_node_style_unknown", "id": "n1", "style": "box"}`
  Then: Returns `Err(serde_json::Error)` - unknown variant

- `test_postcondition_q4_kind_returns_wrong_opkind_fails`
  Given: DomainOp::UpdateNodeStyle instance
  When: Calling .kind() method
  Then: Returns OpKind::Node (not Edge, ZOrder, or Composite)

## Given-When-Then Scenarios

### Scenario 1: Construct UpdateNodeStyle with Box style
Given: A valid NodeStyle::Box variant and non-empty node id "n1"
When: Constructing DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Box }
Then:
- The variant is successfully created
- The id field equals "n1"
- The style field equals NodeStyle::Box

### Scenario 2: Serialize UpdateNodeStyle to JSON
Given: DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Cloud }
When: Serializing to JSON via serde
Then:
- Output contains "op_type": "update_node_style"
- Output contains "id": "n1"
- Output contains "style": "cloud"

### Scenario 3: Deserialize UpdateNodeStyle from JSON
Given: Valid JSON string `{"op_type":"update_node_style","id":"n1","style":"cylinder"}`
When: Deserializing to DomainOp
Then:
- Result is DomainOp::UpdateNodeStyle
- id field equals "n1"
- style field equals NodeStyle::Cylinder

### Scenario 4: Full serialization roundtrip
Given: Original DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Dashed }
When: Serializing to JSON, then deserializing back to DomainOp
Then:
- The reconstructed DomainOp equals the original
- All fields preserved through roundtrip

### Scenario 5: Kind classification is correct
Given: DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Box }
When: Calling .kind() method
Then:
- Returns OpKind::Node
- This classifies the operation as a node operation

## Test Organization

### Serialization Tests
- test_update_node_style_serialization_box
- test_update_node_style_serialization_cloud
- test_update_node_style_serialization_cylinder
- test_update_node_style_serialization_dashed

### Deserialization Tests
- test_update_node_style_deserialization_box
- test_update_node_style_deserialization_cloud
- test_update_node_style_deserialization_cylinder
- test_update_node_style_deserialization_dashed

### Projection Integration Tests (when wired to apply_operation)
- test_apply_update_node_style_changes_node_style
- test_apply_update_node_style_preserves_other_fields
- test_apply_update_node_style_does_not_affect_other_nodes
