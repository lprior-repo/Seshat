# Martin Fowler Test Plan for seshat-shb: UI Dispatch - Z-Index Layering

## Overview

This test plan validates the complete toolbar action pipeline for z-index layering operations (BringToFront, SendToBack). The tests verify behavior from user intent through to persistence.

**Pipeline Flow**: `toolbar button click` → `action handler` → `dispatch function` → `db_tx channel` → `store bridge` → `WAL`

## Happy Path Tests (Behavior-Driven Names)

### BringToFront Operations
- `test_when_user_clicks_to_front_with_selection_then_envelope_contains_selected_ids`
  - Given: Selected node IDs ["node1", "node2", "node3"]
  - When: User clicks "To Front" toolbar button
  - Then: EventEnvelope contains DomainOp::BringToFront with all 3 node IDs

- `test_when_user_clicks_to_front_with_selection_then_local_z_order_is_updated`
  - Given: Document with nodes in order [A, B, C, D, E], nodes B and D selected
  - When: User clicks "To Front" toolbar button
  - Then: Local document state updates to [A, C, E, B, D]

- `test_when_user_clicks_to_front_with_selection_then_dispatch_result_shows_correct_counts`
  - Given: Selected node IDs ["node1", "node2", "node3"]
  - When: dispatch_bring_to_front is called with valid db_tx
  - Then: Returns Ok(DispatchResult { nodes_affected: 3, dispatches_sent: 1 })

- `test_when_user_clicks_to_front_with_selection_then_envelope_has_valid_metadata`
  - Verifies op_id is valid UUID, author is non-empty, timestamp is positive

### SendToBack Operations
- `test_when_user_clicks_to_back_with_selection_then_envelope_contains_selected_ids`
  - Given: Selected node IDs ["node1", "node2"]
  - When: User clicks "To Back" toolbar button
  - Then: DomainOp::SendToBack contains both node IDs

- `test_when_user_clicks_to_back_with_selection_then_local_z_order_is_updated`
  - Given: Document with nodes in order [A, B, C, D, E], nodes B and D selected
  - When: User clicks "To Back" toolbar button
  - Then: Local document state updates to [B, D, A, C, E]

- `test_when_user_clicks_to_back_with_selection_then_dispatch_result_shows_correct_counts`
  - Given: Selected node IDs ["node1", "node2"]
  - When: dispatch_send_to_back is called with valid db_tx
  - Then: Returns Ok(DispatchResult { nodes_affected: 2, dispatches_sent: 1 })

- `test_when_user_clicks_to_back_with_selection_then_envelope_has_valid_metadata`
  - Verifies op_id is valid UUID, author is non-empty, timestamp is positive

### End-to-End Pipeline Tests (DEFECT-003)

- `test_e2e_bring_to_front_full_pipeline_with_real_db_tx_channel`
  - **Setup**: Create DiagramDocument with 5 nodes, select 3 nodes
  - **Action**: Call toolbar action bring_to_front(doc_signal, db_tx_channel)
  - **Verify**:
    1. Envelope is sent to db_tx channel (receive on test channel)
    2. Envelope contains DomainOp::BringToFront with correct node IDs
    3. Local document state reflects new z-order
    4. DispatchResult returns correct counts

- `test_e2e_send_to_back_full_pipeline_with_real_db_tx_channel`
  - **Setup**: Create DiagramDocument with 5 nodes, select 2 nodes
  - **Action**: Call toolbar action send_to_back(doc_signal, db_tx_channel)
  - **Verify**:
    1. Envelope is sent to db_tx channel
    2. Envelope contains DomainOp::SendToBack with correct node IDs
    3. Local document state reflects new z-order

- `test_e2e_toolbar_button_triggers_complete_dispatch_to_store_bridge`
  - **Setup**: Diagram with nodes, mock store bridge connected to test WAL
  - **Action**: Simulate toolbar button click
  - **Verify**:
    1. Event persisted to WAL via store bridge
    2. WAL contains valid sequence number
    3. Transaction committed successfully

## Error Path Tests

