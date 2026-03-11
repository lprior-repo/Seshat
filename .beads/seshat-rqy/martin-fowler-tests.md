# Martin Fowler Test Plan: UpdateLabel Enum (seshat-rqy)

## Happy Path Tests

### test_parse_update_label_with_valid_json_returns_domain_op
Given: Valid JSON with op_type "update_label", valid id, valid UTF-8 label
When: parse_domain_op is called with the JSON
Then: Returns Ok(DomainOp::UpdateLabel { id, label }) with exact input values

### test_update_label_kind_returns_node
Given: A DomainOp::UpdateLabel instance
When: kind() method is called
Then: Returns OpKind::Node

### test_update_label_serialization_roundtrip_preserves_label
Given: DomainOp::UpdateLabel with id="n1", label="New Label"
When: Serialized to JSON and deserialized back
Then: Returns identical DomainOp with label preserved exactly

### test_domain_op_kind_function_for_update_label
Given: A reference to DomainOp::UpdateLabel
When: domain_op_kind() is called
Then: Returns OpKind::Node

### test_parse_update_label_with_ascii_label
Given: JSON with op_type "update_label" and ASCII label "Hello World"
When: parse_domain_op is called
Then: Returns Ok with label exactly "Hello World"

### test_parse_update_label_with_unicode_label
Given: JSON with op_type "update_label" and Unicode label "Héllo Wörld"
When: parse_domain_op is called
Then: Returns Ok with label exactly "Héllo Wörld"

### test_parse_update_label_with_empty_label
Given: JSON with op_type "update_label" and empty label ""
When: parse_domain_op is called
Then: Returns Ok with label "" (empty string is valid)

### test_parse_update_label_with_emoji
Given: JSON with label containing emoji "Hello 👋 World"
When: parse_domain_op is called
Then: Returns Ok with emoji preserved exactly

## Error Path Tests

### test_parse_update_label_with_typo_in_op_type_returns_unknown_op_type
Given: JSON with op_type "update_lable" (typo)
When: parse_domain_op is called
Then: Returns Err(ContractError::UnknownOpType("update_lable"))

### test_parse_update_label_missing_id_field_returns_missing_field
Given: JSON with update_label but missing "id" field
When: parse_domain_op is called
Then: Returns Err(ContractError::MissingField("id"))

### test_parse_update_label_with_empty_id_returns_error
Given: JSON with id ""
When: parse_domain_op is called
Then: Returns Err(ContractError::InvalidPayload(...)) (empty ID)

### test_parse_update_label_missing_label_field_returns_missing_field
Given: JSON with update_label but missing "label" field
When: parse_domain_op is called
Then: Returns Err(ContractError::MissingField("label"))

### test_parse_update_label_with_invalid_json_returns_invalid_json_error
Given: Invalid JSON string
When: parse_domain_op is called
Then: Returns Err(ContractError::InvalidJson(...))

## Edge Case Tests

### test_update_label_with_very_long_label
Given: JSON with very long label (10KB+)
When: parse_domain_op and roundtrip
Then: Label preserved exactly

### test_update_label_with_rtl_text
Given: JSON with RTL text "مرحبا بالعالم"
When: parse_domain_op is called
Then: Returns Ok with RTL text exactly preserved

### test_update_label_with_mixed_direction_text
Given: JSON with mixed LTR/RTL "Hello مرحبا World"
When: parse_domain_op is called
Then: Label exactly preserved

### test_update_label_with_special_characters
Given: JSON with special chars "test <>&\"' chars"
When: parse_domain_op is called
Then: Label exactly preserved after roundtrip

### test_update_label_with_newlines_and_tabs
Given: JSON with "line1\nline2\ttab"
When: parse_domain_op is called
Then: Returns Ok with whitespace exactly preserved

### test_update_label_json_missing_op_field
Given: JSON without "op" field
When: parse_domain_op is called
Then: Returns Err(ContractError::MissingField("op"))

## Contract Verification Tests

### test_precondition_p1_valid_json
Given: Valid JSON input
When: parse_domain_op is called
Then: JSON parsed successfully

### test_precondition_p2_op_type_recognition
Given: JSON with correct op_type "update_label"
When: parse_domain_op is called
Then: Operation is recognized

### test_precondition_p3_id_field_present_and_valid
Given: JSON with non-empty id field
When: parse_domain_op is called
Then: Id field parsed correctly

### test_precondition_p4_label_valid_utf8
Given: JSON with label field
When: parse_domain_op is called
Then: Label is valid UTF-8 (String guarantees this in Rust)

### test_postcondition_q1_exact_values_preserved
Given: Input with specific id and label
When: Parsed and reconstructed
Then: All values match exactly

### test_postcondition_q2_kind_returns_node
Given: DomainOp::UpdateLabel instance
When: kind() is called
Then: Returns OpKind::Node

### test_postcondition_q3_serialization_roundtrip
Given: Serialized DomainOp::UpdateLabel
When: Deserialized back
Then: Produces identical DomainOp

### test_postcondition_q4_exhaustive_match
Given: All DomainOp variants
When: match statement compiles
Then: UpdateLabel handled in kind() method

### test_postcondition_q5_parse_recognizes_type
Given: JSON with "update_label" op type
When: Parsed
Then: Returns UpdateLabel variant

### test_invariant_inv1_exhaustive_enum
Given: New variant added to DomainOp
When: Existing match statements compiled
Then: Compiler warns if not exhaustive

### test_invariant_inv2_trait_derivations
Given: DomainOp::UpdateLabel
When: Clone, Debug, PartialEq, Serialize, Deserialize used
Then: All traits work correctly

### test_invariant_inv3_opkind_classification
Given: DomainOp::UpdateLabel
When: kind() is called
Then: Returns OpKind::Node

## Contract Violation Tests

### test_violation_p2_typo_op_type_returns_unknown_op_type
Given: JSON with op "update_lable"
When: parse_domain_op is called
Then: returns Err(ContractError::UnknownOpType("update_lable"))

### test_violation_p3_missing_id_returns_missing_field
Given: JSON with op "update_label" but no id field
When: parse_domain_op is called
Then: returns Err(ContractError::MissingField("id"))

### test_violation_p3_empty_id_returns_invalid_payload
Given: JSON with id: ""
When: parse_domain_op is called
Then: returns Err(ContractError::InvalidPayload(...))

### test_violation_q1_label_not_preserved
Given: DomainOp::UpdateLabel with specific label
When: Serialized and deserialized
Then: Label matches exactly (not a violation - correct behavior)

### test_violation_q2_kind_not_returning_node
Given: DomainOp::UpdateLabel
When: kind() is called
Then: Returns OpKind::Node (not Edge or Composite)

## Given-When-Then Scenarios

### Scenario 1: Successful Label Update
Given: Valid JSON {"op": "update_label", "id": "node1", "label": "New Label"}
When: parse_domain_op is called
Then:
- Returns Ok(DomainOp::UpdateLabel)
- The id field equals "node1"
- The label field equals "New Label"

### Scenario 2: Unicode Label Preservation
Given: JSON with label containing Chinese characters "你好世界"
When: parse_domain_op and roundtrip
Then:
- Label exactly equals "你好世界"
- All characters preserved

### Scenario 3: Empty Label (Clear Label)
Given: JSON with label ""
When: parse_domain_op is called
Then:
- Returns Ok(DomainOp::UpdateLabel)
- Label is empty string (valid use case)

### Scenario 4: Invalid ID Rejected
Given: JSON with empty id ""
When: parse_domain_op is called
Then:
- Returns Err(ContractError::InvalidPayload(...))
- No DomainOp created
