# Martin Fowler Test Plan: seshat-8xu

## Overview

This test plan covers the Toolbar Add Node button feature that wires the button to construct `DomainOp::NodeAdd` and dispatch it to the `db_tx` coroutine.

---

## Happy Path Tests

### test_toolbar_add_node_creates_valid_domain_operation
**Given**: A DiagramDocument with no nodes, a valid History, and a working db_tx coroutine  
**When**: Toolbar Add Node button is clicked  
**Then**:
- A `DomainOp::NodeAdd` is constructed with valid fields
- The operation contains a valid UUID for id
- The operation contains valid coordinates (x, y)
- The operation contains valid dimensions (width, height >= 0)
- The operation contains a label string

### test_toolbar_add_node_dispatches_to_db_tx
**Given**: A DiagramDocument and a working db_tx coroutine (Some variant)  
**When**: add_node action is invoked  
**Then**:
- An EventEnvelope is sent through db_tx.send()
- The envelope's operation field is DomainOp::NodeAdd
- The envelope's op_id is a non-empty UUID string
- The envelope's author.id is "local-user"
- The envelope's timestamp is a valid Unix timestamp

### test_toolbar_add_node_updates_local_document
**Given**: A DiagramDocument with initial node count N  
**When**: add_node action is invoked  
**Then**:
- The document's nodes map contains N+1 entries
- The new node has the correct id, x, y, width, height, label
- The revision number is incremented
- The new node is selected in selected_items

### test_toolbar_add_node_updates_history
**Given**: A History that can_undo() returns false  
**When**: add_node action is invoked  
**Then**:
- History.can_undo() returns true
- Undo restores the document to previous state (N nodes)

### test_toolbar_add_node_default_values
**Given**: A DiagramDocument and db_tx  
**When**: add_node action is invoked with default parameters  
**Then**:
- Node width defaults to 64.0
- Node height defaults to 64.0
- Node label defaults to "Node"
- Node position defaults to viewport center or (0, 0)

---

## Error Path Tests

### test_add_node_handles_missing_db_tx_gracefully
**Given**: A DiagramDocument, History, and db_tx = None  
**When**: add_node action is invoked  
**Then**:
- No panic occurs
- Local document is still updated (node added)
- Warning is logged about unavailable db_tx

### test_add_node_rejects_nan_coordinates
**Given**: Valid doc_signal and history_signal  
**When**: create_node_add_envelope is called with x = f64::NAN  
**Then**:
- Returns Err(Error::InvalidPosition)

### test_add_node_rejects_infinity_coordinates
**Given**: Valid doc_signal and history_signal  
**When**: create_node_add_envelope is called with y = f64::INFINITY  
**Then**:
- Returns Err(Error::InvalidPosition)

### test_add_node_rejects_negative_width
**Given**: Valid doc_signal and history_signal  
**When**: create_node_add_envelope is called with width = -10.0  
**Then**:
- Returns Err(Error::InvalidDimensions)

### test_add_node_rejects_zero_height
**Given**: Valid doc_signal and history_signal  
**When**: create_node_add_envelope is called with height = 0.0  
**Then**:
- Returns Err(Error::InvalidDimensions)

---

## Edge Case Tests

### test_add_node_generates_unique_ids
**Given**: A DiagramDocument and db_tx  
**When**: add_node is invoked twice rapidly  
**Then**:
- Each call generates a different UUID for the node id
- Both nodes exist in the document with different IDs

### test_add_node_handles_empty_document
**Given**: A fresh DiagramDocument with no nodes or edges  
**When**: add_node action is invoked  
**Then**:
- Node count becomes 1
- Edge count remains 0
- Selected items contains the new node

### test_add_node_maintains_document_structure
**Given**: A DiagramDocument with existing nodes  
**When**: add_node action is invoked  
**Then**:
- Existing nodes are not modified
- Existing edges are not modified
- New node is inserted without disrupting z-order

---

## Contract Verification Tests

