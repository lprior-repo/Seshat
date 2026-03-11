# Martin Fowler Test Plan: NodeResize Tests (seshat-nf1)

## Happy Path Tests

### test_parsing_valid_node_resize_json_returns_domain_op
Given: Valid JSON with op_type "node_resize", valid id, positive finite width and height
When: parse_domain_op is called with the JSON
Then: Returns Ok(DomainOp::NodeResize { id, width, height })

### test_node_resize_encoding_roundtrip_preserves_all_fields
Given: DomainOp::NodeResize with id="n1", width=80.0, height=40.0
When: Serialized to JSON and deserialized back
Then: Returns identical DomainOp with all fields preserved

### test_node_resize_projection_applies_dimensions_correctly
Given: A DiagramDocument with node "n1" having width=50.0, height=30.0
When: project_operation is called with NodeResize { id: "n1", width: 100.0, height: 60.0 }
Then:
- Returns Ok(())
- Node.width == 100.0
- Node.height == 60.0

### test_node_resize_kind_method_returns_node
Given: DomainOp::NodeResize instance
When: kind() is called
Then: Returns OpKind::Node

### test_node_resize_serialization_with_various_dimensions
Given: DomainOp::NodeResize with various valid dimension values
When: Serialized and deserialized
Then: All values preserved exactly

## Error Path Tests

### test_parsing_invalid_op_type_returns_error
Given: JSON with typo in op_type "node_rezise"
When: parse_domain_op is called
Then: Returns Err(ContractError::UnknownOpType(...))

### test_parsing_missing_id_returns_error
Given: JSON with node_resize but missing id field
When: parse_domain_op is called
Then: Returns Err(ContractError::MissingField("id"))

### test_parsing_invalid_width_returns_error
Given: JSON with width = -10.0
When: parse_domain_op is called
Then: Returns Err(ContractError::InvalidPayload(...))

### test_parsing_zero_width_returns_error
Given: JSON with width = 0.0
When: parse_domain_op is called
Then: Returns Err(ContractError::InvalidPayload(...))

### test_parsing_nan_width_returns_error
Given: JSON with width = NaN
When: parse_domain_op is called
Then: Returns Err(ContractError::InvalidPayload(...))

### test_parsing_invalid_height_returns_error
Given: JSON with height = Infinity
When: parse_domain_op is called
Then: Returns Err(ContractError::InvalidPayload(...))

### test_projection_nonexistent_node_returns_error
Given: A DiagramDocument without node "ghost"
When: project_operation is called with NodeResize for "ghost"
Then: Returns Err(ProjectionError::NodeNotFound("ghost"))

### test_projection_invalid_dimensions_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with width=NaN
Then: Returns Err(ProjectionError::InvalidDimensions(...))

## Edge Case Tests

### test_empty_document_with_node_resize
Given: An empty DiagramDocument
When: NodeResize is projected for nonexistent node
Then: Returns Err(ProjectionError::NodeNotFound(...))

### test_multiple_nodes_single_resize
Given: A DiagramDocument with multiple nodes
When: One node is resized
Then: Only target node affected, others unchanged

### test_roundtrip_very_large_dimensions
Given: DomainOp::NodeResize with very large dimensions
When: Serialized and deserialized
Then: Values preserved exactly

### test_roundtrip_very_small_dimensions
Given: DomainOp::NodeResize with very small positive dimensions
When: Serialized and deserialized
Then: Values preserved exactly

### test_projection_preserves_unrelated_node_properties
Given: A DiagramDocument with multiple nodes having various properties
When: One node is resized
Then: All other node properties (position, label, etc.) unchanged

## Contract Verification Tests

### test_contract_precondition_p1_valid_operation
Given: Valid NodeResize operation
When: Used in parse or projection
Then: Operation succeeds

### test_contract_precondition_p2_node_exists
Given: DiagramDocument with target node
When: NodeResize projected
Then: Projection succeeds

### test_contract_precondition_p3_width_valid
Given: Valid width value (> 0, finite)
When: NodeResize parsed or projected
Then: Succeeds

### test_contract_postcondition_q1_roundtrip_preserves_fields
Given: DomainOp::NodeResize
When: Roundtrip serialization
Then: All fields match exactly

### test_contract_postcondition_q2_projection_updates_dimensions
Given: DiagramDocument with node
When: NodeResize projected
Then: Node dimensions updated

### test_contract_postcondition_q3_position_preserved
Given: DiagramDocument with node at (x, y)
When: NodeResize projected
Then: Node position unchanged

### test_contract_invariant_inv1_tests_areolated
Given: Multiple test executions
When: Tests run in any order
Then: Each test is independent, no shared state

### test_contract_invariant_inv2_coverage_includes_happy_and_error
Given: Test suite for NodeResize
When: Analyzing coverage
Then: Both happy path and error paths covered

## Contract Violation Tests

### test_violation_roundtrip_field_preservation
Given: DomainOp::NodeResize { id: "n1", width: 80.0, height: 40.0 }
When: Serialized and deserialized
Then: All fields exactly preserved (not a violation - correct behavior)

### test_violation_projection_dimension_application
Given: DiagramDocument with node
When: NodeResize projected
Then: Dimensions applied exactly (not a violation - correct behavior)

### test_violation_error_cases_return_correct_errors
Given: Various invalid inputs
When: Parsing or projecting
Then: Returns appropriate error variants (not violations)

## Given-When-Then Scenarios

### Scenario 1: Complete NodeResize Workflow
Given: Valid JSON {"op": "node_resize", "id": "n1", "width": 80.0, "height": 40.0}
When: 
1. parse_domain_op is called
2. Result is projected onto DiagramDocument with node "n1"
Then:
- Parse returns Ok(DomainOp::NodeResize)
- Projection returns Ok(())
- Node dimensions updated to 80.0 x 40.0

### Scenario 2: Error Handling Workflow
Given: Invalid JSON {"op": "node_resize", "id": "n1", "width": -10.0, "height": 40.0}
When: parse_domain_op is called
Then:
- Returns Err(ContractError::InvalidPayload(...))
- No DomainOp created

### Scenario 3: Multiple Nodes Independence
Given: DiagramDocument with nodes n1, n2, n3
When: NodeResize for n1 is projected
Then:
- n1 dimensions changed
- n2, n3 completely unchanged
- Document revision incremented

### Scenario 4: Serialization Fidelity
Given: DomainOp::NodeResize { id: "test", width: 123.456, height: 789.012 }
When: Serialized to JSON, stored, loaded, deserialized
Then:
- id == "test"
- width == 123.456
- height == 789.012
- Exactly identical to original
