# Martin Fowler Test Plan: seshat-5y5 (UI Dispatch: Backspace Key)

## Overview
This test plan validates the Backspace/Delete key handler that constructs `DomainOp::NodeDelete` for selected nodes and dispatches to `db_tx` (event sourcing path).

## Happy Path Tests

### test_delete_key_dispatches_node_delete_for_single_selected_node
**Given**: A document with one node "node-1" selected in `editor_state.selected_items`
**When**: User presses the Delete key
**Then**: 
- Returns `Ok(true)` indicating deletion occurred
- One `EventEnvelope` with `DomainOp::NodeDelete { id: "node-1" }` is sent to db_tx
- The EventEnvelope has valid op_id (UUID), author, and timestamp

### test_backspace_key_dispatches_node_delete_for_single_selected_node
**Given**: A document with one node "node-1" selected in `editor_state.selected_items`
**When**: User presses the Backspace key
**Then**: 
- Returns `Ok(true)` indicating deletion occurred
- One `EventEnvelope` with `DomainOp::NodeDelete { id: "node-1" }` is sent to db_tx

### test_delete_key_dispatches_node_delete_for_multiple_selected_nodes
**Given**: A document with nodes "node-1", "node-2", "node-3" selected in `editor_state.selected_items`
**When**: User presses the Delete key
**Then**: 
- Returns `Ok(true)` indicating deletion occurred
- Three `EventEnvelope` messages are sent to db_tx, one for each node ID
- Each envelope contains `DomainOp::NodeDelete` with the corresponding node ID

### test_delete_key_clears_selection_after_dispatch
**Given**: A document with node "node-1" selected
**When**: User presses the Delete key and db_tx is available
**Then**: 
- After dispatch, `doc_signal.read().editor_state.selected_items` is empty

### test_delete_key_creates_valid_event_envelope
**Given**: A document with node "test-node" selected
**When**: User presses the Delete key
**Then**: 
- The dispatched EventEnvelope has:
  - `op_id`: Valid UUID v4 string (can parse with `Uuid::parse_str`)
  - `operation`: `DomainOp::NodeDelete { id: "test-node" }`
  - `author.id`: "local-user"
  - `author.name`: "Local User"
  - `author.email`: `None`
  - `timestamp`: Positive i64 (Unix epoch ms)

## Error Path Tests

### test_delete_key_returns_false_when_no_selection
**Given**: A document with empty `editor_state.selected_items`
**When**: User presses the Delete key
**Then**: 
- Returns `Ok(false)` (no-op, not an error)
- No messages are sent to db_tx

### test_backspace_key_returns_false_when_no_selection
**Given**: A document with empty `editor_state.selected_items`
**When**: User presses the Backspace key
**Then**: 
- Returns `Ok(false)` (no-op, not an error)

### test_delete_key_handles_nonexistent_selected_id_gracefully
**Given**: A document where `selected_items` contains "ghost-node" (not in nodes)
**When**: User presses the Delete key
**Then**: 
- The nonexistent ID is skipped
- If other valid nodes exist, events are sent for those
- If no valid nodes exist, returns `Ok(false)`

### test_delete_key_falls_back_to_local_mutation_when_db_tx_none
**Given**: A document with node "node-1" selected, and db_tx is `None`
**When**: User presses the Delete key
**Then**: 
- Falls back to local mutation (calls existing apply_delete_selected or equivalent)
- Returns `Ok(true)` if deletion succeeded
- Document nodes are modified directly

## Edge Case Tests

### test_delete_key_handles_empty_document_nodes
**Given**: A document with no nodes in `document.nodes`
**When**: User presses the Delete key
**Then**: 
- Returns `Ok(false)` (nothing to delete)

### test_delete_key_handles_mixed_node_and_edge_selection
**Given**: A document with node "node-1" and edge "edge-1" both in selected_items
**When**: User presses the Delete key
**Then**: 
- Only node "node-1" deletion is dispatched (DomainOp::NodeDelete)
- Edge "edge-1" is ignored at this layer (handled by cascade after node deletion)

### test_delete_key_idempotent_dispatch
**Given**: A document with node "node-1" selected
**When**: User presses Delete key multiple times rapidly
**Then**: 
- Each keypress dispatches a new EventEnvelope
- No deduplication at this layer (idempotent dispatch invariant)

### test_delete_key_non_blocking
**Given**: A document with node "node-1" selected, db_tx is available but slow
**When**: User presses the Delete key
**Then**: 
- Handler returns immediately without waiting for db_tx
- The async send is handled by the spawned task

## Contract Verification Tests

