# Contract Specification - Bead seshat-w1j

## Metadata
- bead_id: seshat-w1j
- bead_title: UI Dispatch: Edge Connection
- phase: CONTRACT_SYNTHESIS
- updated_at: 2026-03-12T12:00:00Z

## Context
- **Feature**: Wire toolbar Edge Connect button to construct DomainOp::EdgeConnect and dispatch to db_tx
- **Domain terms**:
  - `DiagramDocument` - the diagram being edited
  - `NodeId` - unique identifier for nodes
  - `EdgeId` - unique identifier for edges  
  - `Coroutine<EventEnvelope>` - WAL channel for persisting operations
  - `DomainOp::EdgeConnect` - operation to create an edge between two nodes
- **Assumptions**:
  - Edge drawing is triggered from toolbar button or canvas interaction
  - Source and target node IDs come from the UI selection/drawing context
  - The edge must preserve DAG (no cycles allowed)
- **Open questions**: None - implementation already exists

## Function Signature
```rust
pub fn handle_edge_drawing_complete(
    db_tx: Option<Coroutine<EventEnvelope>>,
    doc: &DiagramDocument,
    source_id: String,
    target_id: String,
) -> Result<DispatchResult, DispatchError>;
```

## Preconditions (P1-P7)
- [P1] source_id must be non-empty string -> `DispatchError::EdgeNotFound`
- [P2] target_id must be non-empty string -> `DispatchError::EdgeNotFound`
- [P3] source_id must exist in doc.document.nodes -> `DispatchError::EdgeNotFound`
- [P4] target_id must exist in doc.document.nodes -> `DispatchError::EdgeNotFound`
- [P5] source_id must not equal target_id (self-loop) -> `DispatchError::SelfLoop`
- [P6] Edge must preserve DAG (no cycles) -> `DispatchError::CycleDetected`
- [P7] db_tx must be Some (channel available) -> `DispatchError::ChannelMissing`

## Postconditions (Q1-Q3)
- [Q1] Returns Ok(DispatchResult) with nodes_affected >= 1 and dispatches_sent >= 1
- [Q2] EventEnvelope with DomainOp::EdgeConnect is sent to db_tx channel
- [Q3] source and target are correctly mapped to NodeId in the envelope

## Invariants (I1-I2)
- [I1] Document nodes remain unchanged after operation (only edges are added)
- [I2] If dispatch succeeds, exactly one EdgeConnect event is in the WAL channel

## Error Taxonomy
| Error Variant | Condition |
|--------------|-----------|
| `DispatchError::EdgeNotFound` | P1-P4: Invalid/missing source or target node |
| `DispatchError::SelfLoop` | P5: source_id == target_id |
| `DispatchError::CycleDetected` | P6: Edge would create DAG cycle |
| `DispatchError::ChannelMissing` | P7: db_tx is None |

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|--------------|-------------------|----------------|
| P1: source non-empty | Runtime check | `if source_id.is_empty()` |
| P2: target non-empty | Runtime check | `if target_id.is_empty()` |
| P3: source exists | Runtime check | `doc.document.nodes.contains_key(&NodeId::new(source_id))` |
| P4: target exists | Runtime check | `doc.document.nodes.contains_key(&NodeId::new(target_id))` |
| P5: not self-loop | Runtime check | `source != target` |
| P6: DAG preserved | Runtime check | `edge_preserves_dag()` |
| P7: channel available | Runtime check | `if db_tx.is_none()` |

## Violation Examples (REQUIRED)
- VIOLATES P1: `handle_edge_drawing_complete(None, &doc, "", "node-2")` -> `Err(DispatchError::EdgeNotFound)`
- VIOLATES P2: `handle_edge_drawing_complete(None, &doc, "node-1", "")` -> `Err(DispatchError::EdgeNotFound)`
- VIOLATES P3: `handle_edge_drawing_complete(None, &doc, "nonexistent", "node-2")` -> `Err(DispatchError::EdgeNotFound)`
- VIOLATES P4: `handle_edge_drawing_complete(None, &doc, "node-1", "nonexistent")` -> `Err(DispatchError::EdgeNotFound)`
- VIOLATES P5: `handle_edge_drawing_complete(None, &doc, "node-1", "node-1")` -> `Err(DispatchError::SelfLoop)`
- VIOLATES P6: `handle_edge_drawing_complete(None, &doc, "child-node", "parent-node")` (creates cycle) -> `Err(DispatchError::CycleDetected)`
- VIOLATES P7: `handle_edge_drawing_complete(None, &doc, "node-1", "node-2")` -> `Err(DispatchError::ChannelMissing)`

## Ownership Contracts
- `doc: &DiagramDocument` - shared borrow, read-only, no mutation postconditions
- `source_id: String` - ownership transferred to envelope creation
- `target_id: String` - ownership transferred to envelope creation
- `db_tx: Option<Coroutine<EventEnvelope>>` - borrowed reference to channel, not owned

## Non-goals
- [ ] Handle edge style/persistence options
- [ ] Multi-edge selection and bulk operations
- [ ] Edge validation beyond DAG preservation
