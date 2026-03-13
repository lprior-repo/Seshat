# Contract Specification for seshat-shb: UI Dispatch - Z-Index Layering

## EARS Requirements
- **E1**: The system SHALL dispatch `DomainOp::BringToFront` to the db_tx channel when the "To Front" toolbar button is clicked and nodes are selected.
- **E2**: The system SHALL dispatch `DomainOp::SendToBack` to the db_tx channel when the "To Back" toolbar button is clicked and nodes are selected.
- **E3**: The dispatched envelope SHALL contain all currently selected node IDs.
- **E4**: The system SHALL NOT panic when no nodes are selected (no-op behavior).
- **E5**: The system SHALL handle db_tx unavailability gracefully (skip dispatch without crash).

## Context
- **Feature**: Wire the BringToFront/SendToBack toolbar buttons to construct `DomainOp::ZOrder` actions and dispatch to db_tx.
- **Domain terms**:
  - **DomainOp::BringToFront**: Z-order operation that moves selected nodes to the front (highest z-index) of the render order.
  - **DomainOp::SendToBack**: Z-order operation that moves selected nodes to the back (lowest z-index) of the render order.
  - **ZOrderOp**: Internal enum (BringForward, SendBackward, BringToFront, SendToBack) used in commands.rs.
  - **db_tx**: Async channel (Coroutine<EventEnvelope>) that persists EventEnvelope messages to the store bridge.
  - **Toolbar button**: UI trigger in the toolbar that invokes the action.
  - **Selected node IDs**: The set of node IDs currently selected in the document.
- **Assumptions**:
  - The `apply_bring_to_front` and `apply_send_to_back` functions in commands.rs already implement the z-order logic correctly.
  - The domain model already supports `DomainOp::BringToFront { ids: Vec<NodeId> }` and `DomainOp::SendToBack { ids: Vec<NodeId> }`.
  - The db_tx coroutine is available via Dioxus context (when `async-db` feature is enabled).
  - Related beads seshat-4a8 (BringToFront) and seshat-4pr (SendToBack) define similar contracts.
- **Open questions**:
  - Should the dispatch happen before or after the UI signal update? (Assuming: dispatch after local UI update for optimistic UI responsiveness).

## Preconditions
- [P1] `SelectionNotEmpty`: The document must have at least one node selected (non-empty `editor_state.selected_items`).
- [P2] `DbTxAvailable`: The db_tx coroutine context must be available (requires `async-db` feature flag).

## Postconditions
- [Q1] `BringToFrontDispatchesToDbTx`: When preconditions are met for BringToFront, an `EventEnvelope` containing `DomainOp::BringToFront` is sent to db_tx.
- [Q2] `SendToBackDispatchesToDbTx`: When preconditions are met for SendToBack, an `EventEnvelope` containing `DomainOp::SendToBack` is sent to db_tx.
- [Q3] `ContainsSelectedIds`: The dispatched operation contains all currently selected node IDs.
- [Q4] `HasValidMetadata`: The dispatched envelope has valid `op_id` (UUID), `author`, and `timestamp` fields.
- [Q5] `NoPanicOnEmptySelection`: When selection is empty, the action returns early without panicking (no-op).

## Invariants
- [I1] `ZOrderPreserved`: After BringToFront, selected nodes appear at highest z-index. After SendToBack, selected nodes appear at lowest z-index.
- [I2] `DbTxNotBlocked`: Dispatching to db_tx must not block the UI thread.

## Error Taxonomy
- `DispatchError::WalDisconnected` - when db_tx is None (WAL unavailable).
- `DispatchError::ChannelMissing` - when db_tx channel is missing.
- Empty selection is NOT an error - returns `DispatchResult { nodes_affected: 0, dispatches_sent: 0 }` as a no-op.

## Contract Signatures

