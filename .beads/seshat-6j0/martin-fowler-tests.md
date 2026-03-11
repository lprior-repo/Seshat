# Martin Fowler Test Plan

## Happy Path Tests

### test_dispatch_edge_style_update_to_db_tx_when_single_edge_selected
**Given**: PropertiesPanel is rendered with a single edge selected (edge-id: "edge-1", current style: Solid)  
**When**: User changes Line Style select to "dashed"  
**Then**:
- db_tx receives exactly one EventEnvelope
- EventEnvelope.operation is DomainOp::EdgeStyleUpdate { id: "edge-1", style: EdgeStyle::Dashed }
- doc_signal reflects EdgeStyle::Dashed for edge "edge-1"
- doc_signal.revision is incremented

### test_dispatch_edge_style_update_solid_to_dotted
**Given**: Single edge selected with style Dashed  
**When**: User changes to "dotted"  
**Then**:
- db_tx receives EventEnvelope with style Dotted
- Document signal shows Dotted

### test_dispatch_edge_style_update_dotted_to_solid
**Given**: Single edge selected with style Dotted  
**When**: User changes to "solid"  
**Then**:
- db_tx receives EventEnvelope with style Solid
- Document signal shows Solid

## Error Path Tests

### test_no_db_tx_available_does_not_panic
**Given**: PropertiesPanel context has db_tx = None  
**When**: User changes edge style  
**Then**:
- Does NOT panic (P1 violation handled gracefully)
- Document signal still updates (optimistic UI)
- Operation silently succeeds for UX

### test_edge_not_found_returns_error
**Given**: Document with no edges, or edge ID doesn't exist  
**When**: apply_edge_style is called with invalid ID  
**Then**: Returns Err(EdgeOpsError::EdgeNotFound(id))

### test_invalid_style_string_defaults_to_solid
**Given**: parse_edge_style receives unknown string "invalid"  
**When**: Function executes  
**Then**: Returns EdgeStyle::Solid (fail-safe default)

### test_db_tx_channel_closed_returns_error
**Given**: db_tx channel is closed (send fails)  
**When**: tx.send(envelope) is called  
**Then**: Returns Err(ChannelError::SendError)

## Edge Case Tests

### test_no_edge_selected_does_not_show_style_editor
**Given**: No items selected (selected_total = 0)  
**When**: PropertiesPanel renders  
**Then**:
- Line Style select is NOT rendered (only default edge style shown)
- No edge style dispatch possible

### test_multi_select_does_not_show_style_editor
**Given**: Multiple edges selected (selected_edge_count > 1)  
**When**: PropertiesPanel renders  
**Then**:
- Line Style select is NOT rendered
- Multi-select hint is shown

### test_node_selected_does_not_show_edge_style_editor
**Given**: Single node selected (selected_node_count = 1, selected_edge_count = 0)  
**When**: PropertiesPanel renders  
**Then**:
- Line Style select is NOT rendered
- Node properties are shown

### test_same_style_change_is_idempotent
**Given**: Edge with style Dashed is selected, dropdown shows "dashed"  
**When**: User selects "dashed" again (no actual change)  
**Then**:
- db_tx still receives EventEnvelope (idempotent dispatch)
- Document revision still increments

## Contract Verification Tests

### test_precondition_db_tx_must_be_some
**Given**: db_tx context is None  
**When**: on_edge_style_change is invoked  
**Then**: Does NOT panic - uses conditional `if let Some(tx) = &db_tx`

### test_precondition_edge_must_exist
**Given**: Document state without edge "edge-999"  
**When**: apply_edge_style(state, "edge-999", EdgeStyle::Dashed)  
**Then**: Returns Err(EdgeOpsError::EdgeNotFound("edge-999"))

### test_precondition_style_must_be_valid_enum
**Given**: EdgeStyle enum restricts to {Solid, Dashed, Dotted}  
**When**: Code attempts invalid style  
**Then**: Compile-time error - enum exhaustive matching

### test_postcondition_envelope_dispatched_to_db_tx
**Given**: Valid edge selected, db_tx available  
**When**: onchange fires with new style  
**Then**: db_tx.send() called exactly once with correct envelope

### test_postcondition_document_mutated
**Given**: Edge with style Solid  
**When**: User changes to Dashed  
**Then**: doc_signal.read().document.edges[id].style == EdgeStyle::Dashed

### test_postcondition_revision_incremented
**Given**: doc.revision = 5  
**When**: Edge style changes  
**Then**: doc.revision = 6

### test_invariant_edge_style_always_valid
**Given**: Any document state  
**When**: Edge style is read  
**Then**: Value is one of {Solid, Dashed, Dotted}

### test_invariant_edge_exists_in_map
**Given**: Edge ID in event envelope  
**When**: Event is replayed  
**Then**: Edge exists in edges map

## Contract Violation Tests

### test_p1_violation_db_tx_none_panics_if_not_guarded
**Given**: Code calls `db_tx.as_ref().unwrap().send(envelope)` without checking  
**When**: db_tx is None  
**Then**: Would panic (should use `if let Some(tx)` pattern)

**From contract**: VIOLATES P1: `db_tx.send(envelope)` when db_tx is `None` -> causes panic

### test_p2_violation_nonexistent_edge_returns_error
**Given**: apply_edge_style called with "nonexistent-id"  
**When**: Function executes  
**Then**: Returns Err(EdgeOpsError::EdgeNotFound("nonexistent-id"))

**From contract**: VIOLATES P2: `apply_edge_style(state, "nonexistent-id", EdgeStyle::Dashed)` -> returns `Err(EdgeOpsError::EdgeNotFound("nonexistent-id"))`

### test_q1_violation_no_envelope_sent
**Given**: onchange handler implementation missing db_tx.send()  
**When**: User changes style  
**Then**: db_tx never receives envelope (Q1 violated)

**From contract**: VIOLATES Q1: onchange fires but db_tx never receives envelope -> event log desync

### test_q5_violation_send_error
**Given**: Channel closed (simulated)  
**When**: tx.send(envelope) called  
**Then**: Returns Err(SendError)

**From contract**: VIOLATES Q5: `tx.send(envelope)` when channel closed -> returns `Err(SendError)`

## Given-When-Then Scenarios

### Scenario 1: User changes edge style via PropertiesPanel
**Given**:
- A diagram document with at least one edge ("edge-1") exists
- The edge has style Solid
- db_tx coroutine is available in context

**When**:
1. User selects edge "edge-1" in canvas
2. PropertiesPanel shows edge properties
3. User clicks Line Style dropdown
4. User selects "dashed"

**Then**:
- Edge style in document changes to Dashed immediately (optimistic)
- db_tx receives EventEnvelope with DomainOp::EdgeStyleUpdate { id: "edge-1", style: Dashed }
- Document revision increments
- Event is persisted to event store

### Scenario 2: User changes style but db_tx unavailable
**Given**:
- Edge selected
- db_tx context is None (e.g., isolated test environment)

**When**:
User changes Line Style to "dotted"

**Then**:
- Document updates optimistically (UX remains responsive)
- No panic occurs
- No event dispatched (acceptable degradation)

### Scenario 3: Replay edge style from event log
**Given**:
- Event envelope: DomainOp::EdgeStyleUpdate { id: "edge-1", style: Dotted }
- Current document state: edge-1 has style Solid

**When**:
apply_edge_style is called during replay

**Then**:
- Edge style becomes Dotted
- Document revision matches envelope revision
