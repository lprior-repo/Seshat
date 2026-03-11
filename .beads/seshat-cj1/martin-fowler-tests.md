# Martin Fowler Test Plan: seshat-cj1 (UI Dispatch: Inline Text Enter)

## Overview
This test plan validates that the inline text editor Enter key event constructs `DomainOp::UpdateLabel` and dispatches to the `db_tx` coroutine, following the event sourcing pattern used by other UI dispatch operations.

## Happy Path Tests

### test_enter_key_dispatches_update_label_to_db_tx
**Given**: A DiagramDocument with node "node-1" having label "Old Label", editing_node is Some("node-1"), edit_value is "New Label", and db_tx is Some(coroutine)
**When**: User presses Enter key, triggering commit_inline_edit
**Then**:
- db_tx receives exactly one EventEnvelope
- The envelope operation is DomainOp::UpdateLabel { target_id: "node-1", label: "New Label", target_type: Node }
- Document's node-1 label is updated to "New Label"

### test_on_blur_dispatches_update_label_to_db_tx
**Given**: A DiagramDocument with node "node-a" having label "Original", editing_node is Some("node-a"), edit_value is "Modified", and db_tx is Some(coroutine)
**When**: Focus leaves the inline text editor (onBlur event)
**Then**:
- db_tx receives exactly one EventEnvelope with DomainOp::UpdateLabel
- The envelope contains correct target_id and label

### test_label_unchanged_returns_noop
**Given**: A DiagramDocument with node "node-1" having label "SameLabel", editing_node is Some("node-1"), edit_value is "SameLabel", and db_tx is Some(coroutine)
**When**: User presses Enter (or onBlur triggers)
**Then**:
- No EventEnvelope is sent to db_tx
- Function returns Ok(false) indicating no-op

### test_edge_label_update_dispatch
**Given**: A DiagramDocument with edge "e1" having label "OldEdge", editing_edge is Some("e1"), edit_value is "NewEdge", and db_tx is Some(coroutine)
**When**: User presses Enter in edge label editing mode
**Then**:
- db_tx receives exactly one EventEnvelope
- The envelope operation is DomainOp::UpdateLabel { target_id: "e1", label: "NewEdge", target_type: Edge }

## Error Path Tests

### test_no_edit_active_returns_error
**Given**: A DiagramDocument with no active editing (both editing_node and editing_edge are None)
**When**: commit_inline_edit is called
**Then**: Returns Err(UpdateLabelError::NoEditActive)

### test_node_target_not_found_returns_error
**Given**: A DiagramDocument, editing_node is Some("nonexistent-node"), and db_tx is available
**When**: commit_inline_edit is called
**Then**: Returns Err(UpdateLabelError::TargetNotFound("nonexistent-node"))

### test_db_tx_unavailable_falls_back_to_direct_mutation
**Given**: A DiagramDocument with node "node-1" having label "Old", editing_node is Some("node-1"), edit_value is "New", db_tx is None
**When**: commit_inline_edit is called
**Then**:
- No dispatch to db_tx (not available)
- Document's node-1 label is updated to "New" via direct mutation
- Returns Ok(true)

### test_dispatch_failed_returns_error
**Given**: A DiagramDocument with node "node-1", editing_node is Some("node-1"), edit_value is "New", db_tx is Some(closed_coroutine)
**When**: commit_inline_edit is called and db_tx.send() fails
**Then**: Returns Err(UpdateLabelError::DispatchFailed(...))

## Edge Case Tests

### test_empty_label_allowed
**Given**: A DiagramDocument with node "node-1" having label "Text", editing_node is Some("node-1"), edit_value is ""
**When**: User clears the label and presses Enter
**Then**:
- EventEnvelope is dispatched with label: ""
- Document's node-1 label becomes empty string

### test_whitespace_only_label
**Given**: A DiagramDocument with node "node-1" having label "Text", editing_node is Some("node-1"), edit_value is "   "
**When**: User enters whitespace-only label and presses Enter
**Then**:
- EventEnvelope is dispatched with label: "   " (whitespace preserved)
- Document's node-1 label becomes "   "

### test_rapid_successive_edits
**Given**: A DiagramDocument with node "node-1", db_tx is Some(coroutine)
**When**: User edits label to "A", presses Enter, then immediately edits to "B", presses Enter
**Then**:
- Two separate EventEnvelopes are dispatched
- Each has unique op_id (UUID v4)

### test_special_characters_in_label
**Given**: A DiagramDocument with node "node-1", editing_node is Some("node-1"), edit_value contains special chars "Node <test> & 'quotes'"
**When**: User presses Enter
**Then**:
- EventEnvelope is dispatched with label containing special characters
- Label is preserved exactly in document

## Contract Verification Tests

### test_precondition_p1_editing_active
**Given**: No active editing session
**When**: commit_inline_edit is invoked
**Then**: Precondition P1 violated - returns Err(NoEditActive)

### test_precondition_p2_target_exists
**Given**: editing_node references non-existent node
**When**: commit_inline_edit is invoked
**Then**: Precondition P2 violated - returns Err(TargetNotFound(...))

