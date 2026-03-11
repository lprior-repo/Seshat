# Contract Specification: seshat-088 (UI Dispatch: Edge Connect)

## Context

- **Bead ID**: seshat-088
- **Feature**: Wire the edge drawing completion handler to construct `DomainOp::EdgeConnect` and dispatch to `db_tx`
- **Domain terms**:
  - `DomainOp::EdgeConnect` - domain operation connecting two nodes with an edge (fields: `id`, `source`, `target`)
  - `db_tx` - `Option<Coroutine<EventEnvelope>>` channel for dispatching events to the durable backend store
  - `InteractionMode::DrawingEdge` - UI state when user is dragging an edge from a source node
  - Edge drawing completion handler - the `onmouseup` handler for `InteractionMode::DrawingEdge` in `canvas.rs` (lines ~1987-2066)
  - `find_node_at(&doc, x, y)` - function that returns `Option<NodeId>` for hit testing at canvas coordinates
- **Assumptions**:
  - The `db_tx` coroutine is available in the canvas scope (already injected via Dioxus context)
  - `DomainOp::EdgeConnect` parsing and projection logic already exists in the backend
  - Edge validation (DAG acyclicity) is already performed before dispatch
- **Open questions**:
  - Should edge ID be generated in UI or assigned by backend? (Currently: UI generates UUID)
  - Should source_port/target_port be captured? (Currently: not captured in this bead)

---

## EARS (Event-based Attribute Requirement Specification)

### Ubiquitous Requirements
- **U1**: UI shall notify backend when edge is routed (persisted to durable store)

### Event-Driven Requirements
- **E1**: Edge drag released over valid port triggers `EdgeConnect` dispatch to `db_tx`
- **E2**: Edge drag released over invalid target (empty space) shall NOT dispatch to `db_tx`

### Unwanted Behavior
- **U1**: No dispatch occurs if mouse released in empty space (no valid target node)
- **U2**: No dispatch occurs if target node equals source node (self-loop prevention)
- **U3**: No dispatch occurs if edge would create DAG cycle (already validated before dispatch)

---

## Preconditions

| ID | Description | Enforcement Level | Type / Pattern |
|----|-------------|-------------------|----------------|
| P1 | `db_tx` channel is Some (backend connected) | Runtime | `if let Some(tx) = &db_tx` |
| P2 | Source node exists in document | Compile-time | `doc.nodes.get(&from_node).is_some()` |
| P3 | Target node exists in document | Compile-time | `doc.nodes.get(&target_id).is_some()` |
| P4 | Target node is different from source node | Compile-time | `target_id != from_node` |
| P5 | Edge passes DAG validation (no cycles) | Runtime | `edge_preserves_dag()` returns true |
| P6 | Interaction mode is `DrawingEdge` | Compile-time | `matches!(mode, InteractionMode::DrawingEdge { .. })` |

---

## Postconditions

| ID | Description |
|----|-------------|
| Q1 | `EventEnvelope` with `DomainOp::EdgeConnect` is sent to `db_tx` if preconditions met |
| Q2 | Local document state is updated with new edge (already implemented) |
| Q3 | UI transitions from `DrawingEdge` to `Select` mode (or continues chain if in Edge tool) |
| Q4 | No operation is dispatched if preconditions fail (graceful no-op) |

---

## Invariants

| ID | Description |
|----|-------------|
| I1 | Document revision is incremented after successful edge creation |
| I2 | History stack is pushed before mutation (for undo capability) |
| I3 | Edge ID is unique (UUID v4 generated for each edge) |

---

## Error Taxonomy

| Error Variant | Condition | Recovery |
|---------------|-----------|----------|
| `DispatchError::ChannelMissing` | `db_tx` is None (backend disconnected) | Log warning, continue local-only |
| `DispatchError::ChannelSendFailed` | `tx.send()` returns Err (channel closed) | Log error, local mutation still valid |
| `ValidationError::SelfLoop` | Target equals source | Show toast, return to DrawingEdge mode |
| `ValidationError::CycleDetected` | Edge creates DAG cycle | Show toast, return to DrawingEdge mode |

