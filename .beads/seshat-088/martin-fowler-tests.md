# Martin Fowler Test Plan: seshat-088 (UI Dispatch: Edge Connect)

## Happy Path Tests

### test_dispatch_edge_connect_when_released_over_valid_target_node
**Given**: User is in Edge tool mode and has started drawing an edge from node A
**When**: User releases mouse over a different valid target node B (not empty space)
**Then**:
- Edge is created in local document between A and B
- `EventEnvelope` with `DomainOp::EdgeConnect { id, source: A, target: B }` is dispatched to `db_tx`
- UI transitions to `Select` mode (or continues chain if Edge tool still active)

### test_dispatch_creates_unique_edge_id_per_connection
**Given**: User draws edge from node A to node B
**When**: Edge is created and dispatched
**Then**:
- Edge ID is a unique UUID v4
- Two separate edge creations produce different IDs

### test_dispatch_includes_correct_author_metadata
**Given**: User completes edge drawing
**When**: `EventEnvelope` is dispatched
**Then**:
- Author field contains `id: "local-user"`, `name: "Local User"`, `email: None`
- Timestamp is current system time (within 1 second tolerance)

### test_dispatch_succeeds_when_db_tx_channel_is_available
**Given**: `db_tx` channel is Some (backend connected)
**When**: Edge drawing completes over valid target
**Then**:
- `tx.send()` returns `Ok(())`
- No error is returned from handler

---

## Error Path Tests

### test_no_dispatch_when_released_in_empty_space
**Given**: User is drawing edge from node A
**When**: User releases mouse over empty canvas (no node at position)
**Then**:
- No `EventEnvelope` is sent to `db_tx`
- Local document is NOT mutated (no edge created)
- UI transitions to `Select` mode

### test_no_dispatch_when_target_equals_source_self_loop
**Given**: User is drawing edge from node A
**When**: User releases mouse back over node A (self-loop attempt)
**Then**:
- Toast warning "Cannot create circular connection" is shown
- No dispatch occurs
- UI remains in `DrawingEdge` mode (or transitions appropriately)

### test_no_dispatch_when_db_tx_channel_is_none
**Given**: `db_tx` is `None` (backend disconnected)
**When**: Edge drawing completes over valid target
**Then**:
- Local edge is still created in document
- No dispatch occurs (graceful degradation)
- No error is returned (warning may be logged)

### test_no_dispatch_when_edge_creates_cycle
**Given**: User attempts to create edge that would create DAG cycle
**When**: Edge drawing completes over valid-looking target (but creates cycle)
**Then**:
- `edge_preserves_dag()` returns false
- Toast warning "Cannot create circular connection" is shown
- No dispatch occurs
- No local edge is created

### test_channel_send_failure_does_not_crash
**Given**: `db_tx` channel exists but is closed/failed
**When**: Edge drawing completes and dispatch is attempted
**Then**:
- `tx.send()` returns `Err`
- Error is handled gracefully (logged)
- Local edge still created
- No panic occurs

---

## Edge Case Tests

### test_dispatch_handles_rapid_successive_edge_draws
**Given**: User quickly draws multiple edges in succession
**When**: Each edge drawing completes before previous dispatch settles
**Then**:
- Each edge gets unique ID
- Each edge is dispatched separately
- Document state remains consistent

### test_dispatch_preserves_history_for_undo
**Given**: Document with existing nodes
**When**: User draws and completes an edge
**Then**:
- History stack is pushed BEFORE dispatch
- Undo operation can revert edge creation

### test_dispatch_increments_revision_on_success
**Given**: Document at revision N
**When**: Edge drawing completes successfully
**Then**:
- Document revision is N+1 after edge creation

### test_dispatch_continues_chain_in_edge_tool_mode
**Given**: User is in Edge tool mode and draws edge A->B
**When**: Edge completes over valid target B
**Then**:
- UI transitions to `DrawingEdge` mode starting from B (for chaining)
- User can immediately draw B->C without re-selecting

### test_dispatch_transitions_to_select_when_not_in_edge_tool
**Given**: User is in Select tool mode and draws edge A->B
**When**: Edge completes over valid target
**Then**:
- UI transitions to `Select` mode (not DrawingEdge)

---

## Contract Verification Tests

