# Contract Specification for seshat-9yw: UI Dispatch - Node Resize Drag

## EARS Requirements
- **Ubiquitous**: The UI SHALL sync visual boundaries to the backend via event sourcing when nodes are resized.
- **Event-driven**: The resize handle drag completion (pointerup with did_resize=true) SHALL trigger a `DomainOp::NodeResize` dispatch to db_tx.
- **Unwanted**: There SHALL be NO dispatch to db_tx while the resize handle is actively being dragged (only on completion).
- **E1**: The system SHALL dispatch `DomainOp::NodeResize` to the db_tx channel when a resize handle drag completes with `did_resize=true`.
- **E2**: The dispatched envelope SHALL contain the node ID, original dimensions, and new dimensions.
- **E3**: The system SHALL NOT dispatch when no actual resize occurred (did_resize=false).
- **E4**: The system SHALL handle db_tx unavailability gracefully (skip dispatch without crash).

## Context
- **Feature**: Wire the resize handle drag completion event to construct `DomainOp::NodeResize` and dispatch to db_tx.
- **Domain terms**:
  - **DomainOp::NodeResize**: Node operation that records the node ID, original bounds (x, y, width, height), and new bounds.
  - **db_tx**: Async channel (Coroutine) that persists `EventEnvelope` messages to the store bridge.
  - **ResizeHandle**: Enum variant (Nw, N, Ne, E, Se, S, Sw, W) indicating which handle was dragged.
  - **did_resize**: Boolean flag indicating whether actual resizing occurred (true) or just a click on handle (false).
  - **originals**: HashMap storing pre-resize dimensions per node (x, y, width, height).
- **Assumptions**:
  - The domain model needs `DomainOp::NodeResize` added to the DomainOp enum.
  - The resize completion event is already handled in the canvas interaction reducer.
  - The db_tx coroutine is available via Dioxus context (when `async-db` feature is enabled).
  - The existing resize logic in `interaction_reducer.rs` correctly updates node dimensions.
- **Open questions**:
  - Should we dispatch per-node or batch all resized nodes in one envelope? (Assuming: one envelope with all resized node IDs and their dimensions).
  - What is the exact trigger point for dispatch - is it in `finalize_motion_release` or a separate callback? (Assuming: after document is mutated with new dimensions).

## Preconditions
- [P1] `DidResizeOccurred`: The resize interaction must have resulted in actual dimension changes (`did_resize == true`).
- [P2] `DbTxAvailable`: The db_tx coroutine context must be available (requires `async-db` feature flag). If None, fallback to local-only.
- [P3] `ValidNodeIds`: The list of resized node IDs must be non-empty.

## Postconditions
- [Q1] `DispatchesToDbTx`: When preconditions are met, an `EventEnvelope` containing `DomainOp::NodeResize` is sent to db_tx.
- [Q2] `ContainsResizeData`: The dispatched operation contains:
  - `id`: The node ID that was resized
  - `original_x`, `original_y`, `original_width`, `original_height`: Pre-resize bounds
  - `x`, `y`, `width`, `height`: Post-resize bounds
- [Q3] `HasValidMetadata`: The dispatched envelope has valid `op_id`, `author`, and `timestamp` fields.
- [Q4] `NoDispatchOnNoResize`: When `did_resize == false`, no dispatch occurs (no-op).
- [Q5] `NoDispatchWhileDragging`: While the resize handle is actively being dragged (before pointerup), no intermediate dispatches occur.

## Invariants
- [I1] `NodeDimensionsUpdated`: After resize completion, the node's x, y, width, height match the new values in the dispatch.
- [I2] `DbTxNotBlocked`: Dispatching to db_tx must not block the UI thread.

## Error Taxonomy
- `DispatchError::DbTxUnavailable` - when db_tx context is None (only occurs when `async-db` feature is disabled).
- `DispatchError::NoResizeOccurred` - internal signal that no resize happened (should not be an error variant, just skip dispatch).
- Empty selection is NOT an error - returns early as a no-op.

## Contract Signatures
```rust
/// Dispatch NodeResize operation to db_tx after resize handle drag completes.
/// Returns Ok(true) if dispatched, Ok(false) if no resize (no-op), Err on failure.
pub fn dispatch_node_resize(
    doc_signal: Signal<DiagramDocument>,
    db_tx: Option<Coroutine<EventEnvelope>>,
    resized_node_ids: Vec<NodeId>,
) -> Result<bool, DispatchError>;

/// Internal: Construct DomainOp::NodeResize from original and new bounds.
pub fn construct_node_resize_operation(
    node_id: NodeId,
    original_bounds: (f64, f64, f64, f64),
    new_bounds: (f64, f64, f64, f64),
) -> DomainOp;
```

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| `DidResizeOccurred` | Runtime check | Boolean flag from interaction state |
| `DbTxAvailable` | Conditional (feature flag) | `Option<Coroutine<EventEnvelope>>` - None when `async-db` disabled |
| `ValidNodeIds` | Runtime check | Non-empty Vec check before dispatch |

## Violation Examples
- VIOLATES P1: Calling `dispatch_node_resize` with empty `resized_node_ids` -- should return `Ok(false)` (no-op, not an error)
- VIOLATES P2: `dispatch_node_resize(doc_signal, None)` when `async-db` is enabled -- should return `Err(DispatchError::DbTxUnavailable)` or fallback to local
- VIOLATES Q2: Dispatched `DomainOp::NodeResize` has wrong dimensions -- test fails
- VIOLATES Q3: Dispatched envelope has empty `op_id` or invalid timestamp -- test fails
- VIOLATES Q4: Dispatch occurs when `did_resize == false` -- violates no-op requirement

## Ownership Contracts
- `dispatch_node_resize(doc_signal, db_tx, resized_node_ids)`:
  - Shared borrow: Reads `doc_signal` to get new node dimensions
  - Ownership: The `EventEnvelope` is moved into db_tx via `send()`
- `construct_node_resize_operation`:
  - Ownership: Takes node_id and bounds by value, returns owned DomainOp

## Implementation Phases
1. **Phase 1**: Add `DomainOp::NodeResize { id, original_x, original_y, original_width, original_height, x, y, width, height }` to DomainOp enum in envelope.rs
2. **Phase 2**: Implement `construct_node_resize_operation` helper function
3. **Phase 3**: Implement `dispatch_node_resize` function that:
   - Checks if any nodes actually resized (compares original vs new bounds)
   - Constructs EventEnvelope with DomainOp::NodeResize
   - Sends to db_tx if available
4. **Phase 4**: Wire the resize completion in interaction_reducer to call dispatch_node_resize after document mutation
5. **Phase 5**: Test the full pipeline (resize handle drag -> document update -> db_tx dispatch)

## Non-goals
- Modifying core resize geometry logic (already implemented in interaction_reducer.rs)
- Adding undo/redo for resize (handled by existing History system)
- Real-time sync during drag (only dispatch on completion)
