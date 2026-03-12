# Martin Fowler Test Plan: seshat-bml (UI Dispatch: Node Deletion)

## Overview
This test plan validates the Delete/Backspace key handler that constructs `DomainOp::NodeDelete` for selected nodes and dispatches to `db_tx` (event sourcing path).

---

## Happy Path Tests

### test_delete_key_dispatches_node_delete_event_for_single_selected_node
**Given**: A diagram with one selected node (id: "node-1")
**When**: User presses Delete
**Then**:
- Exactly one `EventEnvelope` with `DomainOp::NodeDelete { id: "node-1" }` is sent to `db_tx`
- The envelope has valid `op_id` (UUID v4 format)
- The envelope has valid `author` with id "local-user"
- The envelope has valid `timestamp` (current Unix epoch ms)
- The envelope has valid `revision`

### test_delete_key_dispatches_multiple_node_delete_events_for_multiple_selections
**Given**: A diagram with three selected nodes (ids: "node-1", "node-2", "node-3")
**When**: User presses Delete
**Then**:
- Three separate `EventEnvelope` messages are sent to `db_tx`
- Each envelope contains `DomainOp::NodeDelete` with the respective node ID
- Each envelope has unique `op_id`

### test_backspace_key_triggers_same_dispatch_as_delete
**Given**: A diagram with one selected node (id: "node-1")
**When**: User presses Backspace
**Then**:
- Exactly one `EventEnvelope` with `DomainOp::NodeDelete { id: "node-1" }` is sent to `db_tx`
- Same behavior as Delete key

### test_delete_clears_selection_after_successful_dispatch
**Given**: A diagram with selected nodes "node-1" and "node-2", db_tx is available
**When**: User presses Delete and dispatch succeeds
**Then**:
- `editor_state.selected_items` is cleared after successful dispatch

### test_delete_returns_correct_dispatch_result
**Given**: A diagram with two selected nodes
**When**: User presses Delete
**Then**:
- Returns `Ok(DispatchResult { nodes_affected: 2, dispatches_sent: 2 })`

---

## Error Path Tests

### test_delete_with_empty_selection_returns_no_op
**Given**: A diagram with no selected items
**When**: User presses Delete
**Then**:
- Returns `Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })` (no error, no event dispatched)
- No `EventEnvelope` is sent to `db_tx`

### test_delete_when_db_tx_unavailable_falls_back_to_local_mutation
**Given**: A diagram with selected node "node-1" and db_tx is None
**When**: User presses Delete
**Then**:
- Local `apply_delete_selected` function is called as fallback
- Document is mutated directly (node removed from document.nodes)
- History is pushed before mutation

### test_delete_does_not_fire_when_editing_text
**Given**: User is typing in an input field
**When**: User presses Delete or Backspace
**Then**:
- Key action is not triggered (handled by JS-side check)
- No `EventEnvelope` is sent

### test_delete_with_ctrl_or_shift_modifier_ignored
**Given**: A diagram with selected node "node-1"
**When**: User presses Ctrl+Delete or Shift+Delete
**Then**:
- Delete action is NOT triggered (modifier keys change the action)
- No `EventEnvelope` is sent

---

## Edge Case Tests

### test_delete_multiple_times_sends_multiple_events
**Given**: A diagram with selected node "node-1"
**When**: User presses Delete three times rapidly
**Then**:
- Three separate `EventEnvelope` messages are sent to `db_tx`
- Each with unique `op_id`

### test_delete_handles_missing_node_gracefully
**Given**: A selected item ID "node-missing" that doesn't exist in document.nodes
**When**: User presses Delete
**Then**:
- Skips the missing node, only dispatches for existing nodes
- Returns `Ok(DispatchResult { nodes_affected: X, dispatches_sent: X })` where X is count of existing nodes

### test_delete_with_mixed_valid_and_invalid_selection_ids
**Given**: Selected items contain valid node "node-1" and invalid "node-missing"
**When**: User presses Delete
**Then**:
- Only "node-1" results in a dispatch
- No error is returned

### test_delete_handles_locked_nodes_correctly
**Given**: A diagram with selected nodes including a locked node
**When**: User presses Delete
**Then**:
- Locked nodes are filtered out by `selected_node_ids` function
- Only unlocked nodes are dispatched for deletion

---

## Contract Verification Tests

### test_keyboard_mapping_delete_returns_delete_action
**Given**: Key "Delete", ctrl_or_meta=false, shift=false, is_editing_text=false
**When**: `map_key_to_action` is called
**Then**:
- Returns `KeyAction::Delete`

