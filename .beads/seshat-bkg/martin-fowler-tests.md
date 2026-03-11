# Martin Fowler Test Plan

## Happy Path Tests
- `test_mul031_move_preserves_relative_positions`
- `test_mul032_resize_scales_items_proportionally`
- `test_mul033_delete_removes_all_selected_items`
- `test_mul034_copy_paste_duplicates_selection_with_offset`
- `test_mul035_undo_redo_restores_selection_state`
- `test_mul037_parent_child_selection_rules_handled_correctly`

## Error Path Tests
- `test_mul036_mutating_locked_items_returns_error`
- `test_invalid_hierarchy_returns_error`

## Edge Case Tests
- `test_copy_paste_single_item_in_multi_select_context`
- `test_delete_all_items_in_document`

## Contract Verification Tests
- `test_precondition_p2_item_locked`
- `test_precondition_p3_invalid_hierarchy`
- `test_postcondition_q1_delete_clears_selection`
- `test_postcondition_q2_move_preserves_spacing`
- `test_invariant_i1_no_duplicate_node_ids_in_selection`
- `test_invariant_i3_locked_items_never_mutated`

## Contract Violation Tests
- `test_p2_violation_returns_item_locked_error`
  Given: A selection `selection_with_locked_node` where at least one node is marked as locked
  When: `delete_selection` is called with this selection
  Then: returns `Err(Error::ItemLocked)`

- `test_p3_violation_returns_invalid_hierarchy_error`
  Given: A selection `selection_with_parent_and_child` containing both a container and its direct child
  When: `move_selection` is called
  Then: returns `Err(Error::InvalidHierarchy)`

- `test_q1_violation_returns_postcondition_error`
  Given: A mock document that fails to remove all selected items
  When: `delete_selection` is called
  Then: returns `Err(Error::PostconditionViolated)`

## Given-When-Then Scenarios

### Scenario 1: MUL-031 Move Preserves Relative Positions
Given: Two selected nodes A at (10,10) and B at (20,20)
When: `move_selection` is executed with delta (5,5)
Then:
- Node A is located at (15,15)
- Node B is located at (25,25)
- The relative distance between A and B remains identical
- Returns `Ok(())`

### Scenario 2: MUL-033 Delete Removes All Selected Items
Given: A document with nodes A, B, C, and selection [A, B]
When: `delete_selection` is executed
Then:
- Node A and B are removed from the document
- Node C remains in the document
- The document selection is empty
- Returns `Ok(())`

### Scenario 3: MUL-034 Copy/Paste Duplicates Selection
Given: A document with selected nodes A and B
When: `copy_selection` is called, followed by `paste_selection` with offset (10,10)
Then:
- Two new nodes A' and B' are added to the document
- A' and B' have identical properties to A and B but new IDs
- A' and B' are offset from A and B's original positions by (10,10)
- The active selection is updated to [A', B']

### Scenario 4: MUL-035 Undo Restores Selection State
Given: A selection of [A, B] that was just deleted
When: The Undo operation is executed
Then:
- Nodes A and B are restored to the document
- The active selection is restored to [A, B]

### Scenario 5: MUL-036 Locked Item Constraint (P2 Validation)
Given: A selection [A, B] where A is locked
When: `move_selection` is attempted
Then:
- Returns `Err(Error::ItemLocked)`
- Neither A nor B is moved
- Document state remains unchanged
