# Martin Fowler Test Plan: seshat-3ee

## Overview

This test plan follows Martin Fowler's Given-When-Then (GWT) pattern with expressive test names that describe behavior. Tests are organized into happy path, error path, edge case, and contract verification categories.

## Happy Path Tests

### test_dispatch_delete_sends_nodedelete_for_single_selected_node

**Given**: A document with one selected node "node-1" and db_tx available  
**When**: `dispatch_delete_to_backend(doc_signal, db_tx)` is called  
**Then**:
- Returns `Ok(DispatchResult { nodes_deleted: 1, dispatches_sent: 1 })`
- Exactly one `EventEnvelope` with `DomainOp::NodeDelete { id: "node-1" }` is sent to db_tx

### test_dispatch_delete_sends_nodedelete_for_multiple_selected_nodes

**Given**: A document with three selected nodes ["node-1", "node-2", "node-3"] and db_tx available  
**When**: `dispatch_delete_to_backend(doc_signal, db_tx)` is called  
**Then**:
- Returns `Ok(DispatchResult { nodes_deleted: 3, dispatches_sent: 3 })`
- Exactly three `EventEnvelope` messages are sent, each with unique `DomainOp::NodeDelete`

### test_dispatch_delete_envelope_contains_valid_metadata

**Given**: A document with one selected node and db_tx available  
**When**: `dispatch_delete_to_backend(doc_signal, db_tx)` is called  
**Then**:
- Each sent `EventEnvelope` has:
  - Valid UUID `op_id` (parseable via `Uuid::parse_str`)
  - `author.id` = "local-user"
  - `author.name` = "Local User"
  - `timestamp` > 0 (Unix milliseconds)

### test_dispatch_delete_with_existing_apply_delete_integration

**Given**: A document with selected nodes and db_tx available  
**When**: Delete key handler triggers both local delete and backend dispatch  
**Then**:
- Local document state reflects node removal (verified via `doc.document.nodes`)
- Backend receives NodeDelete envelopes for each deleted node

## Error Path Tests

### test_dispatch_delete_returns_no_selection_error_when_selection_empty

**Given**: A document with empty `selected_items` and db_tx available  
**When**: `dispatch_delete_to_backend(doc_signal, db_tx)` is called  
**Then**:
- Returns `Err(Error::NoSelection)`
- No `EventEnvelope` is sent to db_tx

### test_dispatch_delete_handles_none_db_tx_gracefully

**Given**: A document with one selected node and `db_tx = None`  
**When**: `dispatch_delete_to_backend(doc_signal, None)` is called  
**Then**:
- Returns `Ok(DispatchResult { nodes_deleted: 1, dispatches_sent: 0 })`
- Local delete still occurs (document is mutated)
- Warning is logged about unavailable db_tx

### test_dispatch_delete_handles_send_failure

**Given**: A document with one selected node and db_tx that fails on send  
**When**: `dispatch_delete_to_backend(doc_signal, failing_db_tx)` is called  
**Then**:
- Returns `Err(Error::SendFailed("channel closed"))`
- Local document may or may not be mutated depending on implementation choice

## Edge Case Tests

### test_dispatch_delete_handles_already_deleted_node_in_selection

**Given**: A document with selected items containing "node-1" but node not in document.nodes  
**When**: `dispatch_delete_to_backend(doc_signal, db_tx)` is called  
**Then**:
- Skips non-existent node
- Only sends NodeDelete for nodes that exist in document.nodes
- Returns count of actually deleted nodes

### test_dispatch_delete_with_mixed_node_and_edge_selection

**Given**: A document with selected items containing both nodes and edges  
**When**: `dispatch_delete_to_backend(doc_signal, db_tx)` is called  
**Then**:
- Only sends NodeDelete for node IDs
- Ignores edge IDs in selection

### test_dispatch_delete_concurrent_with_document_modification

**Given**: A document being modified concurrently (theoretical)  
**When**: Delete dispatch captures selection at call time  
**Then**:
- Uses snapshot of selection at dispatch time
- Does not include nodes added after dispatch call

## Contract Verification Tests

### test_precondition_p2_selection_not_empty

**Given**: Empty selection  
**When**: Delete dispatch is invoked  
**Then**: Returns `Err(Error::NoSelection)` (NOT a panic)