### test_postcondition_q1_envelope_dispatched
**Given**: Valid editing session with changed label, db_tx available
**When**: commit_inline_edit succeeds
**Then**: Postcondition Q1 satisfied - db_tx.send() was called once

### test_postcondition_q2_correct_payload
**Given**: Node "node-x" in editing mode with new label "new-text"
**When**: commit_inline_edit succeeds
**Then**: Postcondition Q2 satisfied - envelope has UpdateLabel { target_id: "node-x", label: "new-text", target_type: Node }

### test_postcondition_q3_author_populated
**Given**: Valid editing session, db_tx available
**When**: commit_inline_edit succeeds
**Then**: Postcondition Q3 satisfied - author.id = "local-user", author.name = "Local User"

### test_postcondition_q5_unique_op_id
**Given**: Two rapid label edits
**When**: Both succeed
**Then**: Postcondition Q5 satisfied - each envelope has unique UUID v4 op_id

### test_postcondition_q6_editing_cleared
**Given**: Active editing session
**When**: commit_inline_edit succeeds
**Then**: Postcondition Q6 satisfied - editing_node and editing_edge are both None

### test_postcondition_q7_fallback_behavior
**Given**: db_tx is None
**When**: commit_inline_edit is called
**Then**: Postcondition Q7 satisfied - direct mutation path executed

### test_invariant_i1_revision_increments
**Given**: Document at revision 5
**When**: Successful label update
**Then**: Invariant I1 satisfied - revision becomes 6

### test_invariant_i2_editing_cleared_after_commit
**Given**: editing_node = Some("node-1")
**When**: commit_inline_edit succeeds
**Then**: Invariant I2 satisfied - editing_node = None

## Contract Violation Tests

### test_violation_p1_returns_no_edit_active_error
**Given**: Both editing_node and editing_edge are None
**When**: commit_inline_edit(doc_signal, history_signal, None, None, signal!("value"), Some(db_tx))
**Then**: Returns Err(UpdateLabelError::NoEditActive) -- NOT a panic

### test_violation_p2_returns_target_not_found_error
**Given**: editing_node = Some("fake-id") where node doesn't exist
**When**: commit_inline_edit with fake node ID
**Then**: Returns Err(UpdateLabelError::TargetNotFound("fake-id")) -- NOT a panic

### test_violation_q6_editing_not_cleared
**Given**: Valid editing session
**When**: After successful commit_inline_edit
**Then**: editing_node.read() == None -- assertion passes, editing state cleared

## Given-When-Then Scenarios

### Scenario 1: User edits node label with Enter key
**Given**: Canvas with a node selected, user double-clicks to enter edit mode, types new label "Updated Node"
**When**: User presses Enter key
**Then**:
- EventEnvelope with DomainOp::UpdateLabel is dispatched to db_tx
- Edit mode is exited (editing_node = None)
- Node label in document is "Updated Node"

### Scenario 2: User edits edge label then clicks away (onBlur)
**Given**: Canvas with an edge selected, user enters edit mode for edge label, types "New Edge Label"
**When**: User clicks elsewhere on canvas (blur event)
**Then**:
- EventEnvelope with DomainOp::UpdateLabel is dispatched to db_tx
- Edit mode is exited (editing_edge = None)
- Edge label in document is "New Edge Label"

### Scenario 3: User presses Escape to cancel editing
**Given**: Editing mode active with unsaved changes
**When**: User presses Escape key
**Then**:
- No EventEnvelope dispatched
- Document remains unchanged
- Editing state cleared without label update

### Scenario 4: db_tx unavailable (WASM build)
**Given**: Running in WASM environment where db_tx is None
**When**: User edits label and presses Enter
**Then**:
- Direct document mutation occurs (existing fallback behavior)
- Label is updated in document
- No error returned

## Test Traceability Matrix

| Contract Clause | Test(s) |
|---|---|
| P1 | test_precondition_p1_editing_active, test_violation_p1_returns_no_edit_active_error |
| P2 | test_precondition_p2_target_exists, test_violation_p2_returns_target_not_found_error |
| P3 | test_db_tx_unavailable_falls_back_to_direct_mutation |
| Q1 | test_postcondition_q1_envelope_dispatched |
| Q2 | test_postcondition_q2_correct_payload |
| Q3 | test_postcondition_q3_author_populated |
| Q4 | (Timestamp - implicitly tested via envelope inspection) |
| Q5 | test_postcondition_q5_unique_op_id |
| Q6 | test_postcondition_q6_editing_cleared, test_violation_q6_editing_not_cleared |
| Q7 | test_postcondition_q7_fallback_behavior |
| I1 | test_invariant_i1_revision_increments |
| I2 | test_invariant_i2_editing_cleared_after_commit |
| E1 | test_enter_key_dispatches_update_label_to_db_tx |
| E2 | test_on_blur_dispatches_update_label_to_db_tx |
| E3 | test_label_unchanged_returns_noop |
| E4 | test_db_tx_unavailable_falls_back_to_direct_mutation |

## Test Infrastructure Notes
- Use `tokio::sync::mpsc` channel to mock db_tx coroutine
- Create DiagramDocument fixtures with known node/edge states
- Use `Arc<Mutex<Vec<EventEnvelope>>>` to capture dispatched envelopes for verification
- Signal types can be cloned for test scenarios
