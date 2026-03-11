# Martin Fowler Test Plan: UpdateLabel Projection (seshat-8tj)

## Happy Path Tests

### test_project_update_label_updates_node_label
Given: A DiagramDocument with a node having label "Original"
When: project_operation is called with UpdateLabel { id: "n1", label: "New Label" }
Then:
- Returns Ok(())
- Node.label == "New Label"

### test_project_update_label_preserves_position
Given: A DiagramDocument with a node at position (100.0, 200.0)
When: UpdateLabel is projected
Then:
- Node.x == 100.0 (unchanged)
- Node.y == 200.0 (unchanged)

### test_project_update_label_preserves_dimensions
Given: A DiagramDocument with a node with width=80.0, height=40.0
When: UpdateLabel is projected
Then:
- Node.width == 80.0 (unchanged)
- Node.height == 40.0 (unchanged)

### test_project_update_label_preserves_other_nodes
Given: A DiagramDocument with multiple nodes (n1, n2, n3)
When: UpdateLabel is projected for n1
Then:
- Node n2 unchanged
- Node n3 unchanged

### test_project_update_label_increments_revision
Given: A DiagramDocument with revision=5
When: UpdateLabel is projected successfully
Then:
- Document.revision == 6

### test_project_update_label_with_empty_string_clears_label
Given: A DiagramDocument with a node having label "Some Text"
When: UpdateLabel with label "" is projected
Then:
- Returns Ok(())
- Node.label == "" (cleared)

### test_project_update_label_with_unicode_text
Given: A DiagramDocument with a node
When: UpdateLabel with Unicode label "Héllo 世界" is projected
Then:
- Returns Ok(())
- Node.label == "Héllo 世界"

### test_project_update_label_with_rtl_text
Given: A DiagramDocument with a node
When: UpdateLabel with RTL label "مرحبا" is projected
Then:
- Returns Ok(())
- Node.label == "مرحبا"

## Error Path Tests

### test_project_update_label_nonexistent_node_returns_error
Given: A DiagramDocument without node "nonexistent"
When: project_operation is called with UpdateLabel { id: "nonexistent", label: "New" }
Then:
- Returns Err(ProjectionError::TargetNotFound("nonexistent"))

### test_project_update_label_with_wrong_operation_type
Given: A DiagramDocument and a non-UpdateLabel operation
When: project_operation is called
Then: Returns Err(ProjectionError::InvalidOperation(...))

## Edge Case Tests

### test_project_update_label_single_node_document
Given: A DiagramDocument with only one node
When: UpdateLabel is projected
Then:
- Returns Ok(())
- Only node's label updated

### test_project_update_label_preserves_edges
Given: A DiagramDocument with nodes and edges
When: UpdateLabel is projected
Then:
- All edges unchanged
- Edge connections unaffected

### test_project_update_label_very_long_label
Given: A DiagramDocument with a node
When: UpdateLabel with very long label is projected
Then:
- Returns Ok(())
- Label applied exactly

### test_project_update_label_emoji_preserved
Given: A DiagramDocument with a node
When: UpdateLabel with emoji "Hello 👋🌍" is projected
Then:
- Returns Ok(())
- Emoji exactly preserved

### test_project_update_label_mixed_content
Given: A DiagramDocument with a node
When: UpdateLabel with mixed content "Item #1: ⚡ Power" is projected
Then:
- Returns Ok(())
- All content exactly preserved

## Contract Verification Tests

### test_precondition_p1_valid_operation
Given: Valid UpdateLabel operation
When: project_operation is called
Then: Operation is applied

### test_precondition_p2_target_exists
Given: A DiagramDocument with node "n1"
When: UpdateLabel for "n1" is projected
Then: Returns Ok(())

### test_precondition_p3_label_valid_utf8
Given: Valid UTF-8 label
When: UpdateLabel projected
Then: Succeeds (String guarantees UTF-8)

### test_postcondition_q1_label_updated
Given: A DiagramDocument with node
When: UpdateLabel is projected
Then: Node.label == operation.label

### test_postcondition_q2_position_preserved
Given: A DiagramDocument with node at (x, y)
When: UpdateLabel is projected
Then: Node.x == x AND Node.y == y

### test_postcondition_q3_dimensions_preserved
Given: A DiagramDocument with node having width and height
When: UpdateLabel is projected
Then: Node.width and Node.height unchanged

### test_postcondition_q4_other_nodes_unchanged
Given: A DiagramDocument with multiple nodes
When: UpdateLabel for one node is projected
Then: All other nodes unchanged

### test_postcondition_q5_revision_incremented
Given: A DiagramDocument with revision N
When: UpdateLabel is projected
Then: Document.revision == N + 1

### test_invariant_inv1_document_valid_after_projection
Given: A valid DiagramDocument
When: UpdateLabel is projected
Then: Document remains valid

### test_invariant_inv2_no_nodes_added_or_removed
Given: A DiagramDocument with N nodes
When: UpdateLabel is projected
Then: Document still has N nodes

### test_invariant_inv3_edges_unaffected
Given: A DiagramDocument with edges
When: UpdateLabel is projected
Then: All edges unchanged

## Contract Violation Tests

### test_violation_p2_target_not_found_returns_error
Given: Document without "nonexistent" node
When: project_operation(&mut doc, &DomainOp::UpdateLabel { id: "nonexistent", label: "New".into() })
Then: returns Err(ProjectionError::TargetNotFound("nonexistent"))

### test_violation_q1_label_not_updated
Given: A DiagramDocument with node
When: After projection, check node.label
Then: node.label equals operation.label (not a violation - correct behavior)

### test_violation_q2_position_changed
Given: A DiagramDocument with node at (x, y)
When: After UpdateLabel, check position
Then: Position unchanged (not a violation - correct behavior)

### test_violation_q3_dimensions_changed
Given: A DiagramDocument with node having width/height
When: After UpdateLabel, check dimensions
Then: Dimensions unchanged (not a violation - correct behavior)

### test_violation_q4_other_nodes_affected
Given: A DiagramDocument with multiple nodes
When: After UpdateLabel for one node, check others
Then: Other nodes unchanged (not a violation)

## Given-When-Then Scenarios

### Scenario 1: Successfully Update Label
Given: A DiagramDocument containing node "n1" with label "Old"
When: project_operation is called with UpdateLabel { id: "n1", label: "New Label" }
Then:
- Returns Ok(())
- Node "n1" now has label "New Label"

### Scenario 2: Update Nonexistent Node Fails
Given: A DiagramDocument without node "ghost"
When: project_operation is called with UpdateLabel { id: "ghost", label: "New" }
Then:
- Returns Err(ProjectionError::TargetNotFound("ghost"))
- No nodes are modified

### Scenario 3: Clear Label with Empty String
Given: A DiagramDocument with node "n1" having label "Some Text"
When: UpdateLabel { id: "n1", label: "" } is projected
Then:
- Returns Ok(())
- Node label is now empty string

### Scenario 4: Unicode Label Applied
Given: A DiagramDocument with node "n1"
When: UpdateLabel { id: "n1", label: "中文标签" } is projected
Then:
- Returns Ok(())
- Node label equals "中文标签"
- Other properties unchanged

### Scenario 5: Other Properties Preserved
Given: A DiagramDocument with node "n1" at (50, 50) with dimensions 100x60
When: UpdateLabel { id: "n1", label: "Updated" } is projected
Then:
- Node position remains (50, 50)
- Node dimensions remain 100x60
- Other nodes unchanged
