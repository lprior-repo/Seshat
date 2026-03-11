# Martin Fowler Test Plan: Edge Disconnect UI Dispatch

## Test Category: Edge Disconnect Dispatch

This test plan covers the UI event dispatch for `DomainOp::EdgeDisconnect` per bead seshat-5zs.

---

## Happy Path Tests

### test_dispatch_single_edge_disconnect_when_edge_selected
**Given**: Document with edge "edge-1" in `document.edges` and "edge-1" in `selected_items`  
**When**: `dispatch_edge_disconnect("edge-1", doc, db_tx)` is called  
**Then**:
- Returns `Ok(())`
- `db_tx` receives exactly one `EventEnvelope`
- The envelope's `operation` is `DomainOp::EdgeDisconnect { id: "edge-1" }`
- The envelope has valid `op_id`, `author`, and `timestamp`

### test_dispatch_multiple_edge_disconnects
**Given**: Document with edges "edge-1", "edge-2", "edge-3" in `document.edges` and all three in `selected_items`  
**When**: `dispatch_selected_edge_disconnects(doc, db_tx)` is called  
**Then**:
- Returns `Ok(3)` (count of dispatched operations)
- `db_tx` receives exactly three `EventEnvelope` messages
- Each envelope contains `DomainOp::EdgeDisconnect` with respective edge ID

### test_dispatch_generates_unique_op_ids
**Given**: Document with edge "edge-1" selected  
**When**: `dispatch_edge_disconnect` is called twice in quick succession  
**Then**:
- Both envelopes have unique `op_id` values (UUID v4 format)
- No duplicate operation IDs

### test_dispatch_uses_local_user_author
**Given**: Document with edge "edge-1" selected  
**When**: `dispatch_edge_disconnect("edge-1", doc, db_tx)` is called  
**Then**:
- The envelope's `author.id` equals "local-user"
- The envelope's `author.name` equals "Local User"

### test_dispatch_uses_valid_timestamp
**Given**: Document with edge "edge-1" selected  
**When**: `dispatch_edge_disconnect("edge-1", doc, db_tx)` is called  
**Then**:
- The envelope's `timestamp` is a valid Unix timestamp (milliseconds since epoch)
- Timestamp is within 1 second of current time

---

## Error Path Tests

### test_returns_error_when_edge_not_in_selection
**Given**: Document with edge "edge-1" in `document.edges` but NOT in `selected_items`  
**When**: `dispatch_edge_disconnect("edge-1", doc, db_tx)` is called  
**Then**: Returns `Err(DispatchError::NotSelected)`

### test_returns_error_when_edge_not_in_document
**Given**: Document with no edge "nonexistent" in `document.edges`  
**When**: `dispatch_edge_disconnect("nonexistent", doc, db_tx)` is called  
**Then**: Returns `Err(DispatchError::EdgeNotFound)`

### test_returns_error_when_db_tx_unavailable
**Given**: Document with edge "edge-1" selected and in document  
**When**: `dispatch_edge_disconnect("edge-1", doc, &None)` is called with `db_tx = None`  
**Then**: Returns `Err(DispatchError::NoTx)`

### test_handles_empty_selection_gracefully
**Given**: Document with `selected_items` = empty set  
**When**: `dispatch_selected_edge_disconnects(doc, db_tx)` is called  
**Then**: Returns `Ok(0)` - no operations dispatched, no error

### test_skips_nonexistent_edges_in_selection
**Given**: Document with "edge-1" in `selected_items` but NOT in `document.edges`  
**When**: `dispatch_selected_edge_disconnects(doc, db_tx)` is called  
**Then**:
- Returns `Ok(0)` (skips invalid edge)
- No envelope is dispatched for the nonexistent edge

---

## Edge Case Tests

### test_handles_empty_edge_id
**Given**: Document with empty string "" in `selected_items`  
**When**: `dispatch_edge_disconnect("", doc, db_tx)` is called  
**Then**: Returns `Err(DispatchError::NotSelected)` (empty string not in valid selection)

### test_handles_unicode_edge_id
**Given**: Document with edge "edge-中文" in `document.edges` and `selected_items`  
**When**: `dispatch_edge_disconnect("edge-中文", doc, db_tx)` is called  
**Then**:
- Returns `Ok(())`
- Envelope is dispatched with correct Unicode edge ID

### test_handles_special_characters_in_edge_id
**Given**: Document with edge "edge-$#@!" in `document.edges` and `selected_items`  
**When**: `dispatch_edge_disconnect("edge-$#@!", doc, db_tx)` is called  
**Then**:
- Returns `Ok(())`
- Envelope is dispatched with correct edge ID

### test_handles_very_long_edge_id
**Given**: Document with 10KB edge ID in `document.edges` and `selected_items`  
**When**: `dispatch_edge_disconnect(long_id, doc, db_tx)` is called  
**Then**:
- Returns `Ok(())`
- Envelope is dispatched (assuming reasonable length limits)

### test_dispatch_preserves_other_selections
**Given**: Document with "node-1" and "edge-1" both in `selected_items`  
**When**: `dispatch_edge_disconnect("edge-1", doc, db_tx)` is called  
**Then**:
- "node-1" remains in `selected_items`
- Only "edge-1" disconnect is dispatched

---

## Contract Verification Tests

### test_precondition_p1_edge_in_selection
**Given**: Document with edge "edge-1" in `selected_items`  
**When**: `dispatch_edge_disconnect("edge-1", doc, db_tx)`  
**Then**: Precondition P1 satisfied - edge is in selection

### test_precondition_p2_edge_exists
**Given**: Document with edge "edge-1" in `document.edges`  
**When**: `dispatch_edge_disconnect("edge-1", doc, db_tx)`  
**Then**: Precondition P2 satisfied - edge exists in document

