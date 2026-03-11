# Martin Fowler Test Plan: NodeResize Enum (seshat-fir)

## Happy Path Tests

### test_parse_node_resize_with_valid_json_returns_domain_op
Given: Valid JSON with op_type "node_resize", valid id, positive finite width and height
When: `parse_domain_op` is called with the JSON
Then: Returns `Ok(DomainOp::NodeResize { id, width, height })` with exact input values

### test_node_resize_kind_returns_node
Given: A `DomainOp::NodeResize` instance
When: `kind()` method is called
Then: Returns `OpKind::Node`

### test_node_resize_serialization_roundtrip_preserves_fields
Given: A `DomainOp::NodeResize` with id="n1", width=80.0, height=40.0
When: Serialized to JSON and deserialized back
Then: Returns identical DomainOp with all fields preserved exactly

### test_domain_op_kind_function_for_node_resize
Given: A reference to `DomainOp::NodeResize`
When: `domain_op_kind()` is called
Then: Returns `OpKind::Node`

### test_parse_node_resize_with_various_valid_dimensions
Given: JSON with node_resize operation with various valid dimension values (small, large, decimal)
When: Parsing the JSON
Then: All dimension values are preserved exactly in the result

## Error Path Tests

### test_parse_node_resize_with_typo_in_op_type_returns_unknown_op_type
Given: JSON with op_type "node_rezise" (typo)
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::UnknownOpType("node_rezise"))`

### test_parse_node_resize_missing_id_field_returns_missing_field
Given: JSON with node_resize but missing "id" field
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::MissingField("id"))`

### test_parse_node_resize_with_negative_width_returns_invalid_payload
Given: JSON with width = -10.0
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_parse_node_resize_with_zero_width_returns_invalid_payload
Given: JSON with width = 0.0
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_parse_node_resize_with_nan_width_returns_invalid_payload
Given: JSON with width = NaN
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_parse_node_resize_with_infinity_width_returns_invalid_payload
Given: JSON with width = Infinity
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_parse_node_resize_with_negative_height_returns_invalid_payload
Given: JSON with height = -10.0
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_parse_node_resize_with_zero_height_returns_invalid_payload
Given: JSON with height = 0.0
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_parse_node_resize_with_nan_height_returns_invalid_payload
Given: JSON with height = NaN
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_parse_node_resize_with_infinity_height_returns_invalid_payload
Given: JSON with height = Infinity
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_parse_node_resize_with_invalid_json_returns_invalid_json_error
Given: Invalid JSON string
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidJson(...))`

### test_parse_node_resize_with_empty_id_returns_invalid_payload
Given: JSON with empty string id ""
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

## Edge Case Tests

### test_node_resize_with_very_large_dimensions
Given: JSON with very large width and height (f64::MAX / 2)
When: Parsing and roundtrip
Then: Values preserved exactly

### test_node_resize_with_very_small_positive_dimensions
Given: JSON with very small positive width and height (f64::MIN_POSITIVE)
When: Parsing and roundtrip
Then: Values preserved exactly

### test_node_resize_with_subnormal_dimensions
Given: JSON with subnormal width and height
When: Parsing and roundtrip
Then: Values preserved exactly

### test_node_resize_json_missing_width_field
Given: JSON with node_resize but missing "width" field
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::MissingField("width"))`

### test_node_resize_json_missing_height_field
Given: JSON with node_resize but missing "height" field
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::MissingField("height"))`

## Contract Verification Tests

### test_precondition_p1_valid_json
Given: Invalid JSON input
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidJson(...))`

### test_precondition_p2_op_type_recognition
Given: JSON with correct op_type "node_resize"
When: `parse_domain_op` is called
Then: Operation is recognized and parsed

### test_precondition_p3_id_field_present
Given: JSON with valid id field
When: `parse_domain_op` is called
Then: Id field is parsed correctly

### test_precondition_p4_width_validation
Given: JSON with invalid width values
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_precondition_p5_height_validation
Given: JSON with invalid height values
When: `parse_domain_op` is called
Then: Returns `Err(ContractError::InvalidPayload(...))`

