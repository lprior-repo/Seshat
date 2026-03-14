# Martin Fowler Test Plan for seshat-1m9: Z-index Ordering (DOC-016 to DOC-020)

## Overview

This test plan validates strict z-index handling for nodes that overlap or are brought to front. Tests verify uniqueness, sequential ordering, relative order preservation, and proper handling of locked nodes.

## Happy Path Tests

### BringToFront Operations
- `test_bring_to_front_moves_selected_nodes_to_front`
  - Given: Document with nodes [A, B, C, D, E] at z-indexes [0, 1, 2, 3, 4]
  - When: bring_to_front is called with nodes B and D selected
  - Then: New z-indexes are [0, 2, 4, 3, 1] (selected moved to end, order preserved)

- `test_bring_to_front_returns_true_when_changes_made`
  - Given: Document with multiple nodes, some selected
  - When: bring_to_front is called
  - Then: Returns true indicating changes were made

- `test_bring_to_front_preserves_relative_order_of_selected`
  - Given: Document with nodes [A, B, C, D, E], nodes B (z=1) and D (z=3) selected
  - When: bring_to_front is applied
  - Then: Selected nodes appear at front in same relative order (B before D)

### SendToBack Operations
- `test_send_to_back_moves_selected_nodes_to_back`
  - Given: Document with nodes [A, B, C, D, E] at z-indexes [0, 1, 2, 3, 4]
  - When: send_to_back is called with nodes B and D selected
  - Then: New z-indexes are [1, 3, 0, 2, 4] (selected moved to front, order preserved)

- `test_send_to_back_returns_true_when_changes_made`
  - Given: Document with multiple nodes, some selected
  - When: send_to_back is called
  - Then: Returns true indicating changes were made

### BringForward Operations
- `test_bring_forward_swaps_selected_with_next_unselected`
  - Given: Document with nodes [A, B, C, D, E], node B selected, z-indexes [0, 1, 2, 3, 4]
  - When: bring_forward is called
  - Then: B swaps with C, new z-indexes [0, 2, 1, 3, 4]

- `test_bring_forward_handles_multiple_selected_in_sequence`
  - Given: Document with nodes [A, B, C, D, E], nodes B and C selected
  - When: bring_forward is called
  - Then: B swaps with D, C swaps with E (both move forward one position)

### SendBackward Operations
- `test_send_backward_swaps_selected_with_previous_unselected`
  - Given: Document with nodes [A, B, C, D, E], node C selected, z-indexes [0, 1, 2, 3, 4]
  - When: send_backward is called
  - Then: C swaps with B, new z-indexes [0, 2, 1, 3, 4]

- `test_send_backward_handles_multiple_selected_in_sequence`
  - Given: Document with nodes [A, B, C, D, E], nodes C and D selected
  - When: send_backward is called
  - Then: D swaps with C, C swaps with B (both move backward one position)

### No-Change Scenarios
- `test_returns_false_when_no_nodes_selected`
  - Given: Document with empty selection
  - When: Any z-order operation is called
  - Then: Returns false (no change made)

- `test_returns_false_when_single_node_in_layer`
  - Given: Document with only one node in a layer
  - When: Any z-order operation is called
  - Then: Returns false (no change possible)

## Error Path Tests

- `test_projection_returns_no_nodes_specified_error_for_empty_ids`
  - Given: Empty node ID slice
  - When: apply_bring_forward is called on projection
  - Then: Returns Err(ReplayError::NoNodesSpecified)

- `test_projection_returns_all_nodes_invalid_error_for_nonexistent_ids`
  - Given: Node IDs that don't exist in the document
  - When: apply_bring_to_front is called
  - Then: Returns Err(ReplayError::AllNodesInvalid)

- `test_projection_returns_z_index_overflow_error_for_excessive_nodes`
  - Given: More than i64::MAX nodes (extreme case)
  - When: apply_z_order is called
  - Then: Returns Err(ReplayError::ZIndexOverflow)

## Edge Case Tests

- `test_handles_single_selected_node_correctly`
  - Given: Document with nodes [A, B, C], node B selected
  - When: bring_to_front is called
  - Then: B moves to front, order [A, C, B]

- `test_handles_all_nodes_selected`
  - Given: Document with nodes [A, B, C], all selected
  - When: bring_to_front is called
  - Then: Returns false (no change, already at front relative to each other)

- `test_handles_adjacent_selected_nodes`
  - Given: Document with nodes [A, B, C, D, E], nodes B and C selected (adjacent)
  - When: bring_forward is called
  - Then: B swaps with D, C swaps with E

- `test_handles_adjacent_unselected_nodes`
  - Given: Document with nodes [A, B, C, D, E], nodes A and E selected
  - When: bring_forward is called
  - Then: A swaps with B, E has no unselected node after (no swap)

- `test_handles_mixed_layer_nodes`
  - Given: Document with regular nodes and subgraphs
  - When: z-order operation is called
  - Then: Only affects nodes in same layer (subgraphs with subgraphs, nodes with nodes)

- `test_handles_locked_nodes_as_noops`
  - Given: Document with some locked nodes in selection
  - When: z-order operation is called
  - Then: Locked nodes are excluded, only unlocked nodes move