### test_precondition_p3_db_tx_available
**Given**: `db_tx` is `Some(coroutine)`  
**When**: `dispatch_edge_disconnect("edge-1", doc, &db_tx)`  
**Then**: Precondition P3 satisfied - tx is available

### test_postcondition_q1_envelope_sent
**Given**: Valid edge in selection and document, db_tx available  
**When**: `dispatch_edge_disconnect("edge-1", doc, db_tx)`  
**Then**: Postcondition Q1 satisfied - envelope sent to tx

### test_postcondition_q2_valid_op_id
**Given**: Valid dispatch call  
**When**: After dispatch, inspect envelope  
**Then**: Postcondition Q2 satisfied - op_id is valid UUID

### test_postcondition_q3_valid_author
**Given**: Valid dispatch call  
**When**: After dispatch, inspect envelope  
**Then**: Postcondition Q3 satisfied - author is "local-user"

### test_postcondition_q4_valid_timestamp
**Given**: Valid dispatch call  
**When**: After dispatch, inspect envelope  
**Then**: Postcondition Q4 satisfied - timestamp is valid Unix ms

---

## Contract Violation Tests

### test_violation_p1_returns_not_selected_error
**Given**: `dispatch_edge_disconnect("edge-not-selected", doc, db_tx)` where "edge-not-selected" NOT in selection  
**When**: Function is called  
**Then**: Returns `Err(DispatchError::NotSelected)` -- NOT a panic

### test_violation_p2_returns_edge_not_found_error
**Given**: `dispatch_edge_disconnect("nonexistent", doc, db_tx)` where "nonexistent" NOT in document.edges  
**When**: Function is called  
**Then**: Returns `Err(DispatchError::EdgeNotFound)` -- NOT a panic

### test_violation_p3_returns_no_tx_error
**Given**: `dispatch_edge_disconnect("edge-1", doc, &None)` with db_tx = None  
**When**: Function is called  
**Then**: Returns `Err(DispatchError::NoTx)` -- NOT a panic

---

## Given-When-Then Scenarios

### Scenario 1: Delete Key Pressed with Single Edge Selected
**Given**: A diagram document with one edge "edge-1" that is selected  
**When**: User presses the Delete key  
**Then**:
- `dispatch_edge_disconnect("edge-1", doc, db_tx)` is invoked
- Returns `Ok(())`
- `db_tx` receives `EventEnvelope` with `DomainOp::EdgeDisconnect { id: "edge-1" }`
- "edge-1" is removed from `selected_items`

### Scenario 2: Delete Key Pressed with Multiple Edges Selected
**Given**: A diagram document with three edges "e1", "e2", "e3" all selected  
**When**: User presses the Delete key  
**Then**:
- `dispatch_selected_edge_disconnects(doc, db_tx)` is invoked
- Returns `Ok(3)`
- `db_tx` receives three separate `EventEnvelope` messages
- All three edges are removed from `selected_items`

### Scenario 3: Delete Key Pressed with No Selection
**Given**: A diagram document with no items selected  
**When**: User presses the Delete key  
**Then**:
- No dispatch occurs
- No error is returned
- No messages sent to `db_tx`

### Scenario 4: Delete Key Pressed with Only Nodes Selected
**Given**: A diagram document with node "node-1" selected but no edges selected  
**When**: User presses the Delete key  
**Then**:
- Node deletion is handled by `apply_delete_selected` (separate behavior)
- No `EdgeDisconnect` operations are dispatched

### Scenario 5: Delete Key Pressed but db_tx Coroutine Unavailable
**Given**: A diagram document with edge "edge-1" selected, but `db_tx = None`  
**When**: User presses the Delete key  
**Then**:
- `dispatch_edge_disconnect` returns `Err(DispatchError::NoTx)`
- No panic occurs
- Warning is logged (if logging configured)

---

## Integration Tests (End-to-End)

### test_e2e_full_edge_disconnect_workflow
**Given**: 
- A running Dioxus application with canvas
- Document with nodes "n1", "n2" and edge "e1" connecting them
- Edge "e1" is currently selected

**When**:
1. User presses Delete key
2. Canvas event handler processes keydown
3. `dispatch_edge_disconnect("e1", doc, db_tx)` is called

**Then**:
- `EventEnvelope` is sent to `db_tx` coroutine
- The envelope contains valid `DomainOp::EdgeDisconnect { id: "e1" }`
- Operation flows through the event system to projection
- Edge "e1" is removed from the document on replay

---

## Test Matrix

| Test Name | Input | Expected Output | Coverage |
|-----------|-------|-----------------|----------|
| test_dispatch_single_edge_disconnect_when_edge_selected | Edge in selection | Ok(()), envelope sent | Happy path |
| test_dispatch_multiple_edge_disconnects | Multiple edges selected | Ok(3), 3 envelopes | Happy path |
| test_returns_error_when_edge_not_in_selection | Edge not selected | Err(NotSelected) | Error path |
| test_returns_error_when_edge_not_in_document | Nonexistent edge | Err(EdgeNotFound) | Error path |
| test_returns_error_when_db_tx_unavailable | db_tx = None | Err(NoTx) | Error path |
| test_handles_empty_selection_gracefully | Empty selection | Ok(0) | Edge case |
| test_handles_unicode_edge_id | Unicode edge ID | Ok(()), envelope sent | Edge case |
| test_violation_p1_returns_not_selected_error | Violates P1 | Err(NotSelected) | Contract violation |
| test_violation_p2_returns_edge_not_found_error | Violates P2 | Err(EdgeNotFound) | Contract violation |
| test_violation_p3_returns_no_tx_error | Violates P3 | Err(NoTx) | Contract violation |
| test_e2e_full_edge_disconnect_workflow | Full workflow | Envelope flows through | E2E |