### test_precondition_p1_db_tx_available
**Given**: db_tx = Some(mock_coroutine)  
**When**: add_node is called  
**Then**:
- Precondition P1 satisfied: db_tx is available
- EventEnvelope is sent successfully

### test_precondition_p5_valid_coordinates
**Given**: x = 100.0, y = 200.0  
**When**: create_node_add_envelope is called  
**Then**:
- Precondition P5 satisfied: coordinates are finite

### test_precondition_p6_positive_dimensions
**Given**: width = 64.0, height = 64.0  
**When**: create_node_add_envelope is called  
**Then**:
- Precondition P6 satisfied: dimensions > 0

### test_postcondition_q5_event_sent_to_db_tx
**Given**: A working db_tx coroutine  
**When**: add_node is invoked  
**Then**:
- Postcondition Q5 satisfied: .send() called on db_tx

### test_postcondition_q6_node_in_document
**Given**: DiagramDocument  
**When**: add_node completes  
**Then**:
- Postcondition Q6 satisfied: doc_signal.document.nodes contains new node

### test_postcondition_q7_history_pushed
**Given**: History  
**When**: add_node completes  
**Then**:
- Postcondition Q7 satisfied: history_signal contains previous state

### test_invariant_i3_history_consistency
**Given**: History that can_undo()  
**When**: undo is called after add_node  
**Then**:
- Invariant I3 maintained: History push/pop is consistent

### test_invariant_i4_revision_increment
**Given**: Document with revision N  
**When**: add_node completes  
**Then**:
- Invariant I4 maintained: revision = N + 1

---

## Contract Violation Tests

### test_violation_p1_db_tx_none_logs_warning
**Given**: db_tx = None  
**When**: add_node is invoked  
**Then**: Returns gracefully, logs warning (not a panic)

```rust
// VIOLATES P1: add_node(doc, history, None) -- should NOT panic
```

### test_violation_p5_nan_returns_error
**Given**: x = f64::NAN  
**When**: create_node_add_envelope is called  
**Then**: Returns Err(Error::InvalidPosition)

```rust
// VIOLATES P5: create_node_add_envelope(id, f64::NAN, y, w, h, label)
// Expected: Err(Error::InvalidPosition)
```

### test_violation_p6_negative_dimension_returns_error
**Given**: width = -50.0  
**When**: create_node_add_envelope is called  
**Then**: Returns Err(Error::InvalidDimensions)

```rust
// VIOLATES P6: create_node_add_envelope(id, x, y, -50.0, h, label)
// Expected: Err(Error::InvalidDimensions)
```

---

## Given-When-Then Scenarios

### Scenario 1: Toolbar Add Node Button Click
**Given**: 
- A DiagramDocument with 2 existing nodes
- History in initial state
- db_tx coroutine is available

**When**: 
User clicks the "Add Node" toolbar button

**Then**:
- 3 nodes exist in the document
- The new node has default dimensions (64x64)
- The new node is selected
- EventEnvelope with DomainOp::NodeAdd is sent to db_tx
- History.can_undo() is true
- Document revision is incremented

### Scenario 2: Add Node Without Database Connection
**Given**:
- A DiagramDocument
- History
- db_tx = None (not available)

**When**:
User clicks "Add Node" toolbar button

**Then**:
- Node is still added to local document (for immediate feedback)
- Warning is logged about db_tx unavailability
- No panic occurs
- User sees the new node in the UI

### Scenario 3: Rapid Sequential Adds
**Given**:
- A DiagramDocument with 0 nodes
- db_tx is available

**When**:
User clicks "Add Node" 5 times in quick succession

**Then**:
- 5 nodes exist in the document
- Each has a unique ID
- 5 EventEnvelopes are sent to db_tx
- All nodes are selected (or last one selected)
- History tracks all 5 operations

---

## Integration Considerations

- The toolbar button should have `data-testid="toolbar-add-node"` for E2E testing
- The button should be disabled when in certain tool modes (e.g., not in Select mode)
- The node should appear at a sensible default position (center of viewport or canvas origin)
- The new node should immediately be editable (focus moves to label edit)
