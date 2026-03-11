# Martin Fowler Test Plan: UpdateLabel Tests (seshat-pfc)

## Happy Path Tests

### test_parsing_valid_update_label_json_returns_domain_op
Given: Valid JSON with op_type "update_label", valid id, valid UTF-8 label
When: parse_domain_op is called with the JSON
Then: Returns Ok(DomainOp::UpdateLabel { id, label })

### test_update_label_encoding_roundtrip_preserves_all_fields
Given: DomainOp::UpdateLabel with id="n1", label="New Label"
When: Serialized to JSON and deserialized back
Then: Returns identical DomainOp with label preserved

### test_update_label_projection_applies_label_correctly
Given: A DiagramDocument with node "n1" having label "Original"
When: project_operation is called with UpdateLabel { id: "n1", label: "New" }
Then:
- Returns Ok(())
- Node.label == "New"

### test_update_label_kind_method_returns_node
Given: DomainOp::UpdateLabel instance
When: kind() is called
Then: Returns OpKind::Node

### test_update_label_serialization_with_various_labels
Given: DomainOp::UpdateLabel with various valid label values
When: Serialized and deserialized
Then: All values preserved exactly

### test_update_label_projection_preserves_other_properties
Given: A DiagramDocument with node at (x, y) with dimensions (w, h)
When: UpdateLabel is projected
Then:
- Node.x == x
- Node.y == y
- Node.width == w
- Node.height == h

## Error Path Tests

### test_parsing_invalid_op_type_returns_error
Given: JSON with typo in op_type "update_lable"
When: parse_domain_op is called
Then: Returns Err(ContractError::UnknownOpType(...))

### test_parsing_missing_id_returns_error
Given: JSON with update_label but missing id field
When: parse_domain_op is called
Then: Returns Err(ContractError::MissingField("id"))

### test_parsing_empty_id_returns_error
Given: JSON with id ""
When: parse_domain_op is called
Then: Returns Err(ContractError::InvalidPayload(...))

### test_parsing_missing_label_returns_error
Given: JSON with update_label but missing label field
When: parse_domain_op is called
Then: Returns Err(ContractError::MissingField("label"))

### test_projection_nonexistent_node_returns_error
Given: A DiagramDocument without node "ghost"
When: project_operation is called with UpdateLabel for "ghost"
Then: Returns Err(ProjectionError::TargetNotFound("ghost"))

### test_projection_wrong_operation_type_returns_error
Given: A DiagramDocument and wrong operation type
When: project_operation is called
Then: Returns Err(ProjectionError::InvalidOperation(...))

## Edge Case Tests

### test_update_label_with_unicode_preserved
Given: JSON with Unicode label "Héllo Wörld 日本語"
When: parse_domain_op and roundtrip
Then: Label exactly preserved

### test_update_label_with_rtl_text_preserved
Given: JSON with RTL text "مرحبا بالعالم"
When: parse_domain_op and roundtrip
Then: RTL text exactly preserved

### test_update_label_with_empty_string_clears_label
Given: JSON with empty label ""
When: parse_domain_op is called
Then: Returns Ok with empty label (valid)

### test_update_label_projection_empty_string_clears
Given: DiagramDocument with node having label "Text"
When: UpdateLabel with "" is projected
Then:
- Returns Ok(())
- Node.label == ""

### test_update_label_with_emoji_preserved
Given: JSON with emoji "Hello 👋🌍🎉"
When: parse_domain_op and roundtrip
Then: Emoji exactly preserved

### test_update_label_with_special_characters
Given: JSON with special chars "<>&\"'\\n\\t"
When: parse_domain_op and roundtrip
Then: All characters preserved

### test_empty_document_with_update_label
Given: An empty DiagramDocument
When: UpdateLabel is projected for nonexistent node
Then: Returns Err(ProjectionError::TargetNotFound(...))

### test_multiple_nodes_single_label_update
Given: A DiagramDocument with multiple nodes
When: One node's label is updated
Then: Only target node affected, others unchanged

