# Martin Fowler Test Plan: Clipboard Contract

## Happy Path Tests
- test_clp001_copy_paste_single_node_creates_new_node_with_new_id
- test_clp002_copy_paste_multiple_nodes_preserves_edges_and_remaps_ids
- test_clp003_copy_paste_subgraph_preserves_parent_child_relationships
- test_clp004_cut_operation_removes_original_nodes_and_places_in_clipboard
- test_clp005_paste_operation_applies_incremental_offset_based_on_serial

## Error Path Tests
- test_copy_returns_error_when_selection_is_empty
- test_cut_returns_error_when_selection_is_empty
- test_paste_returns_error_when_clipboard_is_empty

## Edge Case Tests
- test_handles_single_node_copy_paste_correctly
- test_handles_multiple_nodes_with_edges
- test_handles_subgraph_with_parent_child_relationships
- test_handles_incremental_paste_offset
- test_handles_cut_followed_by_paste
- test_handles_paste_serial_zero
- test_handles_paste_with_no_edges

## Contract Verification Tests
- test_precondition_p1_selection_not_empty_for_copy
- test_precondition_p2_selection_not_empty_for_cut
- test_precondition_p3_clipboard_not_empty_for_paste
- test_postcondition_q1_copy_does_not_mutate_document
- test_postcondition_q2_paste_generates_new_ids
- test_postcondition_q4_cut_removes_original_nodes
- test_postcondition_q5_paste_applies_offset
- test_invariant_i1_all_edge_references_valid_after_paste
- test_invariant_i2_all_parent_references_valid_after_paste

## Contract Violation Tests
- test_p1_violation_returns_empty_selection_error
  Given: Selection with empty nodes vector
  When: copy() is called
  Then: returns Err(Error::EmptySelection)

- test_p2_violation_returns_empty_selection_error
  Given: Selection with empty nodes vector
  When: cut() is called
  Then: returns Err(Error::EmptySelection)

- test_p3_violation_returns_empty_clipboard_error
  Given: ClipboardData with empty nodes vector
  When: paste() is called
  Then: returns Err(Error::EmptyClipboard)

- test_q1_violation_returns_postcondition_error_for_changed_original_id
  Given: Valid document with node
  When: copy() is called
  Then: document nodes remain unchanged

- test_q6_violation_returns_invalid_edge_reference_error
  Given: ClipboardData with edge referencing non-existent node
  When: paste() is called
  Then: returns Err(Error::InvalidEdgeReference)

- test_q7_violation_returns_invalid_parent_reference_error
  Given: ClipboardData with node having non-existent parent
  When: paste() is called
  Then: returns Err(Error::InvalidParentReference)

## Given-When-Then Scenarios

### Scenario 1: Single Node Copy and Paste
Given: A document with one node at (0, 0)
When: User selects the node, copies, then pastes with serial 1
Then: A new node is created at (20, 20) with a new unique ID
And: The original node remains in the document

### Scenario 2: Multi-Node Copy with Edges
Given: A document with two nodes and an edge between them
When: User selects both nodes, copies, then pastes
Then: Both nodes are copied with new IDs
And: The edge is copied with a new ID
And: The edge's source/target point to the new node IDs

### Scenario 3: Subgraph Copy with Parent-Child
Given: A document with parent node P and child node C (where C.parent = P)
When: User selects both P and C, copies, then pastes
Then: New parent P' and child C' are created
And: C'.parent = P' (parent reference is remapped)

### Scenario 4: Cut Removes Original
Given: A document with one node
When: User cuts the node
Then: The node is removed from the document
And: The node is placed in the clipboard
And: The selection is cleared

### Scenario 5: Incremental Paste Offset
Given: A document with one node at (0, 0), copied to clipboard
When: User pastes three times (serials 1, 2, 3)
Then: First paste creates node at (20, 0)
And: Second paste creates node at (40, 0)
And: Third paste creates node at (60, 0)

### Scenario 6: Invalid Edge Reference Rejected
Given: Clipboard with edge referencing a node not in clipboard
When: User pastes
Then: Error::InvalidEdgeReference is returned

### Scenario 7: Invalid Parent Reference Rejected
Given: Clipboard with child node referencing parent not in clipboard or document
When: User pastes
Then: Error::InvalidParentReference is returned
