# Martin Fowler Test Plan: seshat-84y

## Overview

This test plan covers the SendBackward toolbar button feature that wires the button to construct `DomainOp::SendBackward` and dispatch it to the `db_tx` coroutine.

---

## Happy Path Tests

### test_toolbar_send_backward_creates_valid_domain_operation
**Given**: A DiagramDocument with nodes at z-order [A, B, C] where C is selected, a valid History, and a working db_tx coroutine  
**When**: Toolbar SendBackward button is clicked  
**Then**:
- A `DomainOp::SendBackward` is constructed with valid `ids` field
- The operation contains the selected node ID(s) in a Vec
- The envelope's operation field is `DomainOp::SendBackward { ids }`

### test_toolbar_send_backward_dispatches_to_db_tx
**Given**: A DiagramDocument with selected nodes and a working db_tx coroutine (Some variant)  
**When**: send_backward action is invoked  
**Then**:
- An EventEnvelope is sent through db_tx.send()
- The envelope's operation field is `DomainOp::SendBackward`
- The envelope's op_id is a non-empty UUID string
- The envelope's author.id is "local-user"
- The envelope's timestamp is a valid Unix timestamp

### test_toolbar_send_backward_updates_local_z_order
**Given**: A DiagramDocument with nodes [A, B, C] where C is selected (C is at highest z-order)  
**When**: send_backward action is invoked  
**Then**:
- Node C's z_index is decremented (moved backward)
- The relative z-order of unselected nodes is preserved
- The revision number is incremented

### test_toolbar_send_backward_updates_history
**Given**: A History that can_undo() returns false, document with selected node  
**When**: send_backward action is invoked  
**Then**:
- History.can_undo() returns true
- Undo restores the z-order to previous state

### test_toolbar_send_backward_returns_true_when_nodes_moved
**Given**: A DiagramDocument with selectable nodes at different z-order levels  
**When**: send_backward action is invoked with valid selection  
**Then**:
- Returns `true` indicating successful z-order change

---

## Error Path Tests

### test_send_backward_handles_missing_db_tx_gracefully
**Given**: A DiagramDocument with selected nodes, History, and db_tx = None  
**When**: send_backward action is invoked  
**Then**:
- No panic occurs
- Local z-order is still updated (for immediate UI feedback)
- Debug is logged about unavailable db_tx
- Returns `true` if nodes moved

### test_send_backward_returns_false_when_no_selection
**Given**: A DiagramDocument with empty selected_items  
**When**: send_backward action is invoked  
**Then**:
- Returns `false`
- No EventEnvelope is sent to db_tx
- No local mutation occurs
- No history update

### test_send_backward_returns_false_when_all_nodes_locked
**Given**: A DiagramDocument where all selected nodes have `locked = true` and are not Subgraph kind  
**When**: send_backward action is invoked  
**Then**:
- Returns `false`
- No EventEnvelope is sent to db_tx
- No local mutation occurs
- No history update

### test_send_backward_allows_locked_subgraphs
**Given**: A DiagramDocument with selected nodes where selected includes a Subgraph (which can be moved even when locked)  
**When**: send_backward action is invoked  
**Then**:
- Subgraph node is included in the z-order change
- EventEnvelope is sent with Subgraph ID in ids
- Returns `true`

---

## Edge Case Tests

### test_send_backward_single_selected_node
**Given**: A DiagramDocument with nodes [A, B, C] and only C selected (C is at front)  
**When**: send_backward action is invoked  
**Then**:
- Node C moves one position backward in z-order
- EventEnvelope contains ids: ["C"]
- Returns `true`

### test_send_backward_multiple_selected_nodes
**Given**: A DiagramDocument with nodes [A, B, C, D] and nodes C and D selected  
**When**: send_backward action is invoked  
**Then**:
- Nodes C and D move one position backward (if possible)
- EventEnvelope contains ids: ["C", "D"]
- Both nodes' z_index values are updated

### test_send_backward_noncontiguous_selection
**Given**: A DiagramDocument with nodes [A, B, C, D] and nodes A and D selected (non-contiguous)  
**When**: send_backward action is invoked  
**Then**:
- Node D moves backward (since there are unselected nodes after it)
- Node A cannot move backward (no unselected nodes before it)
- EventEnvelope contains ids: ["A", "D"]
- Partial success: function returns `true` if any movement occurred

### test_send_backward_front_node_already_at_back
**Given**: A DiagramDocument with nodes [A, B, C] where A is at front (z-index highest) and only A selected  
**When**: send_backward action is invoked  
**Then**:
- Node A cannot move further back
- Returns `false` (no change possible)
- No dispatch to db_tx

### test_send_backward_handles_empty_document
**Given**: A fresh DiagramDocument with no nodes  
**When**: send_backward action is invoked  
**Then**:
- Returns `false`
- No dispatch to db_tx

### test_send_backward_preserves_unselected_z_order
**Given**: A DiagramDocument with nodes [A, B, C] and only C selected  
**When**: send_backward action is invoked  
**Then**:
- Nodes A and B maintain their relative z-order
- Only C's z_index changes

---

## Contract Verification Tests

### test_precondition_p1_db_tx_optional
**Given**: db_tx = None  
**When**: send_backward is called with valid selection  
**Then**:
- Precondition P1 satisfied: handles gracefully
- Local z-order still updated