### test_precondition_p3_db_tx_availability

**Given**: `db_tx = None`  
**When**: Delete dispatch is invoked with non-empty selection  
**Then**: Returns `Ok` with `dispatches_sent = 0` (graceful degradation)

### test_postcondition_q1_exact_dispatch_count

**Given**: 5 selected nodes  
**When**: Delete dispatch is invoked  
**Then**: Returns `Ok(DispatchResult { nodes_deleted: 5, dispatches_sent: 5 })`

### test_postcondition_q2_valid_envelope_metadata

**Given**: Valid document and db_tx  
**When**: Delete dispatch is invoked  
**Then**: All sent envelopes have valid `op_id`, `author`, and `timestamp` fields

### test_invariant_i1_dispatch_equals_selection

**Given**: Selection contains nodes [A, B, C]  
**When**: Delete dispatch completes  
**Then**: Dispatched NodeDelete IDs exactly match {A, B, C}

### test_invariant_i2_send_failure_logged

**Given**: db_tx that fails on send  
**When**: Delete dispatch is invoked  
**Then**: Error is logged, no panic occurs

## Contract Violation Tests

These tests verify that the contract's violation examples from contract.md are properly handled:

### test_violation_p2_no_selection_returns_no_selection_error

**Given**: Empty `selected_items`  
**When**: `dispatch_delete_to_backend(doc_signal, db_tx)` is called  
**Then**: Returns `Err(Error::NoSelection)` -- NOT a panic, NOT an unwrap failure

**Contract Reference**: VIOLATES P2 from contract.md

### test_violation_q1_under_dispatch_fails

**Given**: 3 selected nodes, but only 2 envelopes can be sent (simulated failure on 3rd)  
**When**: `dispatch_delete_to_backend(doc_signal, db_tx)` is called  
**Then**: Returns `Err(Error::DispatchIncomplete)` -- partial dispatch is rejected

**Contract Reference**: VIOLATES Q1 (Under-dispatch) from contract.md

### test_violation_q1_over_dispatch_fails

**Given**: This scenario should be impossible by construction (envelope count equals selection count)  
**When**: Implementation attempts to send extra envelopes  
**Then**: Returns `Err(Error::DispatchIncomplete)` if over-dispatch detected

**Contract Reference**: VIOLATES Q1 (Over-dispatch) from contract.md

## Given-When-Then Scenarios

### Scenario 1: User presses Delete key with nodes selected

**Given**:
- User has selected one or more nodes in the diagram
- db_tx coroutine is available and functional

**When**:
- User presses Delete or Backspace key
- Key handler invokes `dispatch_delete_to_backend`

**Then**:
- Each selected node ID is sent as `DomainOp::NodeDelete` to db_tx
- Local document state is updated (nodes removed)
- Selection is cleared
- Revision is incremented

### Scenario 2: User presses Delete key with no selection

**Given**:
- User has no nodes selected (selection is empty)
- db_tx is available

**When**:
- User presses Delete key

**Then**:
- No dispatch occurs
- Local document is unchanged
- Returns `Err(Error::NoSelection)` or equivalent (handled silently by UI)

### Scenario 3: Delete key during text editing is ignored

**Given**:
- User is editing text in an input field

**When**:
- User presses Delete key

**Then**:
- Key handler returns early (handled by existing code)
- No delete dispatch occurs

### Scenario 4: Delete with db_tx unavailable (first load)

**Given**:
- Document has selected nodes
- db_tx context is None (not yet initialized)

**When**:
- User presses Delete key

**Then**:
- Local delete still succeeds
- Warning logged about unavailable db_tx
- UI remains functional

## Implementation Phases

| Phase | Description | Tests |
|-------|-------------|-------|
| Phase 1 | Wire db_tx into delete handler in canvas.rs | test_dispatch_delete_sends_nodedelete_* |
| Phase 2 | Add envelope construction with proper metadata | test_dispatch_delete_envelope_contains_valid_metadata |
| Phase 3 | Handle empty selection case | test_dispatch_delete_returns_no_selection_error_* |
| Phase 4 | Graceful degradation when db_tx is None | test_dispatch_delete_handles_none_db_tx_gracefully |
| Phase 5 | Error handling and logging | test_dispatch_delete_handles_send_failure |
