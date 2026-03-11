# Martin Fowler Test Plan: seshat-4a8

## Happy Path Tests

- **test_dispatch_bring_to_front_with_valid_selection_sends_to_db_tx**
  - Given: A document with 3 nodes (node1, node2, node3), node1 and node2 selected
  - When: `dispatch_bring_to_front(doc_signal, history_signal, Some(db_tx))` is called
  - Then: db_tx receives EventEnvelope with DomainOp::BringToFront { ids: ["node1", "node2"] }
  - Then: returns true

- **test_dispatch_bring_to_front_with_single_selected_node**
  - Given: A document with 2 nodes (node1, node2), only node1 selected
  - When: dispatch_bring_to_front is called with db_tx present
  - Then: db_tx receives EventEnvelope with DomainOp::BringToFront { ids: ["node1"] }
  - Then: returns true

- **test_dispatch_bring_to_front_with_all_nodes_selected**
  - Given: A document with 5 nodes, all selected
  - When: dispatch_bring_to_front is called with db_tx present
  - Then: db_tx receives EventEnvelope with all 5 node IDs in correct order
  - Then: returns true

---

## Error Path Tests

- **test_dispatch_bring_to_front_returns_false_when_no_selection**
  - Given: A document with nodes, but selected_items is empty
  - When: dispatch_bring_to_front is called
  - Then: returns false
  - Then: db_tx does not receive any message

- **test_dispatch_bring_to_front_fallback_when_db_tx_none**
  - Given: A document with nodes selected, db_tx is None
  - When: dispatch_bring_to_front is called
  - Then: falls back to direct mutation (calls apply_bring_to_front)
  - Then: returns true (backward compatible)

- **test_dispatch_bring_to_front_handles_dropped_coroutine_gracefully**
  - Given: A document with nodes selected, db_tx is Some(coroutine) but coroutine was dropped
  - When: dispatch_bring_to_front is called
  - Then: send() returns Err, function handles gracefully
  - Then: returns false or indicates failure

---

## Edge Case Tests

- **test_dispatch_bring_to_front_ignores_selected_edges**
  - Given: A document with nodes and edges, edges are in selected_items
  - When: dispatch_bring_to_front is called
  - Then: only node IDs are included in DomainOp::BringToFront
  - Then: edge IDs are filtered out

- **test_dispatch_bring_to_front_handles_missing_node_ids**
  - Given: selected_items contains ID that doesn't exist in document.nodes
  - When: dispatch_bring_to_front is called
  - Then: only valid node IDs are included in DomainOp::BringToFront
  - Then: missing IDs are silently skipped (consistent with apply_z_order_to_ids)

- **test_dispatch_bring_to_front_empty_document**
  - Given: A document with no nodes
  - When: dispatch_bring_to_front is called
  - Then: returns false (no nodes to bring to front)

---

## Contract Verification Tests

- **test_precondition_p1_signal_not_poisoned**
  - Given: A valid Signal<DiagramDocument>
  - When: dispatch_bring_to_front is called
  - Then: Signal can be read without panic

- **test_precondition_p4_selection_not_empty**
  - Given: selected_items is empty
  - When: dispatch_bring_to_front is called
  - Then: returns false (early exit)

- **test_postcondition_q1_event_envelope_sent**
  - Given: Valid selection and db_tx is Some
  - When: dispatch_bring_to_front is called
  - Then: db_tx receives exactly one EventEnvelope
  - Then: EventEnvelope.operation is DomainOp::BringToFront

- **test_postcondition_q2_ids_are_valid_node_ids**
  - Given: Valid selection
  - When: dispatch_bring_to_front is called
  - Then: All ids in DomainOp::BringToFront exist in document.nodes

- **test_postcondition_q3_fallback_to_direct_mutation**
  - Given: db_tx is None
  - When: dispatch_bring_to_front is called
  - Then: apply_bring_to_front is invoked (direct mutation path)

- **test_invariant_i2_selected_items_unchanged**
  - Given: A document with selected_items
  - When: dispatch_bring_to_front is called
  - Then: selected_items before and after are identical

---

## Contract Violation Tests

- `test_p4_violation_empty_selection_returns_false`
  - Given: doc_signal with empty selected_items
  - When: dispatch_bring_to_front(doc_signal, history_signal, Some(tx))
  - Then: returns `false` (NOT a panic, NOT an unwrap failure)

- `test_q1_violation_no_message_sent_with_empty_selection`
  - Given: doc_signal with empty selected_items
  - When: dispatch_bring_to_front called
  - Then: db_tx does NOT receive any message

- `test_q2_violation_invalid_ids_not_included`
  - Given: selected_items contains non-existent node IDs
  - When: dispatch_bring_to_front called
  - Then: only valid node IDs in DomainOp::BringToFront

---

## Given-When-Then Scenarios

### Scenario 1: Successful Dispatch via db_tx

**Given**: A DiagramDocument with nodes ["A", "B", "C"], selected_items = {"A", "B"}
**And**: db_tx is available as Some(coroutine)
**When**: User clicks "Bring to Front" toolbar button
**Then**: 
- actions::bring_to_front is invoked (wired to button)
- dispatch_bring_to_front constructs DomainOp::BringToFront { ids: ["A", "B"] }
- EventEnvelope is sent to db_tx
- db_tx processes the envelope and applies mutation
- Function returns true

### Scenario 2: Fallback to Direct Mutation

**Given**: A DiagramDocument with nodes, selected_items is non-empty
**And**: db_tx context is None (not available)
**When**: User clicks "Bring to Front" toolbar button
**Then**:
- dispatch_bring_to_front detects db_tx is None
- Falls back to apply_bring_to_front (direct signal mutation)
- Document is mutated in-place
- Function returns true

### Scenario 3: No Selection - No-op

**Given**: A DiagramDocument with nodes, selected_items is empty
**When**: User clicks "Bring to Front" toolbar button
**Then**:
- dispatch_bring_to_front detects empty selection
- Returns false immediately
- No message sent to db_tx
- No mutation occurs
- Function returns false

---

## Implementation Phases

1. **Phase 1**: Create `dispatch_bring_to_front` function in commands.rs with db_tx parameter
2. **Phase 2**: Wire toolbar/actions.rs to use dispatch function instead of direct apply
3. **Phase 3**: Update toolbar.rs button to pass db_tx context to action
4. **Phase 4**: Add fallback to direct mutation when db_tx is None
5. **Phase 5**: Test integration with existing NodeMove and other db_tx dispatch patterns