### test_postcondition_q1_exact_values_preserved
Given: Input with specific id, width, height
When: Parsed and reconstructed
Then: All values match exactly

### test_postcondition_q2_kind_returns_node
Given: DomainOp::NodeResize instance
When: kind() is called
Then: Returns OpKind::Node

### test_postcondition_q3_serialization_roundtrip
Given: Serialized DomainOp::NodeResize
When: Deserialized back
Then: Produces identical DomainOp

### test_postcondition_q4_exhaustive_match
Given: All DomainOp variants
When: match statement covers all variants
Then: NodeResize is handled in kind() method

### test_postcondition_q5_parse_recognizes_type
Given: JSON with "node_resize" op type
When: Parsed
Then: Returns NodeResize variant

### test_invariant_inv1_exhaustive_enum
Given: New variant added to DomainOp
When: Existing match statements are compiled
Then: Compiler warns if match is not exhaustive

### test_invariant_inv2_trait_derivations
Given: DomainOp::NodeResize
When: Clone, Debug, PartialEq, Serialize, Deserialize are used
Then: All traits work correctly

### test_invariant_inv3_opkind_classification
Given: DomainOp::NodeResize
When: kind() is called
Then: Returns OpKind::Node

## Contract Violation Tests

### test_violation_p2_typo_op_type_returns_unknown_op_type
Given: JSON with op "node_rezise"
When: parse_domain_op is called
Then: returns Err(ContractError::UnknownOpType("node_rezise"))

### test_violation_p3_missing_id_returns_missing_field
Given: JSON with op "node_resize" but no id field
When: parse_domain_op is called
Then: returns Err(ContractError::MissingField("id"))

### test_violation_p4_negative_width_returns_invalid_payload
Given: JSON with width: -10.0
When: parse_domain_op is called
Then: returns Err(ContractError::InvalidPayload(...))

### test_violation_p4_zero_width_returns_invalid_payload
Given: JSON with width: 0.0
When: parse_domain_op is called
Then: returns Err(ContractError::InvalidPayload(...))

### test_violation_p4_nan_width_returns_invalid_payload
Given: JSON with width: NaN
When: parse_domain_op is called
Then: returns Err(ContractError::InvalidPayload(...))

### test_violation_p5_height_validation_violations
Given: JSON with invalid height values
When: parse_domain_op is called
Then: returns Err(ContractError::InvalidPayload(...))

### test_violation_q1_serialization_not_preserving_values
Given: DomainOp::NodeResize with specific values
When: Serialized and deserialized
Then: Values match exactly (not a violation - this is correct behavior)

### test_violation_q2_kind_not_returning_node
Given: DomainOp::NodeResize
When: kind() is called
Then: Returns OpKind::Node (not Edge or Composite)

## Given-When-Then Scenarios

### Scenario 1: Successful Node Resize Operation
Given: A valid JSON document with op_type "node_resize", id "node1", width 100.0, height 50.0
When: parse_domain_op is called
Then:
- Returns Ok(DomainOp::NodeResize)
- The id field equals "node1"
- The width field equals 100.0
- The height field equals 50.0

### Scenario 2: Invalid Dimension Rejected
Given: A JSON document with width set to -5.0
When: parse_domain_op is called
Then:
- Returns Err(ContractError::InvalidPayload(...))
- The error message mentions "width"
- No DomainOp is created

### Scenario 3: Missing Required Field Rejected
Given: A JSON document missing the "height" field
When: parse_domain_op is called
Then:
- Returns Err(ContractError::MissingField("height"))
- No DomainOp is created

### Scenario 4: Roundtrip Preservation
Given: DomainOp::NodeResize with id "test", width 80.0, height 40.0
When: The operation is serialized to JSON and then deserialized
Then:
- The deserialized operation has id "test"
- The deserialized operation has width 80.0
- The deserialized operation has height 40.0