### test_precondition_p4_selection_not_empty
**Given**: doc_signal with non-empty selected_items  
**When**: send_backward is called  
**Then**:
- Precondition P4 satisfied: selection exists

### test_precondition_p5_movable_nodes_exist
**Given**: doc_signal with selected nodes where at least one is not locked or is Subgraph  
**When**: send_backward is called  
**Then**:
- Precondition P5 satisfied: movable nodes available

### test_postcondition_q1_domain_op_constructed
**Given**: Selected node IDs  
**When**: send_backward action runs  
**Then**:
- Postcondition Q1 satisfied: DomainOp::SendBackward { ids } constructed

### test_postcondition_q5_event_sent_to_db_tx
**Given**: A working db_tx coroutine  
**When**: send_backward is invoked with valid selection  
**Then**:
- Postcondition Q5 satisfied: .send() called on db_tx

### test_postcondition_q6_local_z_order_updated
**Given**: DiagramDocument with selected nodes  
**When**: send_backward completes  
Then:
- Postcondition Q6 satisfied: doc_signal z_index values changed

### test_postcondition_q7_history_pushed
**Given**: History  
**When**: send_backward completes successfully  
Then:
- Postcondition Q7 satisfied: history_signal contains previous state

### test_postcondition_q8_revision_incremented
**Given**: Document with revision N  
**When**: send_backward completes  
Then:
- Postcondition Q8 satisfied: revision = N + 1

### test_invariant_i3_history_consistency
**Given**: History that can_undo()  
**When**: undo is called after send_backward  
Then:
- Invariant I3 maintained: History push/pop is consistent

### test_invariant_i5_unselected_unchanged
**Given**: Document with selected and unselected nodes  
**When**: send_backward completes  
Then:
- Invariant I5 maintained: unselected node z-order unchanged

---

## Contract Violation Tests

### test_violation_p1_db_tx_none_continues_locally
**Given**: db_tx = None  
**When**: send_backward is invoked with valid selection  
**Then**: Returns true (or false if no movement), logs debug, local state updated

```rust
// VIOLATES P1: send_backward(doc, history, None)
// Expected: No panic, local update continues, debug log
```

### test_violation_p4_empty_selection_returns_false
**Given**: doc_signal with empty selected_items  
**When**: send_backward is invoked  
**Then**: Returns false, no dispatch

```rust
// VIOLATES P4: send_backward with no selection
// Expected: Err(false) - returns false, no EventEnvelope sent
```

### test_violation_p5_all_locked_returns_false
**Given**: doc_signal where all selected nodes are locked and not Subgraph  
**When**: send_backward is invoked  
**Then**: Returns false, no dispatch

```rust
// VIOLATES P5: send_backward with all selected nodes locked
// Expected: Returns false, no EventEnvelope sent
```

---

## Given-When-Then Scenarios

### Scenario 1: Toolbar SendBackward Button Click
**Given**:
- A DiagramDocument with 3 nodes at z-order: [Node1, Node2, Node3] (Node3 at front)
- Node3 is selected
- History in initial state
- db_tx coroutine is available

**When**:
User clicks the "Back" (SendBackward) toolbar button

**Then**:
- Node3 moves one step back: z-order becomes [Node1, Node3, Node2]
- EventEnvelope with DomainOp::SendBackward { ids: ["Node3"] } is sent to db_tx
- History.can_undo() is true
- Document revision is incremented
- Function returns `true`

### Scenario 2: SendBackward Without Database Connection
**Given**:
- A DiagramDocument with selected nodes
- History
- db_tx = None (not available)

**When**:
User clicks SendBackward button

**Then**:
- Local z-order is still updated (for immediate feedback)
- Debug is logged about db_tx unavailability
- No panic occurs
- User sees the nodes reorder in the UI
- Function returns `true` if nodes moved

### Scenario 3: SendBackward With No Selection
**Given**:
- A DiagramDocument with nodes but nothing selected
- db_tx is available

**When**:
User clicks SendBackward button

**Then**:
- Nothing happens
- No dispatch to db_tx
- Function returns `false`
- Button is disabled when selection is empty (UI-level)

### Scenario 4: SendBackward All Nodes Locked
**Given**:
- A DiagramDocument where all selected nodes have `locked = true`
- db_tx is available

**When**:
User clicks SendBackward button (button enabled but selection is locked)

**Then**:
- No z-order change occurs
- No dispatch to db_tx
- Function returns `false`
- Button should ideally be disabled when all selected are locked

### Scenario 5: Rapid Sequential SendBackward
**Given**:
- A DiagramDocument with 3 nodes [A, B, C] where C is selected
- db_tx is available

**When**:
User clicks SendBackward 3 times in quick succession

**Then**:
- First click: [A, C, B]
- Second click: [C, A, B] (C cannot move further back)
- Third click: no change (C already at back)
- 2 EventEnvelopes sent to db_tx (first two clicks)
- History tracks all operations
- Final function returns `false` (no change on third)

---

## Integration Considerations

- The toolbar button has `data-testid="toolbar-send-backward"` for E2E testing (already exists)
- The button should be disabled when selection is empty (already implemented: `disabled: stats.selected_count == 0`)
- Consider also disabling when all selected nodes are locked and not Subgraphs
- The local mutation happens first, then db_tx dispatch (optimistic UI update)