### Dispatch Functions (pure dispatch logic)
```rust
/// Dispatch BringToFront operation to db_tx.
/// Returns Ok(DispatchResult) with nodes_affected and dispatches_sent.
/// Returns Err(DispatchError::WalDisconnected) if db_tx is None.
pub fn dispatch_bring_to_front(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError>;

/// Dispatch SendToBack operation to db_tx.
/// Returns Ok(DispatchResult) with nodes_affected and dispatches_sent.
/// Returns Err(DispatchError::WalDisconnected) if db_tx is None.
pub fn dispatch_send_to_back(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError>;
```

### Toolbar Action Functions (UI wiring)
```rust
/// Wrapper action for BringToFront toolbar button.
/// Extracts selected IDs, dispatches to db_tx, applies local mutation.
pub fn bring_to_front(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
);

/// Wrapper action for SendToBack toolbar button.
/// Extracts selected IDs, dispatches to db_tx, applies local mutation.
pub fn send_to_back(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
);
```

### Envelope Creation Functions
```rust
/// Create an EventEnvelope for a BringToFront operation.
pub fn create_bring_to_front_envelope(ids: Vec<String>) -> EventEnvelope;

/// Create an EventEnvelope for a SendToBack operation.
pub fn create_send_to_back_envelope(ids: Vec<String>) -> EventEnvelope;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| `SelectionNotEmpty` | Runtime check | Check `doc_signal.read().editor_state.selected_items.is_empty()` before dispatch |
| `DbTxAvailable` | Runtime (conditional) | `Option<Coroutine<EventEnvelope>>` - None when WAL disconnected |
| `node_ids` non-empty | Runtime check | Early return with `DispatchResult { 0, 0 }` if empty slice |

## Violation Examples
- VIOLATES P1: `dispatch_bring_to_front(&Some(db_tx), &[])` with empty ids -- returns `Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })` (no-op)
- VIOLATES P2: `dispatch_bring_to_front(&None, &["node1"])` when db_tx unavailable -- returns `Err(DispatchError::WalDisconnected)`
- VIOLATES Q3: Dispatched `DomainOp::BringToFront` contains only subset of selected IDs -- test fails
- VIOLATES Q4: Dispatched envelope has empty `op_id` or zero timestamp -- test fails

## Ownership Contracts

### dispatch_bring_to_front / dispatch_send_to_back
- **Input**: `db_tx: &Option<Coroutine<EventEnvelope>>` - shared borrow of channel, no ownership transferred
- **Input**: `node_ids: &[String]` - shared borrow of slice, no ownership transferred
- **Output**: Creates new `EventEnvelope` (owned), moves into db_tx via `send()`
- **Mutation**: No mutation of input parameters

### bring_to_front / send_to_back (toolbar actions)
- **Input**: `doc_signal: Signal<DiagramDocument>` - exclusive borrow via Signal::read()
- **Input**: `history_signal: Signal<History>` - for undo/redo support
- **Input**: `db_tx: Option<Coroutine<EventEnvelope>>` - ownership passed to dispatch function
- **Mutation**: `doc_signal` is mutated via `apply_bring_to_front` / `apply_send_to_back`
- **Ownership**: The selected node IDs are cloned from doc_signal to create the envelope

## Implementation Phases
1. **Phase 1**: Ensure `create_bring_to_front_envelope` and `create_send_to_back_envelope` exist in dispatch/create.rs
2. **Phase 2**: Ensure `dispatch_bring_to_front` and `dispatch_send_to_back` exist in dispatch/send/zorder.rs
3. **Phase 3**: Wire toolbar buttons in toolbar.rs to call `bring_to_front` and `send_to_back` actions
4. **Phase 4**: Verify full pipeline: toolbar -> action -> db_tx -> store bridge

## Non-goals
- Modifying the core z-order logic (already implemented in `apply_z_order_to_ids`).
- Adding new domain operations (BringToFront/SendToBack already exist in DomainOp enum).
- Implementing BringForward/SendBackward (separate beads: seshat-lt0, seshat-84y).
