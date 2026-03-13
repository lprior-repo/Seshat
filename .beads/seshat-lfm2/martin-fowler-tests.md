# Martin Fowler Test Plan: seshat-lfm2 - HIS-003..HIS-008

## Overview

Test plan for the History system's undo/redo functionality, specifically covering HIS-003 through HIS-008 test cases. These tests verify that the history system correctly captures and restores document state for complex mutations.

## Happy Path Tests

### test_his003_drag_creates_single_history_entry
**Scenario**: Drag gesture creates one history entry

Given:
- A document with a node at position (100, 100)
- History initialized with initial state pushed

When:
- Node is dragged to new position (150, 150) and state is pushed once

Then:
- History undo_stack has exactly 1 entry (not per-frame)
- Undo restores original position (100, 100)

### test_his004_group_undo_removes_group
**Scenario**: Undo after grouping nodes removes the group

Given:
- A document with two nodes (node-a, node-b) at positions (100, 100) and (200, 100)
- Both nodes have no parent (are top-level)

When:
- Nodes are grouped into a subgraph
- Undo is called

Then:
- Group node (subgraph) is removed from document
- node-a parent is None
- node-b parent is None

### test_his005_reparent_undo_restores_parent
**Scenario**: Undo after reparenting restores original parent

Given:
- A document with parent-1, parent-2, and child nodes
- Child node has parent: parent-1

When:
- Child is reparented to parent-2
- Undo is called

Then:
- Child node's parent is restored to parent-1

### test_his006_connector_create_undo_removes_edge
**Scenario**: Undo after creating edge removes the edge

Given:
- A document with node-a and node-b (no edges)

When:
- An edge is created connecting node-a to node-b
- Undo is called

Then:
- Edge is removed from document
- edges collection is empty

### test_his007_style_change_undo_restores_style
**Scenario**: Undo after changing node style restores original style

Given:
- A document with a node having style: NodeStyle::Box

When:
- Node style is changed to NodeStyle::Dashed
- Undo is called

Then:
- Node style is restored to NodeStyle::Box

### test_his008_text_edit_creates_single_entry
**Scenario**: Text edit creates single history entry

Given:
- A document with a node labeled "Original Label"

When:
- Label is changed to "New Label" and pushed

Then:
- History undo_stack has exactly 1 entry
- Undo restores original label "Original Label"

### test_apply_undo_success_restores_previous_state
**Scenario**: apply_undo restores document to previous state

Given:
- History with initial state pushed: push(state_a)
- Another state pushed: push(state_b)

When:
- apply_undo(current_doc) is called

Then:
- Returns Ok with document restored to state_a
- Redo stack now contains state_b

### test_apply_undo_failure_returns_error_on_empty_history
**Scenario**: apply_undo fails when no history available

Given:
- History with no pushes (empty undo_stack)

When:
- apply_undo(current_doc) is called

Then:
- Returns Err("Cannot undo: undo stack is empty")

### test_apply_redo_success_restores_next_state
**Scenario**: apply_redo restores document to next state

Given:
- History with push(state_a), push(state_b)
- Undo performed (now at state_a, redo_stack has state_b)

When:
- apply_redo(current_doc) is called

Then:
- Returns Ok with document restored to state_b
- Undo stack now contains state_a and state_b

### test_apply_redo_failure_returns_error_on_empty_redo_stack
**Scenario**: apply_redo fails when no redo available

Given:
- History with one push (no undo performed)

When:
- apply_redo(current_doc) is called

Then:
- Returns Err("Cannot redo: redo stack is empty")

## Error Path Tests

### test_undo_on_empty_history_returns_none
**Scenario**: Undo on fresh history returns None

Given:
- History with no pushes (empty undo_stack)

When:
- undo() is called

Then:
- Returns None (not panic)

### test_redo_on_empty_redo_stack_returns_none
**Scenario**: Redo when no undo performed returns None

Given:
- History with one push (no undo performed)

When:
- redo() is called

Then:
- Returns None (not panic)

### test_multiple_redo_on_exhausted_stack_returns_none
**Scenario**: Redo after all redo entries exhausted returns None

Given:
- History with push(A), push(B), undo (back to A), redo (forward to B)
- Now at B with empty redo_stack

When:
- redo() is called again

Then:
- Returns None (not panic)

