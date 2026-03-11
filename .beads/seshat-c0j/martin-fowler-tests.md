# Martin Fowler Test Plan: NodeResize Projection (seshat-c0j)

## Happy Path Tests

### test_project_node_resize_updates_width_and_height
Given: A DiagramDocument with a node having initial width=50.0, height=30.0
When: project_operation is called with NodeResize { id: "n1", width: 80.0, height: 40.0 }
Then:
- Returns Ok(())
- Node.width == 80.0
- Node.height == 40.0

### test_project_node_resize_preserves_position
Given: A DiagramDocument with a node at position (100.0, 200.0)
When: NodeResize is projected
Then:
- Node.x == 100.0 (unchanged)
- Node.y == 200.0 (unchanged)

### test_project_node_resize_preserves_label
Given: A DiagramDocument with a node having label "Original"
When: NodeResize is projected
Then:
- Node.label == "Original" (unchanged)

### test_project_node_resize_preserves_other_nodes
Given: A DiagramDocument with multiple nodes (n1, n2, n3)
When: NodeResize is projected for n1
Then:
- Node n2 unchanged
- Node n3 unchanged

### test_project_node_resize_increments_revision
Given: A DiagramDocument with revision=5
When: NodeResize is projected successfully
Then:
- Document.revision == 6

### test_project_node_resize_with_various_dimensions
Given: A DiagramDocument with a node
When: NodeResize with various valid dimensions is projected
Then:
- All dimension values are applied exactly

## Error Path Tests

### test_project_node_resize_nonexistent_node_returns_error
Given: A DiagramDocument without node "nonexistent"
When: project_operation is called with NodeResize { id: "nonexistent", width: 80.0, height: 40.0 }
Then:
- Returns Err(ProjectionError::NodeNotFound("nonexistent"))

### test_project_node_resize_nan_width_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with width=f64::NAN
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))

### test_project_node_resize_infinity_width_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with width=f64::INFINITY
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))

### test_project_node_resize_negative_width_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with width=-10.0
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))

### test_project_node_resize_zero_width_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with width=0.0
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))

### test_project_node_resize_nan_height_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with height=f64::NAN
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))

### test_project_node_resize_infinity_height_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with height=f64::INFINITY
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))

### test_project_node_resize_negative_height_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with height=-10.0
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))

### test_project_node_resize_zero_height_returns_error
Given: A DiagramDocument with node "n1"
When: project_operation is called with height=0.0
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))

### test_project_node_resize_with_wrong_operation_type
Given: A DiagramDocument and a non-NodeResize operation
When: project_operation is called
Then: Returns Err(ProjectionError::InvalidOperation(...))

## Edge Case Tests

### test_project_node_resize_very_large_dimensions
Given: A DiagramDocument with a node
When: NodeResize with very large dimensions is projected
Then:
- Returns Ok(())
- Dimensions applied exactly

### test_project_node_resize_very_small_dimensions
Given: A DiagramDocument with a node
When: NodeResize with very small positive dimensions is projected
Then:
- Returns Ok(())
- Dimensions applied exactly

### test_project_node_resize_subnormal_dimensions
Given: A DiagramDocument with a node
When: NodeResize with subnormal dimensions is projected
Then:
- Returns Ok(())
- Dimensions applied exactly

### test_project_node_resize_single_node_document
Given: A DiagramDocument with only one node
When: NodeResize is projected
Then:
- Returns Ok(())
- Only node is updated

### test_project_node_resize_preserves_edges
Given: A DiagramDocument with nodes and edges
When: NodeResize is projected
Then:
- All edges unchanged
- Edge connections unaffected

## Contract Verification Tests

### test_precondition_p1_valid_operation
Given: Valid NodeResize operation
When: project_operation is called
Then: Operation is applied (not rejected)

### test_precondition_p2_node_exists
Given: A DiagramDocument with node "n1"
When: NodeResize for "n1" is projected
Then: Returns Ok(())

### test_precondition_p3_width_valid
Given: A DiagramDocument with node "n1"
When: NodeResize with valid width is projected
Then: Returns Ok(())

### test_precondition_p4_height_valid
Given: A DiagramDocument with node "n1"
When: NodeResize with valid height is projected
Then: Returns Ok(())

### test_postcondition_q1_width_updated
Given: A DiagramDocument with node
When: NodeResize is projected
Then: Node.width == operation.width

### test_postcondition_q2_height_updated
Given: A DiagramDocument with node
When: NodeResize is projected
Then: Node.height == operation.height

