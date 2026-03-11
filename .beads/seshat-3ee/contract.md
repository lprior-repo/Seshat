# Contract Specification: seshat-3ee

## Context

- **Feature**: UI Dispatch: Delete Key
- **Bead ID**: seshat-3ee
- **Domain terms**:
  - `db_tx`: Dioxus Coroutine for dispatching `EventEnvelope` to the backend persistence layer
  - `DomainOp::NodeDelete`: Domain operation representing node deletion with `id: String` field
  - `EventEnvelope`: Wrapper containing `op_id`, `operation`, `author`, and `timestamp`
  - `DiagramDocument`: The document model containing nodes, edges, and editor state
  - `Signal<T>`: Dioxus reactive signal type for state management
  - `selected_items`: Set of selected node/edge IDs in `editor_state`
- **Assumptions**:
  - The delete key handler already exists in `canvas.rs` and calls `apply_delete_selected`
  - `db_tx` is obtained via `use_context::<Option<Coroutine<EventEnvelope>>>()`
  - Each selected node requires a separate `DomainOp::NodeDelete` event
- **Open questions**:
  - Should edges connected to deleted nodes also generate `EdgeDisconnect` events?
  - Is there a batch operation needed, or is one-at-a-time sufficient?

## EARS (Requirements)

| Type | Statement |
|------|-----------|
| **Ubiquitous** | UI shall notify backend when nodes are deleted |
| **Event-driven** | Delete key press with non-empty selection triggers NodeDelete dispatch |
| **Unwanted** | No dispatch occurs if selection is empty |

## Preconditions

- **P1 (Compile-time)**: `Signal<DiagramDocument>` must be a valid non-null signal
- **P2 (Runtime)**: Selection must contain at least one node ID to delete
- **P3 (Runtime)**: The `db_tx` context must be available (graceful degradation: log warning if None)

## Postconditions

- **Q1**: Each selected node ID in `selected_items` results in exactly one `DomainOp::NodeDelete { id }` sent to `db_tx`
- **Q2**: The `EventEnvelope` contains valid `op_id` (UUID), `author` (with id "local-user"), and `timestamp` (Unix millis)
- **Q3**: The local document state is mutated (nodes removed from document) via existing `apply_delete_selected` logic
- **Q4**: If `db_tx` is `None`, a warning is logged but local delete still succeeds

## Invariants

- **I1**: After delete dispatch, the set of dispatched `NodeDelete` operations equals the set of selected nodes that existed at delete time
- **I2**: `db_tx` send failures are logged but do not propagate as errors to the UI

## Error Taxonomy

| Error Variant | Condition |
|---------------|------------|
| `Error::NoSelection` | Delete key pressed with empty selection |
| `Error::DbTxUnavailable` | `db_tx` context is `None` (warning-level, not failure) |
| `Error::SendFailed` | `db_tx.send()` returns Err (e.g., channel closed) |

## Contract Signatures

```rust
/// Dispatches NodeDelete operations for selected nodes to db_tx
/// Returns: Result<DispatchResult, Error>
pub fn dispatch_delete_to_backend(
    doc_signal: Signal<DiagramDocument>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> Result<DispatchResult, Error>;

/// DispatchResult contains counts of dispatched operations
pub struct DispatchResult {
    pub nodes_deleted: usize,
    pub dispatches_sent: usize,
}
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|--------------|--------------------|-----------------|
| Signal valid | Compile-time | `Signal<DiagramDocument>` (Dioxus type) |
| Selection non-empty | Runtime-check | `if selected.is_empty() { return Err(NoSelection) }` |
| db_tx available | Runtime-check | `Option<Coroutine<...>>` with graceful fallback |
| Valid UUID for op_id | Compile-time | `uuid::Uuid::new_v4()` (guaranteed valid) |
| Valid timestamp | Compile-time | `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()` |

## Violation Examples (REQUIRED)

- **VIOLATES P2 (NoSelection)**: `dispatch_delete_to_backend(doc_signal, db_tx)` with `selected_items = {}`  
  Expected: `Err(Error::NoSelection)` -- does NOT dispatch, does NOT modify document

- **VIOLATES Q1 (Under-dispatch)**: `dispatch_delete_to_backend(doc_signal, db_tx)` with 3 selected nodes but only 2 envelopes sent  
  Expected: `Err(Error::DispatchIncomplete)` -- all 3 must be sent or operation fails

- **VIOLATES Q1 (Over-dispatch)**: `dispatch_delete_to_backend(doc_signal, db_tx)` with 1 selected node but 2 envelopes sent  
  Expected: `Err(Error::DispatchIncomplete)` -- exact count match required

- **VIOLATES Q2 (Invalid envelope)**: Envelope sent with missing `op_id` or `timestamp`  
  Expected: Compile-time guarantee via `EventEnvelope` struct construction

## Ownership Contracts

- **`doc_signal: Signal<DiagramDocument>`**: Exclusive borrow via `with_mut()` for local document mutation
- **`db_tx: Option<Coroutine<EventEnvelope>>`**: Shared borrow (clone), no ownership transfer
- **`EventEnvelope`**: Value type, cloned into channel, no ownership retained

## Mutation Postconditions

For `doc_signal.with_mut(|doc| { ... })`:
- `doc.document.nodes` - nodes matching selected IDs are removed
- `doc.document.edges` - edges connected to deleted nodes are removed
- `doc.editor_state.selected_items` - cleared after successful delete
- `doc.revision` - incremented

## Non-goals

- [ ] Implementing undo/redo for delete operations (handled elsewhere)
- [ ] Batch deletion optimization (one-at-a-time is acceptable)
- [ ] Edge disconnection events (out of scope for this bead)
- [ ] Remote sync conflict resolution (handled by sync layer)