## Edge Case Tests

### test_undo_stack_bounded_at_100
**Scenario**: History does not grow beyond MAX_HISTORY

Given:
- History with 100 pushes

When:
- One more push is performed

Then:
- undo_stack length is exactly 100 (oldest entry dropped)

### test_multiple_operations_create_multiple_entries
**Scenario**: Multiple operations each create a history entry

Given:
- History initialized with initial state pushed

When:
- push(state_a) is performed
- push(state_b) is performed
- push(state_c) is performed

Then:
- undo_stack has exactly 3 entries
- Each entry corresponds to distinct state

### test_push_after_undo_clears_redo_stack
**Scenario**: New action after undo clears redo stack

Given:
- History with push(A), push(B)
- Undo performed (back to A, redo_stack has B)

When:
- push(C) is performed

Then:
- redo_stack is empty
- undo_stack has A, C

### test_can_undo_returns_correct_state
**Scenario**: can_undo reflects actual undo capability

Given:
- Fresh History

Then:
- can_undo() returns false

Given:
- History with one push

Then:
- can_undo() returns true

### test_can_redo_returns_correct_state
**Scenario**: can_redo reflects actual redo capability

Given:
- Fresh History

Then:
- can_redo() returns false

Given:
- History with push, undo performed

Then:
- can_redo() returns true

## Contract Verification Tests

### test_integration_e2e_full_history_workflow
**Integration Test**: Full undo/redo workflow end-to-end

Given:
- A fresh DiagramDocument with a node at position (0, 0)
- History initialized: `let history = History::new().push(doc.clone())`

When:
1. Node is moved to (100, 100) and pushed: `history = history.push(doc1)`
2. Node is moved to (200, 200) and pushed: `history = history.push(doc2)`
3. Undo is performed: `(doc_restored, history) = history.undo(doc2).unwrap()`
4. Undo is performed: `(doc_restored, history) = history.undo(doc_restored).unwrap()`
5. Redo is performed: `(doc_redo1, history) = history.redo(doc_restored).unwrap()`
6. Redo is performed: `(doc_redo2, history) = history.redo(doc_redo1).unwrap()`

Then:
- After step 1: undo_stack.len() = 1
- After step 2: undo_stack.len() = 2
- After step 3: can_redo() = true, undo_stack.len() = 1
- After step 4: can_redo() = true, undo_stack.len() = 0
- After step 5: can_undo() = true, undo_stack.len() = 1
- After step 6: can_undo() = true, undo_stack.len() = 2
- Final document position is (200, 200)

This test exercises the full history system end-to-end, verifying that:
- Multiple pushes create proper history entries
- Undo correctly restores previous states
- Redo correctly restores forward states
- Stack boundaries are respected
- can_undo/can_redo correctly reflect state

### test_precondition_p2_undo_requires_nonempty_stack
**VIOLATES_P2**: `History::new().undo(doc)` returns None (contract satisfied - does not panic)

Given:
- History with empty undo_stack

When:
- undo() is called

Then:
- Returns None (not panic or error)

### test_precondition_p3_redo_requires_nonempty_stack
**VIOLATES_P3**: After undo+redo exhausts redo_stack, redo returns None

Given:
- History with push(A), undo (at A, redo has A)
- redo (at original, redo empty)

When:
- redo() is called again

Then:
- Returns None

### test_postcondition_q2_push_clears_redo_stack
**VIOLATES_Q2**: Verify push clears redo stack

Given:
- History with push(A), push(B), undo (back to A, redo has B)

When:
- push(C) is called

Then:
- Result has empty redo_stack

### test_postcondition_q8_single_entry_per_operation
**VIOLATES_Q8**: Verify single entry per logical operation

Given:
- History with push(A)

When:
- Single push(B) representing completed drag gesture

Then:
- undo_stack.len() is exactly 2 (A and B)

### test_invariant_i1_undo_stack_is_reverse_chronological
**VIOLATES_I1**: Verify undo stack maintains reverse chronological order

Given:
- History with push(A), push(B), push(C)

Then:
- undo_stack[0] = A (oldest)
- undo_stack[1] = B
- undo_stack[2] = C (newest)

### test_invariant_i2_redo_stack_is_chronological
**VIOLATES_I2**: Verify redo stack maintains chronological order

