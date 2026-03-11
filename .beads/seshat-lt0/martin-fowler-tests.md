# Martin Fowler Test Plan: seshat-lt0 (UI Dispatch: Bring Forward)

## Test Categories
- Happy Path Tests
- Error Path Tests  
- Edge Case Tests
- Contract Verification Tests
- Contract Violation Tests
- Given-When-Then Scenarios

---

## Happy Path Tests

### test_bring_forward_dispatches_envelope_when_db_tx_available
**Given**: Document with 3 nodes (node-a, node-b, node-c), node-a and node-b selected, db_tx is Some(coroutine)
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- db_tx receives exactly one EventEnvelope
- envelope.operation equals DomainOp::BringForward { ids: ["node-a", "node-b"] }
- envelope.author.id equals "local-user"
- envelope.author.name equals "Local User"
- envelope.timestamp is within 1000ms of current time
- envelope.op_id is a valid UUID v4

### test_bring_forward_falls_back_to_direct_manipulation_when_db_tx_none
**Given**: Document with 2 nodes (node-x, node-y), both selected, db_tx is None
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Document is modified (z_index values change)
- No panic or error occurs
- Function returns true (success)

### test_bring_forward_updates_history_on_success
**Given**: Document with 1 node, that node selected, history_signal with empty history
**When**: bring_forward(doc_signal, history_signal) succeeds
**Then**:
- history_signal.read().can_undo() returns true
- Previous document state is preserved in history

### test_bring_forward_preserves_selection_after_dispatch
**Given**: Document with 2 selected nodes
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Selected items set contains the same node IDs after operation

---

## Error Path Tests

### test_bring_forward_returns_no_selection_error_when_empty_selection
**Given**: Document with selected_items = {} (empty)
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Returns Err(BringForwardError::NoSelection)
- No envelope is dispatched
- History is not modified

### test_bring_forward_returns_node_not_found_for_invalid_id
**Given**: Document with selected_items = {"fake-node-123"} but node doesn't exist
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Returns Err(BringForwardError::NodeNotFound("fake-node-123"))
- No envelope is dispatched

### test_bring_forward_handles_db_tx_unavailable_gracefully
**Given**: async-db feature enabled, but db_tx context is None
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Falls back to direct document manipulation
- Returns Ok(true) or Err depending on selection validity
- Does not panic

---

## Edge Case Tests

### test_bring_forward_handles_single_selected_node
**Given**: Document with 1 node selected
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Envelope is dispatched with ids: [single_node_id]
- Operation succeeds

### test_bring_forward_handles_all_nodes_selected
**Given**: Document with all nodes selected (e.g., 50 nodes)
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Envelope contains all 50 node IDs
- Operation completes without performance degradation

### test_bring_forward_handles_locked_nodes_filtered
**Given**: Document with node-a (locked=true), node-b (locked=false), both selected
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Only node-b is included in the envelope ids (locked nodes filtered)
- node-a remains at current z_index

### test_bring_forward_handles_subgraph_nodes_included
**Given**: Document with regular node and Subgraph node, both selected
**When**: bring_forward(doc_signal, history_signal) is called
**Then**:
- Both nodes included in envelope (Subgraph nodes not filtered even if locked)

---

## Contract Verification Tests

### test_precondition_p1_selection_exists
**Given**: doc_signal with empty selected_items
**When**: bring_forward is called
**Then**: Returns Err(NoSelection)

### test_precondition_p2_nodes_exist
**Given**: doc_signal with selected_items containing non-existent ID
**When**: bring_forward is called  
**Then**: Returns Err(NodeNotFound(...))

### test_postcondition_q1_envelope_dispatched
**Given**: db_tx is Some, document with valid selection
**When**: bring_forward succeeds
**Then**: db_tx received exactly one EventEnvelope

### test_postcondition_q2_correct_payload
**Given**: Selected node IDs ["a", "b", "c"]
**When**: bring_forward succeeds
**Then**: envelope.operation is DomainOp::BringForward { ids: ["a", "b", "c"] }

### test_postcondition_q3_author_populated
**Given**: Valid document and selection
**When**: bring_forward succeeds
**Then**: envelope.author.id = "local-user" AND envelope.author.name = "Local User"