## Contract Verification Tests

### Precondition Tests
- `test_contract_precondition_valid_z_index_range`
  - Given: Any z-order operation
  - When: Operation completes
  - Then: All z_index values remain within i64 bounds

- `test_contract_precondition_no_overflow`
  - Given: Document with many nodes
  - When: Z-order operation is applied
  - Then: No integer overflow in z-index assignment

- `test_contract_precondition_nodes_exist`
  - Given: Attempting operation with nonexistent node IDs
  - When: apply_bring_forward is called
  - Then: Returns appropriate error

- `test_contract_precondition_layer_separation`
  - Given: Document with subgraphs and regular nodes
  - When: z-order operation is applied
  - Then: Subgraphs and nodes are processed separately

### Postcondition Tests
- `test_contract_postcondition_unique_z_indexes`
  - Given: Any z-order operation on document
  - When: Operation completes
  - Then: No two nodes in same layer have identical z_index

- `test_contract_postcondition_sequential_indexes`
  - Given: Any z-order operation on document
  - When: Operation completes
  - Then: z-indexes are sequential (no gaps) within each layer

- `test_contract_postcondition_relative_order_preserved`
  - Given: Selected nodes [B, D] (B before D in original order)
  - When: bring_to_front is called
  - Then: B and D still maintain B before D at front

- `test_contract_postcondition_bring_forward_swap_count`
  - Given: Multiple selected nodes
  - When: bring_forward is called
  - Then: Each selected node swaps at most once

- `test_contract_postcondition_selection_not_empty`
  - Given: Empty selection
  - When: z-order operation is called
  - Then: Returns false (no-op)

### Invariant Tests
- `test_invariant_z_index_uniqueness_per_layer`
  - Given: Document with nodes and subgraphs
  - When: Multiple z-order operations are performed
  - Then: No two nodes of same kind have identical z_index

- `test_invariant_layer_integrity`
  - Given: Document with subgraphs and regular nodes
  - When: z-order operations are performed
  - Then: Subgraph and node layers remain independent

## Contract Violation Tests

- `test_violation_duplicate_z_indexes_detected`
  - Given: Two nodes with same z_index exist
  - When: Document state is inspected
  - Then: Test fails - uniqueness invariant violated

- `test_violation_gap_in_z_indexes_detected`
  - Given: z-index sequence has gaps (e.g., [0, 1, 3])
  - When: Document state is inspected
  - Then: Test fails - sequential invariant violated

- `test_violation_relative_order_not_preserved`
  - Given: Selected nodes [B, D], B at lower z-index than D
  - When: bring_to_front is applied
  - Then: Test fails if D appears before B at front

- `test_violation_excessive_swaps_in_bring_forward`
  - Given: Selected nodes at various positions
  - When: bring_forward is applied
  - Then: Test fails if any node swaps more than once

## Given-When-Then Scenarios

### Scenario 1: BringToFront with overlapping nodes
- **Given**: Nodes A, B, C with overlapping bounding boxes, z-indexes [0, 1, 2]
- **And**: Nodes A and C are selected (they overlap with B)
- **When**: User clicks "To Front"
- **Then**: A and C move to front, new z-indexes [0, 1, 3, 2]
- **And**: Relative order of A and C is preserved

### Scenario 2: SendToBack with overlapping nodes
- **Given**: Nodes A, B, C with overlapping bounding boxes, z-indexes [0, 1, 2]
- **And**: Nodes A and C are selected
- **When**: User clicks "To Back"
- **Then**: A and C move to back, new z-indexes [1, 3, 0, 2]

### Scenario 3: BringForward at layer boundary
- **Given**: Nodes in order by z-index, last selected node at max z
- **When**: BringForward is applied
- **Then**: Returns false (no change possible)

### Scenario 4: SendBackward at layer boundary
- **Given**: Nodes in order by z-index, first selected node at min z
- **When**: SendBackward is applied
- **Then**: Returns false (no change possible)

### Scenario 5: Mixed locked and unlocked nodes
- **Given**: Document with nodes A (unlocked), B (locked), C (unlocked)
- **And**: All three nodes selected
- **When**: bring_to_front is called
- **Then**: Only A and C move to front, B remains at original position
- **And**: Returns true (changes were made)

### Scenario 6: Subgraph layer separation
- **Given**: Document with regular nodes and subgraphs
- **When**: Z-order operation is applied to regular node selection
- **Then**: Subgraph z-indexes remain unchanged

## Property-Based Tests

- `test_property_idempotent_multiple_bring_to_front`
  - Given: Any valid selection
  - When: bring_to_front is called twice in succession
  - Then: Second call returns false (no-op, already at front)

- `test_property_idempotent_multiple_send_to_back`
  - Given: Any valid selection
  - When: send_to_back is called twice in succession
  - Then: Second call returns false (no-op, already at back)

- `test_property_z_order_deterministic`
  - Given: Same document state
  - When: Same z-order operation is applied multiple times
  - Then: Results are identical each time

- `test_property_selection_filter_excludes_locked`
  - Given: Selection includes locked nodes
  - When: z-order operation is applied
  - Then: Locked nodes are filtered out before processing