- `test_given_db_tx_unavailable_when_bring_to_front_then_returns_wal_disconnected_error`
  - Given: db_tx channel is None (WAL disconnected or async-db feature disabled)
  - When: dispatch_bring_to_front(&None, &["node1"]) is called
  - Then: Returns Err(DispatchError::WalDisconnected)

- `test_given_db_tx_unavailable_when_send_to_back_then_returns_wal_disconnected_error`
  - Given: db_tx channel is None
  - When: dispatch_send_to_back(&None, &["node1"]) is called
  - Then: Returns Err(DispatchError::WalDisconnected)

- `test_given_db_tx_unavailable_when_toolbar_action_then_local_mutation_still_applies`
  - Given: db_tx is None
  - When: Toolbar bring_to_front action is called
  - Then: Local document state updates (optimistic UI), no panic

- `test_given_db_tx_unavailable_when_toolbar_action_then_error_is_logged`
  - Given: db_tx is None
  - When: Toolbar action executes
  - Then: Error is logged at appropriate level, UI remains responsive

## Edge Case Tests (DEFECT-004)

- `test_edge_single_node_selection_bring_to_front_returns_correct_counts`
  - Given: Single node selected ["node1"]
  - When: dispatch_bring_to_front is called
  - Then: Returns Ok(DispatchResult { nodes_affected: 1, dispatches_sent: 1 })

- `test_edge_single_node_selection_send_to_back_returns_correct_counts`
  - Given: Single node selected ["node1"]
  - When: dispatch_send_to_back is called
  - Then: Returns Ok(DispatchResult { nodes_affected: 1, dispatches_sent: 1 })

- `test_edge_many_selected_nodes_all_included_no_truncation`
  - Given: 10+ nodes selected
  - When: dispatch is called
  - Then: All IDs included in DomainOp, no truncation

- `test_edge_rapid_successive_bring_to_front_clicks`
  - Given: Document with nodes, rapid successive clicks on "To Front"
  - When: 5 rapid clicks within 100ms
  - Then: Each click produces a valid envelope, z-order updates correctly, no race conditions
  - **Implementation**: Use spawn_local with proper ordering, verify final state

- `test_edge_db_tx_channel_closed_returns_error`
  - Given: db_tx channel that has been closed (sender dropped)
  - When: dispatch_bring_to_front is called
  - Then: Returns Err(DispatchError::WalDisconnected) or SendError

- `test_edge_concurrent_bring_to_front_and_send_to_back`
  - Given: Document with nodes [A, B, C, D, E], nodes A and E selected
  - When: BringToFront and SendToBack dispatched concurrently
  - Then: Final z-order is deterministic (depends on operation order), no panic

- `test_edge_bring_to_front_with_locked_nodes_filtered`
  - Given: Selected nodes include locked nodes
  - When: dispatch_bring_to_front is called
  - Then: Only unlocked nodes are dispatched (if filtering is implemented)

## Contract Verification Tests (DEFECT-005)

### Preconditions
- `test_contract_precondition_selection_not_empty_handled_as_noop`
  - Verifies P1: dispatch returns no-op when node_ids is empty
  - Then: Returns Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })

- `test_contract_precondition_db_tx_available_required_for_dispatch`
  - Verifies P2: dispatch returns error when db_tx is None

### Postconditions
- `test_contract_postcondition_bring_to_front_dispatches_to_db_tx`
  - Verifies Q1: EventEnvelope with DomainOp::BringToFront is sent to db_tx

- `test_contract_postcondition_send_to_back_dispatches_to_db_tx`
  - Verifies Q2: EventEnvelope with DomainOp::SendToBack is sent to db_tx

- `test_contract_postcondition_dispatch_contains_all_selected_ids`
  - Verifies Q3: Dispatched operation contains all selected node IDs

- `test_contract_postcondition_dispatch_has_valid_metadata`
  - Verifies Q4: Dispatched envelope has valid op_id, author, timestamp

- `test_contract_postcondition_empty_selection_returns_noop_without_panic`
  - **DEFECT-005**: Verifies Q5: Empty selection returns DispatchResult { nodes_affected: 0, dispatches_sent: 0 }
  - Given: Empty node_ids slice &[]
  - When: dispatch_bring_to_front is called
  - Then: Returns Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })

