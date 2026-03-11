# Martin Fowler Test Plan

## Happy Path Tests
- test_update_edge_style_variant_constructable_with_valid_fields
- test_update_edge_style_serializes_to_correct_json
- test_update_edge_style_deserializes_from_valid_json
- test_update_edge_style_kind_returns_edge
- test_update_edge_style_clone_produces_equivalent_copy

## Error Path Tests
- test_update_edge_style_deserialization_fails_with_unknown_op_type
- test_update_edge_style_deserialization_fails_with_missing_required_fields
- test_update_edge_style_deserialization_fails_with_invalid_style_value

## Edge Case Tests
- test_update_edge_style_all_three_style_variants
- test_update_edge_style_with_various_valid_edge_ids
- test_update_edge_style_serialization_preserves_all_fields

## Contract Verification Tests
- test_precondition_valid_edgestyle_compile_time_enforced
- test_precondition_valid_edgeid_runtime_check
- test_postcondition_variant_exists_and_constructable
- test_postcondition_serialization_format_correct
- test_postcondition_deserialization_format_correct
- test_postcondition_kind_classification_correct
- test_invariant_domainop_exhaustiveness
- test_invariant_serialization_roundtrip

## Contract Violation Tests
- `test_precondition_p1_invalid_style_value_returns_compile_error`
  Given: Attempting to construct DomainOp::UpdateEdgeStyle with invalid style value
  When: style = "invalid_style" (not an EdgeStyle variant)
  Then: Compile-time error - Rust enum prevents invalid variants

- `test_precondition_p2_empty_edgeid_returns_error`
  Given: DomainOp::UpdateEdgeStyle with empty id field
  When: id = ""
  Then: Returns `Err(Error::InvalidInput)` or precondition check fails

- `test_postcondition_q3_unknown_op_type_deserialization_fails`
  Given: JSON with unknown op_type field
  When: JSON `{"op_type": "update_edge_style_unknown", "id": "e1", "style": "solid"}`
  Then: Returns `Err(serde_json::Error)` - unknown variant

- `test_postcondition_q4_kind_returns_wrong_opkind_fails`
  Given: DomainOp::UpdateEdgeStyle instance
  When: Calling .kind() method
  Then: Returns OpKind::Edge (not Node, ZOrder, or Composite)

## Given-When-Then Scenarios

### Scenario 1: Construct UpdateEdgeStyle with Solid style
Given: A valid EdgeStyle::Solid variant and non-empty edge id "e1"
When: Constructing DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Solid }
Then:
- The variant is successfully created
- The id field equals "e1"
- The style field equals EdgeStyle::Solid

### Scenario 2: Serialize UpdateEdgeStyle to JSON
Given: DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed }
When: Serializing to JSON via serde
Then:
- Output contains "op_type": "update_edge_style"
- Output contains "id": "e1"
- Output contains "style": "dashed"

### Scenario 3: Deserialize UpdateEdgeStyle from JSON
Given: Valid JSON string `{"op_type":"update_edge_style","id":"e1","style":"dotted"}`
When: Deserializing to DomainOp
Then:
- Result is DomainOp::UpdateEdgeStyle
- id field equals "e1"
- style field equals EdgeStyle::Dotted

### Scenario 4: Full serialization roundtrip
Given: Original DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Solid }
When: Serializing to JSON, then deserializing back to DomainOp
Then:
- The reconstructed DomainOp equals the original
- All fields preserved through roundtrip

### Scenario 5: Kind classification is correct
Given: DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed }
When: Calling .kind() method
Then:
- Returns OpKind::Edge
- This classifies the operation as an edge operation

## Test Organization

### Serialization Tests
- test_update_edge_style_serialization_solid
- test_update_edge_style_serialization_dashed
- test_update_edge_style_serialization_dotted

### Deserialization Tests
- test_update_edge_style_deserialization_solid
- test_update_edge_style_deserialization_dashed
- test_update_edge_style_deserialization_dotted

### Projection Integration Tests (when wired to apply_operation)
- test_apply_update_edge_style_changes_edge_style
- test_apply_update_edge_style_preserves_other_fields
- test_apply_update_edge_style_does_not_affect_other_edges
