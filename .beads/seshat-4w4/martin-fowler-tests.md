# Martin Fowler Test Plan: Cut and Duplicate

## Happy Path Tests
- `test_cut_single_node_returns_clipboard_and_removes_from_doc`
- `test_cut_multiple_nodes_with_edges_removes_subgraph`
- `test_duplicate_single_node_creates_new_node_with_offset`
- `test_duplicate_multiple_nodes_with_edges_preserves_topology`

## Error Path Tests
- `test_cut_with_empty_selection_returns_empty_selection_error`
- `test_duplicate_with_empty_selection_returns_empty_selection_error`

## Edge Case Tests
- `test_cut_removes_dangling_edges_if_target_is_cut_but_not_source` (verify referential integrity)
- `test_duplicate_assigns_new_unique_ids_to_all_cloned_nodes_and_edges`
- `test_duplicate_multiple_times_applies_increasing_offsets` (optional, depending on offset strategy)

## Contract Verification Tests
- `test_precondition_selection_not_empty_cut`
- `test_precondition_selection_not_empty_duplicate`
- `test_postcondition_cut_removes_nodes`
- `test_postcondition_duplicate_adds_nodes_and_selects_them`

## Contract Violation Tests
- `test_cut_selection_empty_violation_returns_error`
  **Given:** An empty `doc.editor_state.selected_items`
  **When:** `cut_selection(&mut doc)` is called
  **Then:** returns `Err(ClipboardError::EmptySelection)` -- NOT a panic, NOT an unwrap failure

- `test_duplicate_selection_empty_violation_returns_error`
  **Given:** An empty `doc.editor_state.selected_items`
  **When:** `duplicate_selection(&mut doc)` is called
  **Then:** returns `Err(ClipboardError::EmptySelection)` -- NOT a panic, NOT an unwrap failure

## Given-When-Then Scenarios

### Scenario 1: Cut Subgraph
**Given:** A document with Nodes A, B, C and Edges A->B, B->C
**And:** Nodes A and B are selected
**When:** `cut_selection` is executed
**Then:**
- The returned `ClipboardData` contains Nodes A and B and Edge A->B
- The document node count is 1 (Node C remains)
- The document edge count is 0 (Edge B->C is removed due to referential integrity)
- The document selection is empty

### Scenario 2: Duplicate Node
**Given:** A document with Node A at position (10, 10)
**And:** Node A is selected
**When:** `duplicate_selection` is executed
**Then:**
- The document node count is 2 (Node A and new Node B)
- The new Node B has a unique ID, not equal to Node A
- The new Node B is at position (30, 30) (assuming 20px offset)
- Node B is the only selected item

### Scenario 3: Duplicate Edges Topology
**Given:** A document with Nodes A, B and Edge A->B
**And:** Both Nodes A and B are selected
**When:** `duplicate_selection` is executed
**Then:**
- The document has 4 nodes (A, B, A', B') and 2 edges (A->B, A'->B')
- The new Edge A'->B' connects the newly created nodes, not the original ones
- The document selection contains A' and B' only

### Scenario 4: Cut Empty Selection
**Given:** A document with Node A
**And:** No nodes are selected
**When:** `cut_selection` is executed
**Then:**
- An error `ClipboardError::EmptySelection` is returned
- Node A remains in the document
- Document state is strictly unchanged