### test_precondition_p1_db_tx_channel_must_be_some
**Given**: `db_tx = None`
**When**: `dispatch_edge_connect(None, edge_id, source, target, author)` is called
**Then**: Returns `Err(DispatchError::ChannelMissing)`

### test_precondition_p4_self_loop_prevention
**Given**: Source and target are the same node
**When**: Edge dispatch is attempted
**Then**: Returns `Err(ValidationError::SelfLoop)` or prevents dispatch

### test_postcondition_q1_envelope_sent_on_success
**Given**: All preconditions met
**When**: Edge drawing completes
**Then**: `db_tx` receives `EventEnvelope` with correct `DomainOp::EdgeConnect`

### test_postcondition_q3_mode_transition
**Given**: Edge drawing completes in Edge tool mode
**When**: Target is valid and dispatch succeeds
**Then**: UI mode is `DrawingEdge` from new target node

### test_postcondition_q3_mode_transition_select_tool
**Given**: Edge drawing completes in Select tool mode
**When**: Target is valid and dispatch succeeds
**Then**: UI mode is `Select`

### test_invariant_i1_revision_incremented
**Given**: Document revision is N
**When**: Edge is created
**Then**: Document revision becomes N+1

### test_invariant_i2_history_pushed
**Given**: History stack has M entries
**When**: Edge drawing begins/completes
**Then**: History stack has M+1 entries (pushed before mutation)

---

## Contract Violation Tests

### test_violation_p1_returns_channel_missing_error
**Given**: `db_tx = None`
**When**: `dispatch_edge_connect(None, edge_id, node_a, node_b, author)`
**Then**: Returns `Err(DispatchError::ChannelMissing)` -- NOT a panic

### test_violation_p4_self_loop_returns_validation_error
**Given**: Source = target = node_a
**When**: `dispatch_edge_connect(Some(tx), edge_id, node_a, node_a, author)`
**Then**: Returns `Err(ValidationError::SelfLoop)` -- NOT a panic

### test_violation_p5_cycle_returns_validation_error
**Given**: Edge would create DAG cycle
**When**: `dispatch_edge_connect(Some(tx), edge_id, source, target, author)`
**Then**: Returns `Err(ValidationError::CycleDetected)` -- NOT a panic

### test_violation_q1_no_dispatch_in_empty_space
**Given**: `find_node_at()` returns None (empty space)
**When**: Mouse is released
**Then**: No `EventEnvelope` is sent to `db_tx` -- graceful no-op

---

## Given-When-Then Scenarios

### Scenario 1: Complete edge drawing with backend available
**Given**: 
- User is in Edge tool mode
- Canvas has two nodes: NodeA at (100, 100) and NodeB at (300, 100)
- `db_tx` channel is Some and connected

**When**:
1. User clicks on NodeA to start drawing edge
2. User drags edge to NodeB and releases mouse

**Then**:
- Edge appears visually connecting NodeA to NodeB
- `EventEnvelope` with `DomainOp::EdgeConnect { id: UUID, source: NodeA, target: NodeB }` sent to `db_tx`
- Document revision incremented
- History pushed
- UI continues in Edge tool mode with DrawingEdge from NodeB

### Scenario 2: Cancel edge drawing in empty space
**Given**:
- User is in Edge tool mode
- NodeA exists on canvas
- `db_tx` is available

**When**:
1. User starts drawing edge from NodeA
2. User releases mouse in empty space (no node)

**Then**:
- No edge created
- No dispatch to `db_tx`
- UI returns to Select mode (or DrawingEdge if Edge tool)
- No error, no toast

### Scenario 3: Backend disconnected during edge drawing
**Given**:
- User is in Edge tool mode
- `db_tx` is None (backend disconnected)

**When**:
1. User draws edge from NodeA to NodeB

**Then**:
- Edge is created in local document
- No dispatch occurs (graceful)
- No error returned
- Document is still usable

---

## Test Implementation Notes

- Use existing test infrastructure in `diagram_tool/src/ui/canvas/domain/tests/`
- Mock `db_tx` using `Option<test::MockCoroutine>` or similar
- Use `InteractionMode::DrawingEdge` fixtures from test DSL
- Follow naming convention: `test_<description>_<expected_behavior>`
- Each test should be self-contained with clear Given-When-Then