### test_postcondition_q3_position_preserved
Given: A DiagramDocument with node at (x, y)
When: NodeResize is projected
Then: Node.x == x AND Node.y == y

### test_postcondition_q4_label_preserved
Given: A DiagramDocument with node having label
When: NodeResize is projected
Then: Node.label unchanged

### test_postcondition_q5_other_nodes_unchanged
Given: A DiagramDocument with multiple nodes
When: NodeResize for one node is projected
Then: All other nodes unchanged

### test_postcondition_q6_revision_incremented
Given: A DiagramDocument with revision N
When: NodeResize is projected
Then: Document.revision == N + 1

### test_invariant_inv1_document_valid_after_projection
Given: A valid DiagramDocument
When: NodeResize is projected
Then: Document remains valid (all nodes have positive finite dimensions)

### test_invariant_inv2_no_nodes_added_or_removed
Given: A DiagramDocument with N nodes
When: NodeResize is projected
Then: Document still has N nodes

### test_invariant_inv3_edges_unaffected
Given: A DiagramDocument with edges
When: NodeResize is projected
Then: All edges unchanged

## Contract Violation Tests

### test_violation_p2_node_not_found_returns_error
Given: Document without "nonexistent" node
When: project_operation(&mut doc, &DomainOp::NodeResize { id: "nonexistent", width: 80.0, height: 40.0 })
Then: returns Err(ProjectionError::NodeNotFound("nonexistent"))

### test_violation_p3_nan_width_returns_invalid_dimensions
Given: Document with node "n1"
When: project_operation(&mut doc, &DomainOp::NodeResize { id: "n1", width: f64::NAN, height: 40.0 })
Then: returns Err(ProjectionError::InvalidDimensions(...))

### test_violation_p3_infinity_width_returns_invalid_dimensions
Given: Document with node "n1"
When: project_operation(&mut doc, &DomainOp::NodeResize { id: "n1", width: f64::INFINITY, height: 40.0 })
Then: returns Err(ProjectionError::InvalidDimensions(...))

### test_violation_p3_negative_width_returns_invalid_dimensions
Given: Document with node "n1"
When: project_operation(&mut doc, &DomainOp::NodeResize { id: "n1", width: -10.0, height: 40.0 })
Then: returns Err(ProjectionError::InvalidDimensions(...))

### test_violation_p4_height_violations
Given: Document with node "n1"
When: project_operation is called with invalid height
Then: returns Err(ProjectionError::InvalidDimensions(...))

### test_violation_q1_width_not_updated
Given: A DiagramDocument with node
When: After projection, check node.width
Then: node.width equals operation.width (not a violation - this is correct)

### test_violation_q2_height_not_updated
Given: A DiagramDocument with node
When: After projection, check node.height
Then: node.height equals operation.height (not a violation)

### test_violation_q3_position_changed
Given: A DiagramDocument with node at (x, y)
When: After NodeResize, check position
Then: Position unchanged (not a violation - this is correct)

### test_violation_q4_label_changed
Given: A DiagramDocument with node having label
When: After NodeResize, check label
Then: Label unchanged (not a violation - this is correct)

### test_violation_q5_other_nodes_affected
Given: A DiagramDocument with multiple nodes
When: After NodeResize for one node, check others
Then: Other nodes unchanged (not a violation)

## Given-When-Then Scenarios

### Scenario 1: Successfully Resize Node
Given: A DiagramDocument containing node "n1" with width=50, height=30
When: project_operation is called with NodeResize { id: "n1", width: 100, height: 60 }
Then:
- Returns Ok(())
- Node "n1" now has width=100
- Node "n1" now has height=60

### Scenario 2: Resize Nonexistent Node Fails
Given: A DiagramDocument without node "ghost"
When: project_operation is called with NodeResize { id: "ghost", width: 100, height: 60 }
Then:
- Returns Err(ProjectionError::NodeNotFound("ghost"))
- No nodes are modified

### Scenario 3: Invalid Dimensions Rejected
Given: A DiagramDocument with node "n1"
When: project_operation is called with NodeResize { id: "n1", width: NaN, height: 60 }
Then:
- Returns Err(ProjectionError::InvalidDimensions(...))
- Node dimensions unchanged

### Scenario 4: Other Properties Preserved
Given: A DiagramDocument with node "n1" at (50, 50) with label "Test"
When: NodeResize { id: "n1", width: 100, height: 60 } is projected
Then:
- Node position remains (50, 50)
- Node label remains "Test"
- Other nodes unchanged