### test_update_label_very_long_text
Given: A DiagramDocument with a node
When: UpdateLabel with very long text is projected
Then:
- Returns Ok(())
- Label applied exactly

## Contract Verification Tests

### test_contract_precondition_p1_valid_operation
Given: Valid UpdateLabel operation
When: Used in parse or projection
Then: Operation succeeds

### test_contract_precondition_p2_node_exists
Given: DiagramDocument with target node
When: UpdateLabel projected
Then: Projection succeeds

### test_contract_precondition_p3_label_valid_utf8
Given: Valid UTF-8 label
When: UpdateLabel parsed or projected
Then: Succeeds

### test_contract_postcondition_q1_roundtrip_preserves_label
Given: DomainOp::UpdateLabel
When: Roundtrip serialization
Then: Label matches exactly

### test_contract_postcondition_q2_projection_updates_label
Given: DiagramDocument with node
When: UpdateLabel projected
Then: Node.label updated

### test_contract_postcondition_q3_position_preserved
Given: DiagramDocument with node at (x, y)
When: UpdateLabel projected
Then: Node position unchanged

### test_contract_postcondition_q4_dimensions_preserved
Given: DiagramDocument with node having dimensions
When: UpdateLabel projected
Then: Node dimensions unchanged

### test_contract_postcondition_q5_empty_label_valid
Given: Empty string label
When: UpdateLabel parsed/projected
Then: Succeeds (clears label)

### test_contract_invariant_inv1_tests_areolated
Given: Multiple test executions
When: Tests run in any order
Then: Each test is independent

### test_contract_invariant_inv2_coverage_includes_happy_and_error
Given: Test suite for UpdateLabel
When: Analyzing coverage
Then: Both happy path and error paths covered

### test_contract_invariant_inv3_unicode_rtl_preserved
Given: Unicode and RTL labels
When: Roundtrip or projection
Then: Characters exactly preserved

## Contract Violation Tests

### test_violation_roundtrip_label_preservation
Given: DomainOp::UpdateLabel { id: "n1", label: "Test" }
When: Serialized and deserialized
Then: Label exactly "Test" (not a violation - correct behavior)

### test_violation_projection_label_application
Given: DiagramDocument with node
When: UpdateLabel projected
Then: Label applied exactly (not a violation - correct behavior)

### test_violation_error_cases_return_correct_errors
Given: Various invalid inputs
When: Parsing or projecting
Then: Returns appropriate error variants

### test_violation_unicode_not_preserved
Given: DomainOp::UpdateLabel with Unicode
When: Roundtrip
Then: Unicode preserved (not a violation)

## Given-When-Then Scenarios

### Scenario 1: Complete UpdateLabel Workflow
Given: Valid JSON {"op": "update_label", "id": "n1", "label": "New Label"}
When: 
1. parse_domain_op is called
2. Result is projected onto DiagramDocument with node "n1"
Then:
- Parse returns Ok(DomainOp::UpdateLabel)
- Projection returns Ok(())
- Node label updated to "New Label"

### Scenario 2: Unicode Label Workflow
Given: JSON with Unicode label "中文日本語العربية"
When: 
1. parse_domain_op is called
2. Projection applied
Then:
- Label exactly preserved through roundtrip
- Label exactly applied in projection

### Scenario 3: Clear Label Workflow
Given: DiagramDocument with node having label "Some Text"
When: UpdateLabel with "" is projected
Then:
- Returns Ok(())
- Label is now empty string
- Other properties unchanged

### Scenario 4: Error Handling Workflow
Given: Invalid JSON {"op": "update_label", "id": "", "label": "New"}
When: parse_domain_op is called
Then:
- Returns Err(ContractError::InvalidPayload(...))
- No DomainOp created

### Scenario 5: Multiple Nodes Independence
Given: DiagramDocument with nodes n1, n2, n3
When: UpdateLabel for n1 is projected
Then:
- n1 label changed
- n2, n3 completely unchanged (position, dimensions, labels)
- Document revision incremented