### test_postcondition_q4_timestamp_valid
**Given**: Valid document and selection
**When**: bring_forward succeeds
**Then**: envelope.timestamp is within 1000ms of std::time::SystemTime::now()

### test_postcondition_q5_unique_op_id
**Given**: Valid document and selection
**When**: bring_forward called twice in quick succession
**Then**: Both envelopes have different op_id values (UUID v4 uniqueness)

### test_postcondition_q6_history_updated
**Given**: history_signal with empty history
**When**: bring_forward succeeds
**Then**: history_signal.read().can_undo() = true

### test_postcondition_q7_fallback_behavior
**Given**: db_tx is None, valid selection
**When**: bring_forward is called
**Then**: Document is modified via direct manipulation (not an error)

---

## Contract Violation Tests

### test_p1_violation_returns_no_selection_error
**Given**: Empty selected_items
**When**: bring_forward(doc_signal, history_signal)
**Then**: returns Err(BringForwardError::NoSelection) -- NOT a panic

### test_p2_violation_returns_node_not_found
**Given**: selected_items = {"non-existent-id"}
**When**: bring_forward(doc_signal, history_signal)
**Then**: returns Err(BringForwardError::NodeNotFound("non-existent-id")) -- NOT a panic

### test_q5_violation_duplicate_op_id
**Given**: Valid selection, db_tx available
**When**: Call bring_forward twice with same timestamp (simulate clock skew)
**Then**: Each call produces unique op_id via Uuid::new_v4() -- guaranteed unique

---

## Given-When-Then Scenarios

### Scenario 1: User clicks Bring Forward with valid selection
**Given**: 
- Document has nodes: node-1 (z:0), node-2 (z:1), node-3 (z:2)
- node-1 and node-2 are selected
- db_tx is available

**When**: User clicks "Forward" toolbar button

**Then**:
- EventEnvelope dispatched with DomainOp::BringForward { ids: ["node-1", "node-2"] }
- History updated with pre-operation state
- After store replay: node-1 (z:1), node-2 (z:2), node-3 (z:0)

### Scenario 2: User clicks Bring Forward with no selection
**Given**: 
- Document has 3 nodes, none selected
- Toolbar button is disabled (should not be clickable)

**When**: User somehow invokes bring_forward

**Then**:
- Returns Err(NoSelection)
- No state change
- Toast/feedback shown to user

### Scenario 3: User clicks Bring Forward, db unavailable (WASM)
**Given**:
- Building for wasm32 target (no async-db)
- Document has 2 nodes selected

**When**: User clicks "Forward"

**Then**:
- Falls back to direct document manipulation
- Nodes z_order updated immediately in memory
- No persistence (expected for WASM)

---

## Traceability Matrix

| Contract Clause | Test(s) Covering |
|-----------------|------------------|
| P1 | test_bring_forward_returns_no_selection_error_when_empty_selection, test_precondition_p1_selection_exists, test_p1_violation_returns_no_selection_error |
| P2 | test_bring_forward_returns_node_not_found_for_invalid_id, test_precondition_p2_nodes_exist, test_p2_violation_returns_node_not_found |
| P3 | test_bring_forward_handles_db_tx_unavailable_gracefully |
| Q1 | test_bring_forward_dispatches_envelope_when_db_tx_available, test_postcondition_q1_envelope_dispatched |
| Q2 | test_bring_forward_dispatches_envelope_when_db_tx_available, test_postcondition_q2_correct_payload |
| Q3 | test_bring_forward_dispatches_envelope_when_db_tx_available, test_postcondition_q3_author_populated |
| Q4 | test_postcondition_q4_timestamp_valid |
| Q5 | test_bring_forward_dispatches_envelope_when_db_tx_available, test_postcondition_q5_unique_op_id, test_q5_violation_duplicate_op_id |
| Q6 | test_bring_forward_updates_history_on_success, test_postcondition_q6_history_updated |
| Q7 | test_bring_forward_falls_back_to_direct_manipulation_when_db_tx_none, test_postcondition_q7_fallback_behavior |
| I1 | Covered by integration test with store |
| I2 | test_bring_forward_preserves_selection_after_dispatch |
| I3 | Covered by store replay integration test |