## Contract Violation Tests

### Precondition Violations
- `test_violation_empty_ids_returns_ok_with_zero_counts_not_error`
  - Given: node_ids is empty slice `&[]`
  - When: dispatch_bring_to_front(&Some(tx), &[]) is called
  - Then: Returns Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })
  - **Note**: This is NOT an error - it's a no-op (defines P1 as soft precondition)

- `test_violation_db_tx_none_returns_wal_disconnected`
  - Given: db_tx is None
  - When: dispatch_bring_to_front(&None, &["node1"]) is called
  - Then: Returns Err(DispatchError::WalDisconnected)

### Postcondition Violations
- `test_violation_subset_of_ids_fails_verification`
  - Given: Selected IDs are ["node1", "node2", "node3"]
  - When: Dispatched DomainOp::BringToFront contains only ["node1"]
  - Then: Test fails - expected all 3 IDs

- `test_violation_missing_op_id_fails_verification`
  - Given: Dispatched envelope
  - When: op_id is empty string
  - Then: Test fails - expected valid UUID

- `test_violation_zero_timestamp_fails_verification`
  - Given: Dispatched envelope
  - When: timestamp is 0
  - Then: Test fails - expected positive timestamp

## Property-Based Tests (DEFECT-006)

- `test_property_bring_to_front_idempotent_multiple_calls`
  - Given: Any valid set of selected node IDs
  - When: bring_to_front is called twice
  - Then: Second call is a no-op (nodes already at front)

- `test_property_send_to_back_idempotent_multiple_calls`
  - Given: Any valid set of selected node IDs
  - When: send_to_back is called twice
  - Then: Second call is a no-op (nodes already at back)

- `test_property_z_order_preserves_relative_order_of_selected_nodes`
  - Given: Document with N nodes, K selected nodes (K < N)
  - When: bring_to_front is applied
  - Then: Relative order of selected nodes is preserved

- `test_property_dispatch_result_nodes_affected_equals_input_count`
  - Given: Any valid node ID list of length N
  - When: dispatch is called with those IDs
  - Then: DispatchResult.nodes_affected == N

- `test_property_empty_selection_always_returns_zero_counts`
  - Given: Any document state
  - When: dispatch is called with empty selection
  - Then: Always returns DispatchResult { nodes_affected: 0, dispatches_sent: 0 }

## Given-When-Then Scenarios

### Scenario 1: BringToFront toolbar button with selection
**ID**: GWT-BTF-001
- **Given**: Document with nodes ["node1", "node2", "node3"] where all are selected
- **And**: db_tx coroutine is available
- **When**: User clicks "To Front" toolbar button
- **Then**:
  - The action calls `apply_bring_to_front` to update local UI (optimistic update)
  - Then dispatches `EventEnvelope` with `DomainOp::BringToFront { ids: [NodeId("node1"), NodeId("node2"), NodeId("node3")] }` to db_tx
  - Returns Ok(DispatchResult { nodes_affected: 3, dispatches_sent: 1 })

### Scenario 2: SendToBack toolbar button with selection
**ID**: GWT-STB-001
- **Given**: Document with nodes ["node1", "node2", "node3"] where all are selected
- **And**: db_tx coroutine is available
- **When**: User clicks "To Back" toolbar button
- **Then**:
  - The action calls `apply_send_to_back` to update local UI
  - Then dispatches `EventEnvelope` with `DomainOp::SendToBack { ids: [NodeId("node1"), NodeId("node2"), NodeId("node3")] }` to db_tx
  - Returns Ok(DispatchResult { nodes_affected: 3, dispatches_sent: 1 })

### Scenario 3: Toolbar button does nothing when no selection
**ID**: GWT-EMPTY-001
- **Given**: Document with empty selection (editor_state.selected_items is empty)
- **When**: User clicks "To Front" or "To Back" toolbar button
- **Then**:
  - Returns early without panicking
  - Returns Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })
  - No dispatch sent to db_tx