### test_keyboard_mapping_backspace_returns_delete_action
**Given**: Key "Backspace", ctrl_or_meta=false, shift=false, is_editing_text=false
**When**: `map_key_to_action` is called
**Then**:
- Returns `KeyAction::Delete`

### test_keyboard_mapping_delete_ignored_when_editing
**Given**: Key "Delete", ctrl_or_meta=false, shift=false, is_editing_text=true
**When**: `map_key_to_action` is called
**Then**:
- Returns `KeyAction::None`

### test_keyboard_mapping_ctrl_delete_returns_none
**Given**: Key "Delete", ctrl_or_meta=true, shift=false, is_editing_text=false
**When**: `map_key_to_action` is called
**Then**:
- Returns `KeyAction::None` (no action for Ctrl+Delete)

---

## Contract Violation Tests

### test_violation_p1_key_invalid_returns_ok_zero_dispatch
**Given**: Key press is Delete/Backspace but with modifiers (ctrl=true or meta=true)
**When**: `map_key_to_action` is called with modifiers
**Then**: Returns `KeyAction::None` -- not KeyAction::Delete

### test_violation_p2_editing_text_blocks_deletion
**Given**: is_editing_text=true
**When**: `map_key_to_action` is called with Delete key
**Then**: Returns `KeyAction::None` -- keyboard handler is bypassed entirely

### test_violation_p3_empty_selection_returns_ok_with_zero_counts
**Given**: Empty selection `selected_items = {}`
**When**: `handle_delete_key` is called
**Then**: Returns `Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })` -- NOT an error, just no-op

### test_violation_p4_db_tx_unavailable_with_fallback_failure
**Given**: db_tx is None AND local fallback `apply_delete_selected` returns false
**When**: `handle_delete_key` is called
**Then**: Should produce `Err(Error::DbTxUnavailable)`

### test_violation_q1_wrong_node_id_in_domain_op_returns_error
**Given**: Selected node ID is "node-1" but wrong ID sent in DomainOp
**When**: Constructing `DomainOp::NodeDelete` with wrong ID
**Then**: Should produce `Err(Error::InvalidNodeId)` if detected

### test_violation_q2_missing_timestamp_returns_error
**Given**: EventEnvelope being constructed
**When**: timestamp field is not set (None)
**Then**: Should produce `Err(Error::InvalidEnvelope)`

### test_violation_q3_local_mutation_in_happy_path_returns_error
**Given**: db_tx is available and dispatch succeeds
**When**: Document is mutated directly (node removed from document.nodes)
**Then**: Should produce `Err(Error::PostconditionViolation)`

### test_violation_q4_selection_not_cleared_returns_error
**Given**: db_tx successfully sent the event
**When**: Selection is NOT cleared after dispatch
**Then**: Should produce `Err(Error::PostconditionViolation)`

### test_violation_i1_dispatch_count_mismatch_returns_error
**Given**: Three nodes selected but only one dispatch sent
**When**: `handle_delete_key` is called
**Then**: Should produce `Err(Error::DispatchIncomplete)`

---

## Given-When-Then Scenarios

### Scenario 1: Successful Node Deletion via Keyboard
**Given**: 
- Diagram document with nodes "node-1", "node-2", "node-3"
- "node-1" and "node-2" are selected in `editor_state.selected_items`
- db_tx coroutine is available

**When**:
- User presses Delete

**Then**:
- Two `EventEnvelope` messages are sent to db_tx:
  - First: `DomainOp::NodeDelete { id: "node-1" }`
  - Second: `DomainOp::NodeDelete { id: "node-2" }`
- Each envelope has valid op_id, author, timestamp, revision
- Selection is cleared after dispatch
- "node-3" remains in the document

### Scenario 2: Fallback to Local Mutation
**Given**:
- Diagram document with node "node-1" selected
- db_tx is None (unavailable)

**When**:
- User presses Delete

**Then**:
- `apply_delete_selected` is called
- Document is mutated: "node-1" is removed from document.nodes
- History is pushed with previous state
- Selection is cleared

### Scenario 3: No-op with Empty Selection
**Given**:
- Diagram document with no selected items

**When**:
- User presses Delete or Backspace

**Then**:
- No dispatch to db_tx
- No local mutation
- Returns `Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })`

### Scenario 4: Text Editing Prevents Deletion
**Given**:
- User is editing text in a label input field

**When**:
- User presses Delete or Backspace

**Then**:
- No Delete action is triggered
- Text in the input field is deleted (browser default behavior)
- No diagram nodes are affected

---

## Integration with Existing Tests

This bead's tests should complement:
- `keyboard_tests.rs` - Tests for KeyAction::Delete mapping
- `phase4_model_updates.rs` - Integration tests for DomainOp::NodeDelete
- `io_tests.rs` - Persistence tests including node deletion