---

## Contract Signatures

```rust
/// Dispatches EdgeConnect operation to backend after edge drawing completes
fn dispatch_edge_connect(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    edge_id: EdgeId,
    source: NodeId,
    target: NodeId,
    author: Author,
) -> Result<(), DispatchError>;

/// Handler for mouse-up event during edge drawing
/// Returns: Ok(()) if dispatch successful, Err(...) otherwise
fn handle_edge_drawing_complete(
    doc: &DiagramDocument,
    history: &mut History,
    from_node: &NodeId,
    target_node: &NodeId,
    db_tx: &Option<Coroutine<EventEnvelope>>,
) -> Result<DiagramDocument, DispatchError>;
```

---

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---------------|-------------------|----------------|
| P1: db_tx Some | Runtime | `if let Some(tx) = &db_tx { ... }` |
| P2: source exists | Compile-time | `doc.nodes.get(&from_node).is_some()` via `find_node_at` returns Some |
| P3: target exists | Compile-time | `doc.nodes.get(&target_id).is_some()` via `find_node_at` returns Some |
| P4: source != target | Compile-time | `target_id != from_node` |
| P5: DAG valid | Runtime | `edge_preserves_dag(&doc, &candidate_edge)` returns true |
| P6: DrawingEdge mode | Compile-time | Pattern match in `onmouseup` handler |

---

## Violation Examples (REQUIRED)

### Precondition Violations

- **VIOLATES P1**: `dispatch_edge_connect(None, edge_id, source, target, author)`  
  Should produce: `Err(DispatchError::ChannelMissing)`  

- **VIOLATES P4**: `dispatch_edge_connect(Some(tx), edge_id, node_a, node_a, author)` (self-loop)  
  Should produce: `Err(ValidationError::SelfLoop)`  

- **VIOLATES P5**: `dispatch_edge_connect` with edge that creates cycle  
  Should produce: `Err(ValidationError::CycleDetected)`  

### Postcondition Violations

- **VIOLATES Q1**: Edge drawing completes over valid target but `db_tx` is None  
  Expected: Local edge created, no dispatch, no error returned (graceful degradation)  

- **VIOLATES Q3**: Edge drawing completes but UI mode not reset to Select  
  Expected: Mode should transition appropriately per tool state  

---

## Ownership Contracts

- **db_tx**: `&Option<Coroutine<EventEnvelope>>` - shared borrow, no mutation of channel itself
- **doc**: `&DiagramDocument` - read-only reference for validation
- **doc_mut**: `&mut DiagramDocument` - mutation: `doc.document.edges` updated, `doc.revision` incremented
- **history**: `&mut History` - mutation: history stack pushed before mutation

---

## Implementation Phases

### Phase 1: Core Dispatch Logic
1. Extract edge ID generation to helper function
2. Construct `EventEnvelope` with `DomainOp::EdgeConnect`
3. Send to `db_tx` if channel is available

### Phase 2: Error Handling
1. Add `DispatchError` enum
2. Handle channel missing case (log warning, continue)
3. Handle channel send failure (log error, continue)

### Phase 3: Integration
1. Wire into existing `onmouseup` handler at line ~2001
2. Place dispatch call after local document update
3. Ensure history push happens before dispatch

### Phase 4: Validation Parity
1. Ensure parity with `NodeMove` dispatch pattern (lines ~458-477)
2. Verify toast messages for validation failures still work
3. Test edge chain continuation in Edge tool mode

---

## Non-goals

- [ ] Implementing port-based connections (source_port, target_port) - deferred to future bead
- [ ] Implementing edge style/color customization during draw - deferred
- [ ] Implementing edge label during draw - deferred
- [ ] Backend persistence implementation - already exists
- [ ] Edge projection/replay logic - already exists
