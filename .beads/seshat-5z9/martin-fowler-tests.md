# Martin Fowler Test Plan: Copy and Paste (CLP-001 to CLP-005)

## Happy Path Tests
- `test_clp001_copy_paste_single_node_creates_new_node_with_new_id`
- `test_clp002_copy_paste_multiple_nodes_preserves_edges_and_remaps_ids`
- `test_clp003_copy_paste_subgraph_preserves_parent_child_relationships`
- `test_clp004_cut_operation_removes_original_nodes_and_places_in_clipboard`
- `test_clp005_paste_operation_applies_incremental_offset_based_on_serial`

## Error Path Tests
- `test_copy_returns_error_when_selection_is_empty`
- `test_cut_returns_error_when_selection_is_empty`
- `test_paste_returns_error_when_clipboard_is_empty`

## Edge Case Tests
- `test_paste_large_payload_maintains_performance_and_id_uniqueness`
- `test_copy_nodes_with_dangling_edges_in_selection_handles_safely`

## Contract Verification Tests
- `test_precondition_selection_must_not_be_empty_for_copy`
- `test_precondition_clipboard_must_not_be_empty_for_paste`
- `test_postcondition_paste_assigns_strictly_unique_ids`
- `test_postcondition_paste_applies_correct_offset`
- `test_invariant_edges_always_reference_valid_nodes_after_paste`

## Contract Violation Tests
- `test_p1_violation_returns_empty_selection_error`
  Given: `Selection::empty()`
  When: `copy(selection, doc)` is called
  Then: returns `Err(Error::EmptySelection)`

- `test_p3_violation_returns_empty_selection_error`
  Given: `Selection::empty()`
  When: `cut(selection, &mut doc)` is called
  Then: returns `Err(Error::EmptySelection)`

- `test_p4_violation_returns_empty_clipboard_error`
  Given: `ClipboardData::empty()`
  When: `paste(clipboard, &mut doc, 1)` is called
  Then: returns `Err(Error::EmptyClipboard)`

- `test_q1_violation_returns_postcondition_error_for_changed_original_id`
  Given: A mock copy implementation that changes original node ID
  When: The output is validated
  Then: returns `Err(Error::PostconditionViolated("Original node ID changed"))`

- `test_q2_violation_returns_postcondition_error_for_missing_edges`
  Given: A mock copy implementation that ignores edges between selected nodes
  When: The output is validated
  Then: returns `Err(Error::PostconditionViolated("Edges missing from clipboard"))`

- `test_q3_violation_returns_postcondition_error_for_nodes_not_deleted`
  Given: A mock cut implementation that leaves original nodes in doc
  When: The output is validated
  Then: returns `Err(Error::PostconditionViolated("Nodes not deleted"))`

- `test_q4_violation_returns_duplicate_id_error`
  Given: A mock paste implementation that reuses original IDs
  When: The output is validated
  Then: returns `Err(Error::DuplicateIdCreated)`

- `test_q5_violation_returns_postcondition_error_for_zero_offset`
  Given: A mock paste implementation that applies 0 offset for paste_serial > 0
  When: The output is validated
  Then: returns `Err(Error::PostconditionViolated("Incorrect offset applied"))`

- `test_q6_violation_returns_invalid_edge_reference_error`
  Given: A mock paste implementation that copies original edge references verbatim
  When: The output is validated
  Then: returns `Err(Error::InvalidEdgeReference)`

- `test_q7_violation_returns_invalid_parent_reference_error`
  Given: A mock paste implementation that retains original parent ID verbatim
  When: The output is validated
  Then: returns `Err(Error::InvalidParentReference)`

## Given-When-Then Scenarios

### Scenario 1: CLP-001 Copy and Paste Single Node
Given: A document with Node A selected
When: The copy operation is performed, followed by a paste operation
Then:
- Node A remains in the document unchanged
- A new Node B is added to the document
- Node B has a distinct ID from Node A
- Node B's coordinates are offset from Node A by the standard paste offset

### Scenario 2: CLP-002 Copy Multiple Nodes with Edges
Given: A document with Node A and Node B, connected by Edge E1, all selected
When: The copy operation is performed, followed by a paste operation
Then:
- Node A, Node B, and Edge E1 remain unchanged
- New Node C and Node D are created with new IDs
- New Edge E2 is created connecting Node C and Node D
- Edge E2's source and target references exactly match the new IDs of Node C and Node D

### Scenario 3: CLP-003 Copy Subgraph Structure
Given: A parent Subgraph Node P containing Child Node C, both selected
When: The copy operation is performed, followed by a paste operation
Then:
- A new Subgraph Node P' is created
- A new Child Node C' is created
- Node C' references Node P' as its parent

### Scenario 4: CLP-004 Cut Operation
Given: A document with Node A selected
When: The cut operation is performed
Then:
- Node A is removed from the document
- The returned clipboard data contains Node A
- A subsequent paste operation creates Node B with a new ID

### Scenario 5: CLP-005 Paste Operation Offset Increment
Given: A clipboard containing Node A
When: The paste operation is performed three consecutive times without a new copy
Then:
- Three new nodes are created in the document
- The first pasted node has an offset of (20, 20) relative to Node A
- The second pasted node has an offset of (40, 40) relative to Node A
- The third pasted node has an offset of (60, 60) relative to Node A
