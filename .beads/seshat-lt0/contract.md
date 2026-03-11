# Contract Specification: seshat-lt0 (UI Dispatch: Bring Forward)

## Context
- **Feature**: Wire BringForward toolbar button to construct DomainOp::BringForward and dispatch to db_tx
- **Domain terms**:
  - `DomainOp::BringForward { ids: Vec<String> }` - Z-order operation to move selected nodes forward in layer stack
  - `EventEnvelope` - Wrapper containing DomainOp, Author, timestamp, and op_id
  - `db_tx` - Dioxus coroutine for async dispatch of EventEnvelope to store bridge
  - `DiagramDocument` - Document signal containing nodes, edges, and editor state
  - `History` - Undo/redo stack
  - `ZOrderOp` - Internal enum (BringForward, SendBackward, BringToFront, SendToBack)
- **Assumptions**:
  - db_tx is available via `use_context::<Option<Coroutine<EventEnvelope>>>()`
  - The store bridge is initialized in app.rs with async-db feature
  - Fallback to direct document manipulation when db_tx is None (WASM builds)
- **Open questions**:
  - Should the old direct-manipulation path be removed entirely or kept as fallback?
  - Should we validate that selected nodes exist before constructing the envelope?

## Preconditions
- [P1] **Selection exists**: At least one node must be selected in `doc_signal.read().editor_state.selected_items`
- [P2] **Selected nodes exist**: All selected node IDs must exist in `doc_signal.read().document.nodes`
- [P3] **db_tx available**: When `cfg!(feature = "async-db")`, db_tx context must be Some; otherwise fallback path is used

## Postconditions
- [Q1] **Envelope dispatched**: If db_tx is Some, an EventEnvelope with DomainOp::BringForward is sent via `db_tx.send(envelope)`
- [Q2] **Correct payload**: The EventEnvelope.operation must be DomainOp::BringForward { ids } where ids matches selected node IDs
- [Q3] **Author populated**: EventEnvelope.author must have id="local-user", name="Local User"
- [Q4] **Timestamp valid**: EventEnvelope.timestamp must be current Unix timestamp in milliseconds
- [Q5] **Unique op_id**: EventEnvelope.op_id must be a fresh UUID v4
- [Q6] **History updated**: On successful dispatch, history_signal must be updated with current document state for undo support
- [Q7] **Fallback behavior**: If db_tx is None, the original direct document manipulation path is executed

## Invariants
- [I1] Document revision increments exactly once per successful z-order operation
- [I2] Selected items set remains unchanged after dispatch (selection persists)
- [I3] Node z_index values are modified only through the store event replay path (not directly)

## Error Taxonomy
- **NoSelection**: Returned when selected set is empty (precondition P1 violation)
- **NodeNotFound**: Returned when selected node ID doesn't exist (precondition P2 violation)
- **DbTxUnavailable**: Returned when async-db feature enabled but db_tx is None
- **DispatchFailed**: Returned when db_tx.send() fails (channel closed)

## Contract Signatures
```rust
// Primary dispatch function
pub fn bring_forward(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> Result<bool, BringForwardError> {
    // If db_tx available: construct EventEnvelope, send via db_tx, update history
    // Else: fallback to direct apply_bring_forward
}

// Helper: construct envelope from selected nodes
fn construct_bring_forward_envelope(
    selected_ids: &BTreeSet<NodeId>,
    doc: &DiagramDocument,
) -> EventEnvelope;

// Error enum
#[derive(Debug, Error)]
pub enum BringForwardError {
    #[error("no nodes selected")]
    NoSelection,
    #[error("node not found: {0}")]
    NodeNotFound(String),
    #[error("db_tx not available")]
    DbTxUnavailable,
    #[error("dispatch failed: {0}")]
    DispatchFailed(String),
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Selection exists | Runtime check + early return | `if selected.is_empty() { return Err(NoSelection) }` |
| P2: Nodes exist | Runtime check | `doc.document.nodes.get(id).ok_or(NodeNotFound(id))` |
| P3: db_tx available | Compile-time conditional | `#[cfg(feature = "async-db")]` with fallback |

## Violation Examples (REQUIRED)
- VIOLATES P1: `bring_forward(doc_signal, history_signal)` with empty selected_items -- should produce `Err(BringForwardError::NoSelection)`
- VIOLATES P2: `bring_forward(doc_signal, history_signal)` with selected_items containing non-existent node "fake-id" -- should produce `Err(BringForwardError::NodeNotFound("fake-id"))`
- VIOLATES Q1: Calling with db_tx = None (when async-db enabled) -- should fallback to direct manipulation (not an error)
- VIOLATES Q5: Two calls in same millisecond could produce same op_id -- should use `Uuid::new_v4()` which is guaranteed unique

## Ownership Contracts
- `doc_signal: Signal<DiagramDocument>` - Read-only access to current state; clone taken for history push
- `history_signal: Signal<History>` - Exclusive write to push current document state
- `db_tx: Coroutine<EventEnvelope>` - Owned sender; send() transfers ownership of envelope

## Non-goals
- [ ] Implement BringToFront, SendBackward, SendToBack (future beads)
- [ ] Add validation UI for selection state
- [ ] Persist to different store backends (SQLite only for now)
- [ ] Add optimistic UI updates before store confirmation
