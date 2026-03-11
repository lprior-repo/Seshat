# Martin Fowler Test Plan for seshat-4pr: UI Dispatch - Send to Back

## Happy Path Tests
- test_dispatch_send_to_back_constructs_valid_envelope
- test_dispatch_send_to_back_contains_all_selected_ids
- test_dispatch_send_to_back_has_valid_metadata
- test_dispatch_send_to_back_returns_true_when_dispatched
- test_send_to_back_action_calls_dispatch_after_apply

## Error Path Tests
- test_dispatch_send_to_back_returns_false_when_selection_empty
- test_dispatch_send_to_back_returns_error_when_db_tx_unavailable

## Edge Case Tests
- test_dispatch_send_to_back_handles_single_selected_node
- test_dispatch_send_to_back_handles_all_nodes_selected
- test_dispatch_send_to_back_with_locked_nodes_filtered

## Contract Verification Tests
- test_precondition_selection_not_empty_verified
- test_precondition_db_tx_available_verified
- test_postcondition_dispatches_to_db_tx_verified
- test_postcondition_contains_selected_ids_verified
- test_postcondition_has_valid_metadata_verified

## Contract Violation Tests
- `test_violation_p1_empty_selection_returns_ok_false`
  Given: Document with empty selection
  When: `dispatch_send_to_back(doc_signal, Some(db_tx))` is called
  Then: returns `Ok(false)` (no-op, not an error)

- `test_violation_p2_db_tx_unavailable_returns_error`
  Given: db_tx is None
  When: `dispatch_send_to_back(doc_signal, None)` is called
  Then: returns `Err(DispatchError::DbTxUnavailable)`

- `test_violation_q2_subset_of_ids_fails_verification`
  Given: Selected IDs are ["node1", "node2", "node3"]
  When: Dispatched DomainOp::SendToBack contains only ["node1"]
  Then: Test fails - expected all 3 IDs

- `test_violation_q3_missing_metadata_fails_verification`
  Given: Dispatched envelope
  When: op_id is empty string or timestamp is 0
  Then: Test fails - expected valid metadata

## Given-When-Then Scenarios

### Scenario 1: Toolbar button dispatches SendToBack with selection
Given: Document with nodes ["node1", "node2", "node3"] where all are selected
And: db_tx coroutine is available
When: User clicks "To Back" toolbar button
Then:
- The action calls `apply_send_to_back` to update local UI
- Then dispatches `EventEnvelope` with `DomainOp::SendToBack { ids: ["node1", "node2", "node3"] }` to db_tx
- Returns `Ok(true)`

### Scenario 2: Toolbar button does nothing when no selection
Given: Document with empty selection
When: User clicks "To Back" toolbar button
Then:
- Returns early without panicking
- Returns `Ok(false)` or equivalent no-op signal

### Scenario 3: Dispatch fails gracefully when db_tx unavailable
Given: db_tx context is None (async-db feature disabled)
When: User clicks "To Back" toolbar button
Then:
- Operation may still apply locally (UI update)
- Dispatch attempt returns `Err(DispatchError::DbTxUnavailable)` or is skipped

## Test Naming Conventions
All test names follow: `test_<category>_<specific_behavior>`

Categories:
- `happy_` - Happy path scenarios
- `error_` - Error handling scenarios
- `edge_` - Edge case scenarios
- `contract_` - Contract verification
- `violation_` - Contract violation reproduction

## Integration Points to Verify
1. `toolbar.rs` button onclick -> `actions::send_to_back` function
2. `actions::send_to_back` -> `apply_send_to_back` (local mutation)
3. `actions::send_to_back` -> `dispatch_send_to_back` (db_tx dispatch)
4. db_tx -> `store_bridge.append_event_sync` (persistence)

## Test Execution Notes
- Tests require `async-db` feature flag for full db_tx dispatch testing
- Without `async-db`, db_tx is None and dispatch tests verify graceful degradation
- Use mocked db_tx coroutine for unit testing the dispatch logic