Given:
- History with push(A), push(B)
- Undo (back to A), undo (back to initial)

When:
- Two redo operations available

Then:
- redo_stack[0] = A (oldest redo)
- redo_stack[1] = B (newest redo)

### test_invariant_i3_after_push_redo_stack_is_empty
**VIOLATES_I3**: Verify redo stack is empty after push

Given:
- History with push(A), push(B), undo (back to A, redo has B)

When:
- push(C) is called (new timeline branch)

Then:
- redo_stack is empty
- undo_stack contains [A, C]

### test_invariant_i4_after_undo_can_redo_is_true
**VIOLATES_I4**: Verify can_redo returns true after undo

Given:
- History with push(A), push(B)

When:
- Undo is performed (current state: A)

Then:
- can_redo() returns true

### test_invariant_i5_after_redo_can_undo_is_true
**VIOLATES_I5**: Verify can_undo returns true after redo

Given:
- History with push(A), push(B)
- Undo performed (at A, redo has B)

When:
- Redo is performed (at B)

Then:
- can_undo() returns true

## Given-When-Then Scenarios

### Scenario 1: HIS-003 - Drag Gesture Creates One Entry
**Test**: test_his003_drag_creates_single_history_entry

Given:
- A DiagramDocument with a Node at position (100, 100)
- History initialized via `History::new().push(doc_before)`

When:
- Node is moved to (150, 150), document revision incremented
- New state pushed via `history.push(doc_after)`

Then:
- `history.undo_stack.len()` equals 1
- Calling `history.undo(doc_after)` returns Some
- The restored document has node at (100, 100)

### Scenario 2: HIS-004 - Group Undo Removes Group
**Test**: test_his004_group_undo_removes_group

Given:
- DiagramDocument with node-a and node-b (both top-level, parent: None)
- History: `History::new().push(doc_before)`

When:
- Create group (subgraph) containing node-a and node-b
- node-a.parent = Some(group_id), node-b.parent = Some(group_id)
- Push: `history.push(doc_with_group)`

Then:
- `history.undo(doc_with_group)` returns Some
- Restored document does not contain group_id
- Restored node-a.parent is None
- Restored node-b.parent is None

### Scenario 3: HIS-005 - Reparent Undo Restores Parent
**Test**: test_his005_reparent_undo_restores_parent

Given:
- DiagramDocument with parent-1, parent-2, child (child.parent = Some(parent-1))
- History: `History::new().push(doc_before)`

When:
- Change child.parent to Some(parent-2)
- Push: `history.push(doc_after)`

Then:
- `history.undo(doc_after)` returns Some
- Restored child.parent equals Some(parent-1)

### Scenario 4: HIS-006 - Connector Undo Removes Edge
**Test**: test_his006_connector_create_undo_removes_edge

Given:
- DiagramDocument with node-a, node-b, and no edges
- History: `History::new().push(doc_before)`

When:
- Add edge from node-a to node-b
- Push: `history.push(doc_with_edge)`

Then:
- `history.undo(doc_with_edge)` returns Some
- Restored document has empty edges collection

### Scenario 5: HIS-007 - Style Change Undo
**Test**: test_his007_style_change_undo_restores_style

Given:
- DiagramDocument with node having style: Some(NodeStyle::Box)
- History: `History::new().push(doc_before)`

When:
- Change node.style to Some(NodeStyle::Dashed)
- Push: `history.push(doc_after)`

Then:
- `history.undo(doc_after)` returns Some
- Restored node.style equals Some(NodeStyle::Box)

### Scenario 6: HIS-008 - Text Edit Creates Single Entry
**Test**: test_his008_text_edit_creates_single_entry

Given:
- DiagramDocument with node.label = "Original"
- History: `History::new().push(doc_before)`

When:
- Change label to "New"
- Push: `history.push(doc_after)`

Then:
- `history.undo_stack.len()` equals 1
- `history.undo(doc_after)` returns Some
- Restored node.label equals "Original"

## Test Implementation Notes

- Use `#[cfg(test)]` module with `use super::*` imports
- Create helper function `make_node(label, x, y, width, height)` for consistent test node creation
- Create helper function `doc_with_revision(steps)` to generate docs with specific revisions
- All tests should verify `.is_some()` before accessing returned values
- Use descriptive assertion messages: `assert_eq!(actual, expected, "description")`
