# Contract Specification: seshat-4a8

## Context

- **Feature**: UI Dispatch - Bring to Front
- **Bead ID**: seshat-4a8
- **Description**: Wire the BringToFront toolbar button to construct `DomainOp::BringToFront` and dispatch to `db_tx` coroutine, rather than directly mutating the document signal.
- **Domain Terms**:
  - `DomainOp::BringToFront { ids: Vec<String> }` - Z-order operation variant
  - `EventEnvelope` - wrapper containing operation, author, timestamp
  - `db_tx` - `Coroutine<EventEnvelope>` for async dispatch to persistence layer
  - `Signal<DiagramDocument>` - Dioxus signal for reactive document state
  - `selected_items` - BTreeSet of currently selected node IDs

- **Assumptions**:
  - The db_tx coroutine is available via Dioxus context (like canvas.rs line 676)
  - The selected items must be filtered to node IDs only (not edges)
  - The existing `apply_bring_to_front` function handles the z-order logic correctly
  - Author metadata uses "local-user" placeholder (consistent with canvas.rs pattern)

- **Open Questions**:
  - Should the new dispatch function replace or complement the existing direct mutation?
  - What error handling when db_tx is None (not available in context)?

---

## Preconditions

- **P1**: The function must receive a valid `Signal<DiagramDocument>` that is not in a poisoned state.
- **P2**: The function must receive a valid `Signal<History>` for undo support.
- **P3**: The function must have access to `db_tx` via Dioxus context (`Option<Coroutine<EventEnvelope>>`).
- **P4**: There must be at least one selected node ID in `doc_signal.read().editor_state.selected_items` that exists in `doc_signal.read().document.nodes`.

---

## Postconditions

- **Q1**: If `db_tx` is `Some` and selection is non-empty, an `EventEnvelope` containing `DomainOp::BringToFront { ids }` must be sent to `db_tx.send()`.
- **Q2**: The `ids` field in `DomainOp::BringToFront` must contain only node IDs that exist in the document and are in the selected set.
- **Q3**: If `db_tx` is `None`, the function must fall back to direct mutation (preserving backward compatibility).
- **Q4**: If selection is empty, no operation should be dispatched and the function returns `false`.

---

## Invariants

- **I1**: The document's structural integrity must remain valid before and after dispatch.
- **I2**: The selected_items set must not be modified by the dispatch function.
- **I3**: The db_tx coroutine, if triggered, receives a valid `EventEnvelope` with non-empty `ids` in the operation.

---

## Error Taxonomy

- **DispatchError::NoDbTx**: When `db_tx` context is `None` and fallback fails
- **DispatchError::EmptySelection**: When no nodes are selected (should return early, not error)
- **DispatchError::SendFailed**: When `db_tx.send()` returns an error (rare, coroutine dropped)

---

## Contract Signatures

```rust
/// Dispatch BringToFront operation to db_tx coroutine
///
/// Returns `true` if dispatch succeeded (or fallback applied), `false` if no selection
pub fn dispatch_bring_to_front(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> bool
```

---

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Signal not poisoned | Runtime (Dioxus) | `Signal<T>` internal state |
| Selection non-empty | Runtime-checked | Early return `false` |
| Valid node IDs | Runtime-filter | Filter selected_items against document.nodes keys |
| db_tx available | Runtime-optional | `Option<Coroutine<T>>`, fallback to direct mutation |

---

## Violation Examples

- **VIOLATES P4**: `dispatch_bring_to_front(doc_signal, history_signal, Some(tx))` where `doc_signal.read().editor_state.selected_items` is empty -- should return `false` without sending to db_tx
- **VIOLATES Q1**: Calling with valid selection but `db_tx` is `Some(coroutine)` that was dropped -- should return `false` or handle error gracefully
- **VIOLATES Q2**: If somehow non-node IDs (edges) end up in the ids vector -- the z-order operation will silently skip them (documented behavior in `apply_z_order_to_ids`)

---

## Ownership Contracts

- **doc_signal**: Shared borrow via `Signal::read()`. No mutation to document structure in dispatch function (delegated to db_tx).
- **history_signal**: Not used in dispatch path (history managed by db_tx processing).
- **db_tx**: Owned `Option<Coroutine<EventEnvelope>>`. The coroutine is cloned when sending.

---

## Non-goals

- [ ] Implementing the actual z-order mutation logic (already exists in `apply_bring_to_front`)
- [ ] Changing the undo/redo system behavior
- [ ] Adding persistence layer logic
