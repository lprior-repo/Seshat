# Contract Specification for seshat-4pr: UI Dispatch - Send to Back

## EARS Requirements
- **E1**: The system SHALL dispatch `DomainOp::SendToBack` to the db_tx channel when the "To Back" toolbar button is clicked and nodes are selected.
- **E2**: The dispatched envelope SHALL contain all currently selected node IDs.
- **E3**: The system SHALL NOT panic when no nodes are selected (no-op behavior).
- **E4**: The system SHALL handle db_tx unavailability gracefully (skip dispatch without crash).

## Context
- **Feature**: Wire the SendToBack toolbar button to construct `DomainOp::SendToBack` and dispatch to db_tx.
- **Domain terms**:
  - **DomainOp::SendToBack**: Z-order operation that moves selected nodes to the back of the render order.
  - **db_tx**: Async channel (Coroutine) that persists `EventEnvelope` messages to the store bridge.
  - **Toolbar button**: UI trigger in the toolbar that invokes the action.
  - **Selected node IDs**: The set of node IDs currently selected in the document.
- **Assumptions**:
  - The `apply_send_to_back` function in commands.rs already implements the z-order logic correctly.
  - The domain model already supports `DomainOp::SendToBack { ids: Vec<String> }`.
  - The db_tx coroutine is available via Dioxus context (when `async-db` feature is enabled).
- **Open questions**:
  - Should the dispatch happen before or after the UI signal update? (Assuming: dispatch after local UI update for responsiveness).

## Preconditions
- [P1] `SelectionNotEmpty`: The document must have at least one node selected (non-empty `selected_items` in editor_state).
- [P2] `DbTxAvailable`: The db_tx coroutine context must be available (requires `async-db` feature flag).

## Postconditions
- [Q1] `DispatchesToDbTx`: When preconditions are met, an `EventEnvelope` containing `DomainOp::SendToBack` is sent to db_tx.
- [Q2] `ContainsSelectedIds`: The dispatched operation contains all currently selected node IDs.
- [Q3] `HasValidMetadata`: The dispatched envelope has valid `op_id`, `author`, and `timestamp` fields.
- [Q4] `NoPanicOnEmptySelection`: When selection is empty, the action returns early without panicking (no-op).

## Invariants
- [I1] `ZOrderPreserved`: After the operation, the selected nodes appear behind all other nodes in the same layer.
- [I2] `DbTxNotBlocked`: Dispatching to db_tx must not block the UI thread.

## Error Taxonomy
- `DispatchError::DbTxUnavailable` - when db_tx context is None (only occurs when `async-db` feature is disabled).
- Empty selection is NOT an error - returns `Ok(false)` as a no-op signal.

## Contract Signatures
```rust
/// Dispatch SendToBack operation to db_tx.
/// Returns Ok(true) if dispatched, Ok(false) if no selection (no-op), Err on failure.
pub fn dispatch_send_to_back(
    doc_signal: Signal<DiagramDocument>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> Result<bool, DispatchError>;

/// Wrapper action for toolbar invocation.
/// Follows pattern of other toolbar actions (bring_forward, send_backward, etc.)
pub fn send_to_back(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
);
```

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| `SelectionNotEmpty` | Runtime check | Check `doc_signal.read().editor_state.selected_items` before dispatch |
| `DbTxAvailable` | Conditional (feature flag) | `Option<Coroutine<EventEnvelope>>` - None when `async-db` disabled |

## Violation Examples
- VIOLATES P1: `dispatch_send_to_back(doc_signal, Some(db_tx))` where selection is empty -- should return `Ok(false)` (no-op, not an error)
- VIOLATES P2: `dispatch_send_to_back(doc_signal, None)` when `async-db` is enabled -- should return `Err(DispatchError::DbTxUnavailable)`
- VIOLATES Q2: Dispatched `DomainOp::SendToBack` contains only a subset of selected IDs -- test fails.
- VIOLATES Q3: Dispatched envelope has empty `op_id` or invalid timestamp -- test fails.

## Ownership Contracts
- `dispatch_send_to_back(doc_signal, db_tx)`:
  - Shared borrow: Reads `doc_signal` to extract selected node IDs.
  - Ownership: The `EventEnvelope` is moved into db_tx via `send()`.
- `send_to_back(doc_signal, history_signal)`:
  - Follows existing pattern: Calls `apply_send_to_back` (which mutates doc_signal), then dispatches to db_tx.

## Implementation Phases
1. **Phase 1**: Add `dispatch_send_to_back` function that constructs `DomainOp::SendToBack` and sends to db_tx.
2. **Phase 2**: Update `send_to_back` action in `toolbar/actions.rs` to call dispatch after local update.
3. **Phase 3**: Test that the full pipeline works (toolbar -> action -> db_tx -> store bridge).

## Non-goals
- Modifying the core z-order logic (already implemented in `apply_send_to_back`).
- Adding new domain operations (SendToBack already exists).
