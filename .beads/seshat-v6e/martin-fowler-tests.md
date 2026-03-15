# Martin Fowler Test Plan: ID Remapping on Paste

## Happy Path Specifications
- `pasting_single_node_generates_new_unique_id`
- `pasting_single_node_applies_immediate_spatial_offset`
- `pasting_multiple_nodes_preserves_relative_positions`
- `pasting_connected_nodes_preserves_their_connection`
- `pasting_child_node_with_its_parent_preserves_hierarchy`
- `pasting_child_node_without_its_parent_attaches_to_original_parent`
- `pasting_root_node_without_parent_remains_root_node`
- `pasting_deeply_nested_topology_preserves_full_hierarchy`
- `multiple_sequential_pastes_increment_paste_serial_and_apply_increasing_offsets`
- `pasting_returns_delta_updating_document_selection_to_newly_pasted_ids`

## Error Path Specifications
- `pasting_fails_when_clipboard_is_empty`
- `pasting_edge_fails_when_target_node_is_missing_from_clipboard`
- `pasting_edge_fails_when_source_node_is_missing_from_clipboard`
- `pasting_edge_fails_when_both_source_and_target_nodes_are_missing_from_clipboard`
- `pasting_fails_when_parent_references_missing_node`
- `pasting_malformed_clipboard_with_cyclic_parent_references_fails`
- `pasting_fails_when_deterministic_prng_generates_colliding_id`
- `pasting_fails_when_clipboard_contains_duplicate_node_ids_internally`
- `pasting_fails_when_clipboard_contains_duplicate_edge_ids_internally`

## Edge Case Specifications
- `pasting_complex_topology_with_multiple_edges_and_nodes`
- `pasting_when_original_nodes_were_deleted_from_document`

## Fuzzing & Mutation Specifications
- `fuzz_adversarial_clipboard_payloads_never_panic`
- `mutation_test_topological_algorithms_ensure_no_logic_inversions`

## Contract Verification Specifications
- `postcondition_all_pasted_nodes_have_new_unique_ids`
- `postcondition_all_pasted_edges_have_new_unique_ids`
- `postcondition_selection_matches_exactly_pasted_items`
- `invariant_document_contains_no_dangling_edges_after_paste`

## Property-Based Specifications (proptest)
- `proptest_pasting_random_valid_clipboard_preserves_all_document_invariants`

## Contract Violation Scenarios

### P1: Empty Clipboard Violation
**Given**: `calculate_paste` is called with an empty clipboard
**When**: The function executes
**Then**: Returns `Err(Error::EmptyClipboard)`

### P2: Dangling Edge Violation
**Given**: `calculate_paste` is called with a clipboard containing an edge connecting two nodes, but only one (or neither) of the nodes is in the clipboard
**When**: The function executes
**Then**: Returns `Err(Error::InvalidEdgeReference)`

### P3: Unresolvable Parent Violation
**Given**: `calculate_paste` is called with a clipboard containing a node assigned to a parent, but the parent exists neither in the clipboard nor in the target document
**When**: The function executes
**Then**: Returns `Err(Error::InvalidParentReference)`

### P4: Cyclic Parent Reference Violation
**Given**: `calculate_paste` is called with a clipboard containing nodes that form a parent-child cycle
**When**: The function executes
**Then**: Returns `Err(Error::CyclicParentReference)`

### P5: Corrupt Clipboard Violation
**Given**: `calculate_paste` is called with a clipboard containing two nodes (or edges) with the exact same ID internally
**When**: The function executes
**Then**: Returns `Err(Error::CorruptClipboard)`

### Q1/I1: Duplicate ID Collision Violation
**Given**: `calculate_paste` is called in an environment where the newly generated ID will collide with an existing ID in the document
**When**: The function executes
**Then**: Returns `Err(Error::DuplicateIdCreated)`

## Given-When-Then Integration Scenarios

### Scenario 1: Topological ID Remapping Integration
**Given**: 
- A document containing a "Server Node" and a "Database Node", with a "Connection Edge" linking them.
- A user has copied the "Server Node", "Database Node", and "Connection Edge" to their clipboard.
**When**: 
- `calculate_paste` is called.
**Then**:
- A `PasteResult` is returned containing the new nodes and edge with remapped IDs.
- The original document remains unmodified by the function.
- Applying the `PasteResult` delta updates the document selection to highlight exactly the newly pasted items.
