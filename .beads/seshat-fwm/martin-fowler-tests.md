# Martin Fowler Test Plan

## Happy Path Tests

### test_dispatch_update_label_when_node_label_changes_on_blur
**Given**: A node exists with label "Old Label", the user edits it to "New Label" and blurs the input
**When**: `commit_inline_edit` is called with the new value and a valid `db_tx`
**Then**:
- An `EventEnvelope` with `DomainOp::UpdateLabel { target_id: node_id, target_type: Node, new_label: "New Label", old_label: "Old Label" }` is sent to `db_tx`
- The document's node label is updated to "New Label"
- The revision is incremented

### test_dispatch_update_label_when_edge_label_changes_on_blur
**Given**: An edge exists with label "Old Edge Label", the user edits it to "New Edge Label" and blurs the input
**When**: `commit_inline_edit` is called with the new value and a valid `db_tx`
**Then**:
- An `EventEnvelope` with `DomainOp::UpdateLabel { target_id: edge_id, target_type: Edge, new_label: "New Edge Label", old_label: "Old Edge Label" }` is sent to `db_tx`
- The document's edge label is updated to "New Edge Label"
- The revision is incremented

### test_dispatch_update_label_on_enter_key
**Given**: A node exists with label "Old", the user edits it to "New" and presses Enter
**When**: `commit_inline_edit` is called (via onkeydown Enter) with the new value
**Then**:
- Same behavior as onBlur - event is dispatched

### test_editing_signals_cleared_after_successful_commit
**Given**: A node is being edited (`editing_node = Some(node_id)`)
**When**: `commit_inline_edit` is called with changed label
**Then**:
- `editing_node` is set to `None`
- `editing_edge` remains `None`

---

## Error Path Tests

### test_no_dispatch_when_db_tx_is_none
**Given**: A node exists with label "Old", user changes it to "New" but no backend connection
**When**: `commit_inline_edit` is called with `db_tx: None`
**Then**:
- Local document is updated with new label
- Revision is incremented
- No panic occurs
- (The function should succeed locally but skip dispatch)

### test_returns_error_when_target_node_not_found
**Given**: A document with no nodes, or the target node was deleted
**When**: `commit_inline_edit` is called with a node_id that doesn't exist
**Then**:
- Returns `Err(CommitError::TargetNotFound)`
- No event is dispatched
- Document state is unchanged

### test_handles_db_tx_send_failure_gracefully
**Given**: A valid `db_tx` but the channel is closed/failing
**When**: `commit_inline_edit` is called and `db_tx.send()` returns an error
**Then**:
- Local document update still occurs (optimistic local-first)
- Error is logged or returned but does not propagate as panic

---

## Edge Case Tests

### test_no_dispatch_when_label_unchanged
**Given**: A node exists with label "Same Label", user focuses the input and blurs without changing
**When**: `commit_inline_edit` is called with current value "Same Label"
**Then**:
- NO `EventEnvelope` is sent to `db_tx` (critical: this is the "unwanted behavior" guard)
- Document revision is NOT incremented (no actual change)
- `editing_node` is set to `None`

### test_empty_label_allowed
**Given**: A node exists with label "Text", user clears the input to ""
**When**: `commit_inline_edit` is called with empty string
**Then**:
- Event is dispatched with `new_label: ""`
- Document node label becomes empty string

### test_unicode_label_handling
**Given**: A node exists with label "English", user changes to "日本語"
**When**: `commit_inline_edit` is called with Unicode label
**Then**:
- Event is dispatched with correct UTF-8 encoding
- Document updates correctly

### test_very_long_label_truncation_or_handling
**Given**: A user enters a very long label (>10KB)
**When**: `commit_inline_edit` is called
**Then**:
- Either truncates to max length, or returns error, or accepts (document current behavior)
- No panic occurs

---

## Contract Verification Tests

### test_precondition_p1_db_tx_handle_valid
**Given**: A valid `db_tx` coroutine handle
**When**: `commit_inline_edit` is invoked
**Then**: The function does not panic on None handle

### test_precondition_p2_target_exists
**Given**: Document with a specific node_id in nodes map
**When**: `commit_inline_edit(target_id=node_id)`
**Then**: Target is found, no TargetNotFound error

### test_precondition_p4_label_change_validated
**Given**: `current_label != new_label`
**When**: `commit_inline_edit` is called
**Then**: Proceeds to dispatch event

### test_postcondition_q1_event_dispatched_on_change
**Given**: `new_label != current_label`
**When**: `commit_inline_edit` completes
**Then**: EventEnvelope was sent to db_tx

### test_postcondition_q2_no_event_on_no_change
**Given**: `new_label == current_label`
**When**: `commit_inline_edit` completes
**Then**: NO EventEnvelope was sent to db_tx

### test_postcondition_q4_signals_cleared
**Given**: Editing is active
**When**: `commit_inline_edit` completes (regardless of change)
**Then**: `editing_node` and `editing_edge` are both `None`

### test_invariant_i1_mutual_exclusion
**Given**: `editing_node = Some(id)` or `editing_edge = Some(id)`
**When**: `commit_inline_edit` runs
**Then**: After completion, both are `None` (mutual exclusion enforced on exit)

### test_invariant_i3_no_panic_on_none_db_tx
**Given**: `db_tx = None`
**When**: `commit_inline_edit` is called
**Then**: No panic, graceful handling

---

## Contract Violation Tests

### test_violation_p1_no_panic_on_none_db_tx
**Given**: `db_tx = None` (from canvas.rs when no backend)
**When**: `commit_inline_edit(..., db_tx)` is called with changed label
**Then**: Returns `Ok(())` or graceful error, NOT a panic

### test_violation_p2_target_not_found_returns_error
**Given**: Target node doesn't exist in document
**When**: `commit_inline_edit(..., edit_value="new")` is called
**Then**: Returns `Err(CommitError::TargetNotFound)`

### test_violation_q1_changed_label_no_dispatch_is_violation
**Given**: `new_label != current_label` but NO dispatch occurs
**When**: The code is executed
**Then**: This is a contract violation - test should FAIL if no dispatch

### test_violation_q2_unchanged_label_dispatch_is_violation  
**Given**: `new_label == current_label`
**When**: `commit_inline_edit` IS called (not skipped)
**Then**: Contract violation if EventEnvelope IS sent (Q2 violated)

---

## Integration Scenario Tests

### test_end_to_end_node_label_edit_via_onblur
**Given**: A canvas with one node "Test Node"
**When**: User double-clicks to edit, changes to "Updated Node", blurs
**Then**:
1. Input field receives focus
2. oninput updates edit_value signal
3. onBlur triggers commit_inline_edit
4. Local document updates to "Updated Node"
5. EventEnvelope dispatched to db_tx
6. editing_node set to None
7. UI re-renders showing "Updated Node"

### test_end_to_end_edge_label_edit_via_enter_key
**Given**: A canvas with one edge between nodes
**When**: User double-clicks edge label, changes text, presses Enter
**Then**:
1. Same flow as node, but via onkeydown Enter handler
2. Event dispatched with Edge target type
