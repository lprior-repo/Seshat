# Martin Fowler Test Plan: seshat-q5e (UI Dispatch: Ungroup Nodes)

## Overview
This test plan validates the Ctrl+Shift+G ungrouping command that constructs `DomainOp::Ungroup` and dispatches to `db_tx` (event sourcing path).

## Test Categories

---

## Happy Path Tests

### test_ctrl_shift_g_dispatches_ungroup_event_with_valid_group_selection
**Given**: A diagram with one selected subgraph node (id: "group-1")
**When**: User presses Ctrl+Shift+G
**Then**:
- Exactly one `EventEnvelope` with `DomainOp::Ungroup { id: "group-1" }` is sent to `db_tx`
- The envelope has valid `op_id` (UUID v4 format)
- The envelope has valid `author` with id "local-user"
- The envelope has valid `timestamp` (current Unix epoch ms)

### test_ungroup_clears_selection_after_dispatch
**Given**: A diagram with selected subgraph "group-1"
**When**: User presses Ctrl+Shift+G and db_tx is available
**Then**:
- `editor_state.selected_items` is cleared after successful dispatch

### test_ungroup_with_single_selected_subgraph
**Given**: A diagram with a single selected Subgraph node
**When**: Ctrl+Shift+G is pressed
**Then**:
- One `EventEnvelope` with `DomainOp::Ungroup` containing that subgraph's ID is sent

---

## Error Path Tests

### test_ctrl_shift_g_with_empty_selection_returns_no_op
**Given**: A diagram with no selected items
**When**: User presses Ctrl+Shift+G
**Then**:
- Returns `Ok(false)` (no error, no event dispatched)
- No `EventEnvelope` is sent to `db_tx`

### test_ctrl_shift_g_with_non_subgraph_selection_returns_no_op
**Given**: A diagram with a selected regular node (not a subgraph)
**When**: User presses Ctrl+Shift+G
**Then**:
- Returns `Ok(false)` (no error, no event dispatched)
- No `EventEnvelope` is sent to `db_tx`

### test_ctrl_shift_g_when_db_tx_unavailable_falls_back_to_local_mutation
**Given**: A diagram with selected subgraph "group-1" and db_tx is None
**When**: User presses Ctrl+Shift+G
**Then**:
- Local `ungroup_selection` function is called as fallback
- Document is mutated directly (subgraph removed, children reparented)

### test_ctrl_shift_g_does_not_fire_when_editing_text
**Given**: User is typing in an input field
**When**: User presses Ctrl+Shift+G
**Then**:
- Key action is not triggered (handled by JS-side check)
- No `EventEnvelope` is sent

---

## Edge Case Tests

### test_ctrl_shift_g_multiple_times_sends_multiple_events
**Given**: A diagram with selected subgraph "group-1"
**When**: User presses Ctrl+Shift+G three times rapidly
**Then**:
- Three separate `EventEnvelope` messages are sent to `db_tx`
- Each with unique `op_id`

### test_ungroup_handles_missing_node_gracefully
**Given**: A selected item ID that doesn't exist in document.nodes
**When**: User presses Ctrl+Shift+G
**Then**:
- Returns `Ok(false)` (skips missing node, no error)

### test_ctrl_shift_g_with_multiple_selected_items_selects_first_subgraph
**Given**: A diagram with multiple selected items including a subgraph
**When**: User presses Ctrl+Shift+G
**Then**:
- Only one `DomainOp::Ungroup` is sent with the first valid subgraph ID

---

## Contract Verification Tests

### test_keyboard_mapping_ctrl_shift_g_returns_ungroup_action
**Given**: Key "g", ctrl=true, shift=true, is_editing_text=false
**When**: `map_key_to_action` is called
**Then**:
- Returns `KeyAction::Ungroup`

### test_keyboard_mapping_ctrl_g_returns_group_action
**Given**: Key "g", ctrl=true, shift=false, is_editing_text=false
**When**: `map_key_to_action` is called
**Then**:
- Returns `KeyAction::Group` (existing behavior unchanged)

### test_keyboard_mapping_ctrl_shift_g_ignored_when_editing
**Given**: Key "g", ctrl=true, shift=true, is_editing_text=true
**When**: `map_key_to_action` is called
**Then**:
- Returns `KeyAction::None`

---

## Contract Violation Tests

### test_violation_p3_empty_selection_returns_ok_false
**Given**: Empty selection `selected_items = {}`
**When**: `handle_ungroup_key` is called
**Then**: Returns `Ok(false)` -- NOT an error, just no-op

### test_violation_p3_non_subgraph_selection_returns_ok_false
**Given**: Selected item is a regular Node (not Subgraph kind)
**When**: `handle_ungroup_key` is called
**Then**: Returns `Ok(false)` -- NOT an error, cannot ungroup non-subgraph

### test_violation_q1_wrong_group_id_in_domain_op_returns_error
**Given**: Selected subgraph ID is "group-1" but wrong ID sent
**When**: Constructing `DomainOp::Ungroup` with wrong ID
**Then**: Should produce `Err(Error::InvalidNodeId)` if detected

### test_violation_q2_missing_timestamp_returns_error
**Given**: EventEnvelope being constructed
**When**: timestamp field is not set (None)
**Then**: Should produce `Err(Error::InvalidEnvelope)`

### test_violation_q4_selection_not_cleared_returns_error
**Given**: db_tx successfully sent the event
**When**: Selection is NOT cleared after dispatch
**Then**: Should produce `Err(Error::PostconditionViolation)`

---

## Given-When-Then Scenarios

### Scenario 1: Successful Ungroup via Keyboard
**Given**: 
- Diagram document with a subgraph node "group-1" containing child nodes "node-1" and "node-2"
- "group-1" is selected in `editor_state.selected_items`
- db_tx coroutine is available

**When**:
- User presses Ctrl+Shift+G

**Then**:
- `map_key_to_action("g", true, true, false)` returns `KeyAction::Ungroup`
- Canvas handler matches `KeyAction::Ungroup`
- Extracts "group-1" from selected_items
- Constructs `EventEnvelope` with `operation: DomainOp::Ungroup { id: "group-1" }`
- Sends envelope to `db_tx`
- Clears `selected_items`

### Scenario 2: No-op When No Group Selected
**Given**:
- Diagram document with regular nodes but no subgraphs
- No items selected

**When**:
- User presses Ctrl+Shift+G

**Then**:
- Returns `Ok(false)`
- No envelope sent to db_tx

### Scenario 3: Fallback to Local Mutation
**Given**:
- Diagram document with selected subgraph "group-1"
- db_tx is `None` (not available)

**When**:
- User presses Ctrl+Shift+G

**Then**:
- Calls `ungroup_selection(doc_signal, history_signal)` directly
- Mutates document: removes subgraph, reparents children
- Selection cleared

---

## Test Execution Notes

- All tests use the existing test infrastructure from `diagram_tool/src/`
- Mock `db_tx` using `tokio::sync::mpsc` channel
- Use existing `DiagramDocument` fixtures with Subgraph nodes
- Follow Martin Fowler's Given-When-Then naming convention
- Each test should be self-contained and idempotent

## Reference Implementation Patterns

See similar test files for pattern reference:
- `seshat-5y5/.beads/seshat-5y5/martin-fowler-tests.md` (NodeDelete)
- `seshat-5zs/.beads/seshat-5zs/martin-fowler-tests.md` (EdgeDisconnect)
- `diagram_tool/src/core/keyboard_tests.rs` (existing keyboard tests)
