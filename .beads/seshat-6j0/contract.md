# Contract Specification

## Context
- **Feature**: Wire PropertiesPanel edge style change to dispatch UpdateEdgeStyle to db_tx
- **Bead ID**: seshat-6j0
- **Domain terms**:
  - `PropertiesPanel` - Dioxus UI component displaying properties of selected nodes/edges (diagram_tool/src/ui/properties.rs)
  - `EdgeStyle` - Enum with variants: Solid, Dashed, Dotted (diagram_tool/src/models/document.rs:340)
  - `db_tx` - Coroutine channel `<Option<Coroutine<EventEnvelope>>>` used to dispatch events to the persistence layer
  - `EventEnvelope` - Struct containing operation, author metadata, timestamp (diagram_tool/src/models/envelope.rs:374)
  - `DomainOp` - Enum of all domain operations (diagram_tool/src/models/envelope.rs:98)
  - `DiagramDocument` - The document model containing nodes and edges
  - `Signal<T>` - Dioxus reactive signal type

- **Assumptions**:
  1. The PropertiesPanel already has access to db_tx via context (like canvas.rs does)
  2. A new DomainOp variant `EdgeStyleUpdate { id: String, style: EdgeStyle }` needs to be added
  3. The existing `apply_edge_style` function in edge_ops.rs can handle the operation once added to DomainOp
  4. The onchange handler at properties.rs:592-601 is the target location for the dispatch

- **Open questions**:
  1. Should the direct `doc_signal.with_mut()` mutation be removed or kept as optimistic UI with db_tx as source of truth?
  2. Does the history signal need to be updated alongside db_tx dispatch?

## Preconditions

- **P1**: `PropertiesPanel` must have valid access to `db_tx` context (non-None)
  - Type encoding: `Option<Coroutine<EventEnvelope>>` - caller must check `Some` before sending
  - Violation: Calling send on None produces runtime panic - should use `if let Some(tx) = &db_tx`

- **P2**: Edge ID must exist in the document before style update
  - Type encoding: Compile-time via `EdgeId::new()` constructor validation
  - Violation: `apply_edge_style` returns `Err(EdgeOpsError::EdgeNotFound(id))`

- **P3**: EdgeStyle value must be a valid variant (Solid, Dashed, or Dotted)
  - Type encoding: Compile-time via enum `EdgeStyle` - only three valid values
  - Violation: Invalid string input defaults to Solid (see parse_edge_style at properties.rs:50-55)

- **P4**: EventEnvelope must have valid author and timestamp
  - Type encoding: Runtime validation - author must have non-empty id
  - Violation: Invalid author produces malformed envelope

## Postconditions

- **Q1**: After onchange fires, db_tx receives exactly one EventEnvelope with operation DomainOp::EdgeStyleUpdate
  - Violation: No envelope sent = postcondition violated, state diverges from event log

- **Q2**: The EventEnvelope.operation must contain the correct edge ID and new EdgeStyle
  - Violation: Incorrect ID or style = event log corruption

- **Q3**: The local document signal (doc_signal) reflects the new edge style (optimistic update)
  - Violation: UI doesn't update immediately = poor UX

- **Q4**: The document revision is incremented after mutation
  - Violation: Revision not incremented = conflict detection fails

- **Q5**: db_tx.send() succeeds (channel not closed)
  - Violation: send()Err variant = dispatch failed, need fallback

## Invariants

- **I1**: Document edges map contains the edge ID throughout the operation
  - Cannot update an edge that doesn't exist

- **I2**: EdgeStyle remains one of {Solid, Dashed, Dotted} at all times
  - No invalid style states

- **I3**: db_tx channel remains open while app is running
  - If closed, no more events can be dispatched

## Error Taxonomy

- **EdgeOpsError::EdgeNotFound(String)** - When edge ID doesn't exist in document
- **ContractError::InvalidJson(String)** - When JSON parsing fails
- **ContractError::MissingField(String)** - When required envelope field is missing
- **ChannelError::SendError** - When db_tx channel is closed or full

## Contract Signatures

```rust
// New DomainOp variant needed:
pub enum DomainOp {
    // ... existing variants
    EdgeStyleUpdate {
        id: String,
        style: EdgeStyle,
    },
}

// PropertiesPanel onchange handler (conceptual):
fn on_edge_style_change(
    edge_id: EdgeId,
    new_style: EdgeStyle,
    doc_signal: &mut Signal<DiagramDocument>,
    db_tx: &Option<Coroutine<EventEnvelope>>,
) -> Result<(), DispatchError> {
    // 1. Validate edge exists (P2)
    // 2. Create EventEnvelope with EdgeStyleUpdate (Q1, Q2)
    // 3. Optimistically update doc_signal (Q3)
    // 4. Increment revision (Q4)
    // 5. Send to db_tx if available (Q5, P1)
}
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| db_tx available | Runtime-checked | `if let Some(tx) = &db_tx` |
| Edge exists | Compile-time | `EdgeId::new()` + runtime check |
| EdgeStyle valid | Compile-time | `enum EdgeStyle { Solid, Dashed, Dotted }` |
| Author valid | Runtime-checked | `if author.id.is_empty()` |
| Channel open | Runtime-checked | `tx.send().map_err(|_| ChannelError::SendError)` |

## Violation Examples

- **VIOLATES P1**: `db_tx.send(envelope)` when db_tx is `None` -> causes panic
- **VIOLATES P2**: `apply_edge_style(state, "nonexistent-id", EdgeStyle::Dashed)` -> returns `Err(EdgeOpsError::EdgeNotFound("nonexistent-id"))`
- **VIOLATES Q1**: onchange fires but db_tx never receives envelope -> event log desync
- **VIOLATES Q2**: Envelope has wrong edge ID -> wrong edge updated in replay
- **VIOLATES Q5**: `tx.send(envelope)` when channel closed -> returns `Err(SendError)`

## Ownership Contracts

- `doc_signal: &mut Signal<DiagramDocument>` - Exclusive borrow for mutation
  - Mutates: `doc.document.edges[id].style`, `doc.revision`
- `db_tx: &Option<Coroutine<EventEnvelope>>` - Shared borrow of channel
  - No ownership transfer, uses clone() for closures
- `edge_id: EdgeId` - Value type, owned copy
- `new_style: EdgeStyle` - Copy type, cloned into closure

## Non-goals

- [ ] Adding undo/redo support for edge style changes (future work)
- [ ] Batch style updates for multi-select (future work)
- [ ] Real-time sync to remote peers (future work)
- [ ] Changing arrow_type in PropertiesPanel (different bead)
- [ ] Changing default edge style (line 245 in properties.rs - different context)