### test_precondition_p1_key_must_be_delete_or_backspace
**Given**: Any document state
**When**: A key event with key "Escape" is processed
**Then**: 
- The delete handler is NOT triggered (match statement doesn't match)

### test_precondition_p3_has_selected_nodes
**Given**: A document with empty selected_items
**When**: Delete key handler is invoked
**Then**: 
- Returns `Ok(false)` without sending to db_tx

### test_postcondition_q1_one_event_per_node
**Given**: A document with nodes ["a", "b", "c"] selected
**When**: Delete key is pressed
**Then**: 
- Exactly 3 EventEnvelope messages are sent to db_tx

### test_postcondition_q2_event_envelope_structure
**Given**: Any valid deletion scenario
**When**: EventEnvelope is constructed
**Then**: 
- All required fields are present and valid

### test_postcondition_q4_selection_cleared
**Given**: A document with selection
**When**: Delete key successfully dispatches
**Then**: 
- selected_items is empty after the operation

### test_invariant_i3_no_panic_on_missing_node
**Given**: Selected items contain IDs not in nodes
**When**: Delete key is pressed
**Then**: 
- No panic occurs
- Valid node IDs are processed, invalid are skipped

## Contract Violation Tests

### test_violation_p3_empty_selection_should_not_error
**Given**: `selected_items = {}` (empty)
**When**: `handle_delete_key(...)` is called
**Then**: Returns `Ok(false)` -- NOT `Err(Error::NoSelection)` -- NOT a panic

### test_violation_q1_wrong_id_should_error
**Given**: Selected node "node-1" exists
**When**: EventEnvelope is sent with wrong ID "wrong-id"
**Then**: Returns `Err(Error::InvalidNodeId)` -- NOT accepted silently

### test_violation_q4_selection_not_cleared_should_error
**Given**: After successful dispatch, selection is not cleared
**When**: Postcondition Q4 is checked
**Then**: Returns `Err(Error::PostconditionViolation)` -- NOT silently accepted

## Given-When-Then Scenarios

### Scenario 1: Single Node Deletion via Delete Key
**Given**: A DiagramDocument with:
- `document.nodes` containing "node-1" -> Node
- `editor_state.selected_items` = {"node-1"}
- `db_tx` = Some(coroutine)

**When**: User presses Delete key

**Then**:
- `handle_delete_key` returns `Ok(true)`
- db_tx receives exactly one `EventEnvelope` with `operation: DomainOp::NodeDelete { id: "node-1" }`
- `editor_state.selected_items` is empty after operation

### Scenario 2: Multiple Node Deletion
**Given**: A DiagramDocument with nodes "node-1", "node-2" selected

**When**: User presses Backspace key

**Then**:
- `handle_delete_key` returns `Ok(true)`
- db_tx receives two EventEnvelopes, one for each node
- Both envelopes have the correct node IDs

### Scenario 3: No Selection (No-Op)
**Given**: A DiagramDocument with empty selected_items

**When**: User presses Delete key

**Then**:
- Returns `Ok(false)` (no-op)
- No messages sent to db_tx

### Scenario 4: Fallback to Local Mutation
**Given**: A DiagramDocument with node selected, but db_tx = None

**When**: User presses Delete key

**Then**:
- Falls back to direct document mutation
- Node is removed from document.nodes

## Implementation Phases

### Phase 1: Create handle_delete_key Function
1. Add new function `handle_delete_key(doc_signal, history_signal, db_tx) -> Result<bool, Error>` in `diagram_tool/src/ui/commands.rs`
2. Extract selected node IDs from `doc_signal.read().editor_state.selected_items`
3. Filter to only include IDs that exist in `doc_signal.read().document.nodes`
4. If no valid nodes, return `Ok(false)`

### Phase 2: Implement Event Dispatch
1. For each selected node ID, construct `EventEnvelope`:
   - `op_id`: `Uuid::new_v4().to_string()`
   - `operation`: `DomainOp::NodeDelete { id: node_id }`
   - `author`: `Author { id: "local-user".to_string(), name: "Local User".to_string(), email: None }`
   - `timestamp`: `SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64`
2. Send via `db_tx.send(envelope)` if db_tx is Some

### Phase 3: Update Canvas Key Handler
1. In `canvas.rs` lines 787-788, replace `apply_delete_selected(doc_signal, history_signal)` with `handle_delete_key(doc_signal, history_signal, db_tx.clone())`
2. Import the new function

### Phase 4: Add Fallback
1. If db_tx is None, call existing `apply_delete_selected(doc_signal, history_signal)` as fallback
2. Return its result

## Traceability Matrix

| Contract Clause | Test Case |
|-----------------|-----------|
| P1: Key is Delete/Backspace | test_precondition_p1_key_must_be_delete_or_backspace |
| P3: Has selected nodes | test_precondition_p3_has_selected_nodes, test_delete_key_returns_false_when_no_selection |
| Q1: Event dispatched | test_delete_key_dispatches_node_delete_for_single_selected_node |
| Q2: EventEnvelope valid | test_delete_key_creates_valid_event_envelope |
| Q4: Selection cleared | test_delete_key_clears_selection_after_dispatch |
| I3: No panic on missing | test_invariant_i3_no_panic_on_missing_node |
| Fallback path | test_delete_key_falls_back_to_local_mutation_when_db_tx_none |
