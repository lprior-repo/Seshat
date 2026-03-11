# Martin Fowler Test Plan: seshat-6vd

## Test Strategy

This test plan follows the Given-When-Then pattern to specify behavior through executable tests. Tests are organized into:
1. Happy Path Tests - valid operations
2. Error Path Tests - precondition failures
3. Edge Case Tests - boundary conditions
4. Contract Verification Tests - pre/post/invariants

---

## Happy Path Tests

### test_ctrl_g_dispatches_group_operation_when_two_nodes_selected
**Given**: User has exactly 2 nodes selected in the canvas
**When**: User presses Ctrl+G keyboard shortcut
**Then**:
- An `EventEnvelope` is sent to `db_tx` channel
- The envelope contains `DomainOp::Group { ids }` with both node IDs
- The `op_id` is a valid UUID string
- The `author` is set to "local-user" / "Local User"
- The `timestamp` is a valid Unix timestamp (milliseconds)

### test_ctrl_g_dispatches_group_operation_when_three_nodes_selected
**Given**: User has 3 nodes selected in the canvas
**When**: User presses Ctrl+G keyboard shortcut
**Then**:
- An `EventEnvelope` is sent to `db_tx` channel
- The envelope contains `DomainOp::Group { ids }` with all three node IDs
- All selected IDs are present in the `ids` vector

### test_ctrl_g_dispatches_group_operation_with_many_nodes_selected
**Given**: User has 10+ nodes selected in the canvas
**When**: User presses Ctrl+G keyboard shortcut
**Then**:
- An `EventEnvelope` is sent to `db_tx` channel
- The `ids` vector contains all 10+ node IDs

---

## Error Path Tests

### test_ctrl_g_no_dispatch_when_single_node_selected
**Given**: User has exactly 1 node selected in the canvas
**When**: User presses Ctrl+G keyboard shortcut
**Then**:
- No `EventEnvelope` is sent to `db_tx` channel
- The function returns early without error
- No toast or error is shown to user

### test_ctrl_g_no_dispatch_when_zero_nodes_selected
**Given**: User has no nodes selected in the canvas (empty selection)
**When**: User presses Ctrl+G keyboard shortcut
**Then**:
- No `EventEnvelope` is sent to `db_tx` channel
- The function returns early silently

### test_ctrl_g_no_dispatch_when_db_tx_unavailable
**Given**: User has 2+ nodes selected, but `db_tx` context is None
**When**: User presses Ctrl+G keyboard shortcut
**Then**:
- No panic occurs
- No `EventEnvelope` is sent
- Function returns early gracefully (optionally logs to console)

### test_ctrl_g_no_dispatch_without_ctrl_modifier
**Given**: User has 2+ nodes selected
**When**: User presses plain "G" key (without Ctrl)
**Then**:
- No group operation is dispatched
- No state mutation occurs

---

## Edge Case Tests

### test_ctrl_g_handles_non_contiguous_selection
**Given**: User has nodes "node-1", "node-5", "node-10" selected (non-contiguous IDs)
**When**: User presses Ctrl+G
**Then**:
- All three non-contiguous node IDs are included in `DomainOp::Group { ids }`

### test_ctrl_g_handles_already_grouped_nodes
**Given**: User selects nodes where some are already children of a group
**When**: User presses Ctrl+G
**Then**:
- Operation still dispatches (backend handles validation)
- No client-side error

### test_ctrl_g_idempotent_multiple_presses
**Given**: User presses Ctrl+G multiple times with valid selection
**When**: Ctrl+G is pressed 3 times in succession
**Then**:
- 3 separate `EventEnvelope` messages are sent
- Each has a unique `op_id` (UUID)
- Each contains the same selected node IDs

### test_ctrl_g_ignores_non_node_selections
**Given**: User has selected edges instead of nodes (if applicable in selection model)
**When**: User presses Ctrl+G
**Then**:
- Behavior is graceful (no panic)

---

## Contract Verification Tests

### test_precondition_p1_ctrl_pressed_guard
**Given**: A keyboard event with key="G" but ctrl=false
**When**: The keyboard handler processes the event
**Then**:
- The group case is NOT matched
- No group operation is dispatched

### test_precondition_p2_selection_count_validation
**Given**: A document with `selected_items.len() = 1`
**When**: The Ctrl+G handler checks preconditions
**Then**:
- Early return occurs before any dispatch

### test_precondition_p3_channel_availability
**Given**: `db_tx` is `None`
**When**: Ctrl+G handler attempts to send
**Then**:
- No send is attempted
- Early return occurs

### test_postcondition_q1_dispatch_occurs
**Given**: Valid selection (2+ nodes) and available channel
**When**: Ctrl+G handler executes
**Then**:
- `tx.send()` is invoked at least once

### test_postcondition_q2_ids_vector_completeness
**Given**: Selected items contain specific IDs
**When**: Group operation is constructed
**Then**:
- The `ids` field contains ALL selected node IDs (no omissions)

### test_postcondition_q3_selection_not_mutated
**Given**: Document with selected items
**When**: Ctrl+G is pressed
**Then**:
- `selected_items` HashSet remains unchanged after handler completes

---

## Contract Violation Tests

### test_violation_p2_single_node_returns_early
**Given**: `selected_items = {"node-1"}`
**When**: Ctrl+G handler is invoked
**Then**: Returns `()` (unit) - NOT an error, NOT a panic

### test_violation_p2_zero_nodes_returns_early
**Given**: `selected_items = {}`
**When**: Ctrl+G handler is invoked
**Then**: Returns `()` - silent no-op

### test_violation_p3_channel_none_returns_early
**Given**: `db_tx = None`, valid selection
**When**: Ctrl+G handler is invoked
**Then**: Returns `()` - graceful handling, no panic

### test_violation_q1_send_failure_is_graceful
**Given**: Channel exists but `send()` fails
**When**: Ctrl+G handler executes
**Then**: No panic, error is handled gracefully (logged or shown as toast)

### test_violation_q2_ids_mismatch_is_caught
**Given**: Selected items = {"A", "B", "C"}
**When**: `DomainOp::Group` is constructed with only ["A", "B"]
**Then**: This is a bug - postcondition Q2 violated - should have all 3 IDs

---

## Given-When-Then Scenarios

### Scenario 1: Successful Group Operation
**Given**: Canvas is active, 3 nodes selected (id1, id2, id3), Ctrl held down
**When**: User presses G key
**Then**:
- `DomainOp::Group { ids: ["id1", "id2", "id3"] }` is sent to db_tx
- EventEnvelope has valid op_id, author, timestamp
- No errors are shown to user

### Scenario 2: Insufficient Selection
**Given**: Canvas is active, 1 node selected, Ctrl held down
**When**: User presses G key
**Then**:
- No operation is dispatched
- No error message is displayed
- Selection remains unchanged

### Scenario 3: No Channel Available
**Given**: Canvas is active, 2+ nodes selected, Ctrl held down, db_tx is None
**When**: User presses G key
**Then**:
- No panic occurs
- No operation is dispatched
- Console may log a warning

### Scenario 4: Wrong Modifier
**Given**: Canvas is active, 2+ nodes selected, NO Ctrl modifier
**When**: User presses G key
**Then**:
- No group operation is triggered
- Tool mode may change (if "g" maps to a tool)

---

## Test Execution Notes

- These tests verify UI event handling behavior
- Unit tests can be written for the handler function in isolation
- Integration tests require the Dioxus runtime and context providers
- Mock `db_tx` channel for testing dispatch without actual backend
- Use property-based testing for ID vector completeness
