# Martin Fowler Test Plan

## Happy Path Tests
- **test_doc_006_node_deletion_removes_node_from_document**
  - Given: A document with one node
  - When: Deleting the node
  - Then: Node is removed from document.nodes

- **test_doc_007_node_deletion_cascades_connected_edges**
  - Given: A document with two nodes and an edge connecting them
  - When: Deleting the source node
  - Then: The edge is automatically removed (cascade)

- **test_doc_008_node_deletion_removes_all_connected_edges**
  - Given: A document with three nodes (N1, N2, N3) and edges N1->N2, N2->N3, N1->N3
  - When: Deleting node N2
  - Then: Edges N1->N2 and N2->N3 are removed, N1->N3 remains

## Error Path Tests
- **test_doc_009_node_not_found_returns_error**
  - Given: An empty document
  - When: Attempting to delete a non-existent node
  - Then: Returns `Err(DocumentError::NodeNotFound(...))`

- **test_doc_010_locked_node_deletion_returns_error**
  - Given: A document with a locked node
  - When: Attempting to delete the locked node
  - Then: Returns `Err(DocumentError::NodeLocked(...))`

- **test_doc_010_locked_node_deletion_preserves_document_state**
  - Given: A document with a locked node
  - When: Attempting to delete the locked node (returns error)
  - Then: Document state is unchanged

## Edge Case Tests
- **test_node_deletion_handles_self_loop_edge**
  - Given: A document with a node that has a self-loop edge
  - When: Deleting the node
  - Then: Node and self-loop edge are both removed

- **test_node_deletion_handles_isolated_node**
  - Given: A document with a node that has no edges
  - When: Deleting the node
  - Then: Node is removed successfully

- **test_node_deletion_handles_multiple_isolated_nodes**
  - Given: A document with multiple unconnected nodes
  - When: Deleting one node
  - Then: Only the specified node is removed

## Contract Verification Tests
- **test_invariant_no_dangling_edge_references**
  - Given: A document with nodes and edges
  - When: Deleting a node
  - Then: No edge in document.edges references the deleted node

- **test_invariant_all_edge_endpoints_exist**
  - Given: Any valid document state
  - Then: For every edge, both source and target nodes exist in document.nodes

## Contract Violation Tests
- `test_p1_violation_returns_node_not_found`
  - Given: Empty document
  - When: `doc.remove_node(&NodeId::new("nonexistent"))`
  - Then: Returns `Err(DocumentError::NodeNotFound(...))`

- `test_p2_violation_returns_node_locked`
  - Given: Document with locked node
  - When: `doc.remove_node(&NodeId::new("locked_node"))` where node.locked == true
  - Then: Returns `Err(DocumentError::NodeLocked(...))`

- `test_i1_violation_no_dangling_edges`
  - Given: Document with node and connected edge
  - When: After remove_node() returns Ok
  - Then: No edge has source or target equal to deleted node ID

## Given-When-Then Scenarios
### Scenario 1: DOC-006 - Node Deletion Basic
**Given**: A document with a single node "N1"
**When**: User deletes node "N1"
**Then**:
- Node "N1" is not in document.nodes
- Document returns Ok(())

### Scenario 2: DOC-007 - Edge Cascade Deletion
**Given**: A document with nodes "N1", "N2" and edge "E1" from N1 to N2
**When**: User deletes node "N1"
**Then**:
- Node "N1" is removed
- Edge "E1" is removed (cascade)
- Node "N2" remains

### Scenario 3: DOC-010 - Locked Node Rejection
**Given**: A document with a locked node "N1"
**When**: User attempts to delete node "N1"
**Then**:
- Returns error DocumentError::NodeLocked
- Node "N1" remains in document
- Document state is unchanged