### Scenario 4: Dispatch fails gracefully when db_tx unavailable
**ID**: GWT-DISCONNECT-001
- **Given**: db_tx is None (WAL disconnected or async-db feature disabled)
- **When**: User clicks "To Front" toolbar button
- **Then**:
  - Local mutation still applies (optimistic UI update)
  - Dispatch attempt returns Err(DispatchError::WalDisconnected)
  - No panic, UI remains responsive

### Scenario 5: BringToFront preserves relative z-order of selected nodes
**ID**: GWT-ZORDER-001
- **Given**: Document with nodes in order [A, B, C, D, E]
- **And**: Nodes B and D are selected
- **When**: User clicks "To Front"
- **Then**:
  - Selected nodes B and D move to front
  - New order: [A, C, E, B, D]
  - Relative order of selected nodes (B before D) is preserved

### Scenario 6: SendToBack preserves relative z-order of selected nodes
**ID**: GWT-ZORDER-002
- **Given**: Document with nodes in order [A, B, C, D, E]
- **And**: Nodes B and D are selected
- **When**: User clicks "To Back"
- **Then**:
  - Selected nodes B and D move to back
  - New order: [B, D, A, C, E]
  - Relative order of selected nodes (B before D) is preserved

## Integration Points to Verify

1. **Toolbar UI**:
   - `toolbar.rs` button onclick -> `actions::bring_to_front`
   - `toolbar.rs` button onclick -> `actions::send_to_back`

2. **Action Layer**:
   - `actions::bring_to_front` -> extracts selected_ids from doc_signal
   - `actions::bring_to_front` -> calls `apply_bring_to_front` (local mutation)
   - `actions::bring_to_front` -> calls `dispatch_bring_to_front` (db_tx dispatch)
   - Same pattern for `send_to_back`

3. **Dispatch Layer**:
   - `dispatch_bring_to_front` -> calls `create_bring_to_front_envelope`
   - `dispatch_send_to_back` -> calls `create_send_to_back_envelope`
   - Both call `db_tx.send(envelope)`

4. **Persistence Layer**:
   - db_tx -> store_bridge.append_event_sync (if sync) or async variant
   - Event persisted to WAL

## Test Naming Conventions

All test names follow behavior-driven format: `test_<context>_<behavior>_<outcome>`

Categories:
- `when_user_` - User-initiated happy path
- `given_` - Precondition setup in tests
- `e2e_` - End-to-end integration tests
- `error_` - Error handling scenarios
- `edge_` - Edge case scenarios
- `contract_` - Contract verification
- `violation_` - Contract violation reproduction
- `property_` - Property-based tests

## Test Execution Notes (DEFECT-001)

- **Integration tests use real db_tx**: Tests should use a real MPSC channel or test channel instead of mocked db_tx
- **Setup**: Create a test channel with `channel::<EventEnvelope>()` and pass sender to dispatch
- **Verification**: Use `try_recv()` or `blocking_recv()` to verify envelope was sent
- **Without async-db feature**: db_tx is None and dispatch tests verify graceful degradation
- **Z-order logic tests**: Verify relative ordering is preserved after operations

## Mutation Testing Consideration (DEFECT-007)

Consider running mutation testing (e.g., with `mutagen` or similar) on the dispatch logic to verify test robustness:

- Mutations to filter logic should be caught by edge case tests
- Mutations to z-order calculation should be caught by property tests
- Mutations to error handling should be caught by error path tests

Ensure test suite has high mutation score to validate specification completeness.

## Defect Resolution Summary

| Defect ID | Description | Resolution |
|-----------|-------------|------------|
| DEFECT-001 | Remove mocking, use real db_tx | Added E2E tests with real MPSC channels |
| DEFECT-002 | Behavior-driven test names | Renamed all tests to express WHAT not HOW |
| DEFECT-003 | Add actual E2E implementation | Added e2e_ prefixed tests with full pipeline |
| DEFECT-004 | Add edge case tests | Added rapid clicks, closed channel, concurrent tests |
| DEFECT-005 | Verify Q5 fully | Added explicit empty selection test with zero counts |
| DEFECT-006 | Property-based tests | Added property_ prefixed invariant tests |
| DEFECT-007 | Mutation testing | Added mutation testing consideration section |

(End of file)
