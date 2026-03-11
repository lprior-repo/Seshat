# Contract Specification: seshat-5y5 (UI Dispatch: Backspace Key)

## Context
- **Bead**: seshat-5y5
- **Feature**: Wire the Backspace/Delete key handler to construct `DomainOp::NodeDelete` for selected nodes and dispatch to `db_tx`
- **Domain terms**:
  - `DomainOp::NodeDelete { id: String }` - Domain operation to delete a node by its ID
  - `db_tx` - Dioxus coroutine channel (`Coroutine<EventEnvelope>`) for dispatching event-sourced operations
  - `EventEnvelope` - Wraps a DomainOp with metadata (op_id, author, timestamp)
  - `DiagramDocument.editor_state.selected_items` - Immutable set of selected element IDs (`im::HashSet<String>`)
  - `DiagramDocument.document.nodes` - Immutable map of node IDs to Node entities
- **Assumptions**:
  - The keyboard handler for Delete/Backspace is already in place (lines 787-788 in canvas.rs call `apply_delete_selected`)
  - `DomainOp::NodeDelete` variant already exists in the DomainOp enum
  - `db_tx` context is available in the canvas component
  - The existing `apply_delete_selected` function can be replaced or refactored
- **Open questions**:
  - [Q1] Should both Delete and Backspace keys trigger the same behavior? (Current code handles both)
  - [Q2] Should this replace or coexist with the local `apply_delete_selected` mutation?
  - [Q3] How should edge deletion be handled when a node is deleted? (Currently handled by cascade in apply_delete_selected)

## EARS Specification
| ID | Requirement | Priority |
|----|-------------|----------|
| EARS-1 | WHEN the user presses Delete or Backspace key AND there are selected nodes, THEN the system SHALL construct one DomainOp::NodeDelete for each selected node AND dispatch to db_tx | Must |
| EARS-2 | WHEN the user presses Delete or Backspace key AND there are no selected nodes, THEN the system SHALL do nothing (no-op) | Must |
| EARS-3 | WHEN the user presses Delete or Backspace key AND an input/textarea is focused, THEN the system SHALL NOT trigger node deletion | Must |
| EARS-4 | WHEN the db_tx coroutine is not available (None), THEN the system SHALL fall back to local mutation | Should |

## Preconditions
- [P1] **Key event valid**: The key event must be either "Delete" or "Backspace" (already enforced by match statement)
- [P2] **Not editing**: No input, textarea, or content-editable element is focused (already handled by `editing` check)
- [P3] **Has selected nodes**: `doc_signal.read().editor_state.selected_items` must contain at least one node ID that exists in `doc.document.nodes`
- [P4] **db_tx available**: `db_tx` context is `Some(coroutine)` (soft precondition - fallback exists)

## Postconditions
- [Q1] **Event dispatched**: For each selected node ID that exists in nodes, exactly one `EventEnvelope` with `DomainOp::NodeDelete { id }` is sent to `db_tx`
- [Q2] **Event envelope valid**: Each dispatched `EventEnvelope` has:
  - `op_id`: Valid UUID v4 string
  - `operation`: `DomainOp::NodeDelete { id: String }` with the node's ID
  - `author`: `Author { id: "local-user", name: "Local User", email: None }`
  - `timestamp`: Current Unix epoch milliseconds (i64)
- [Q3] **No local mutation in happy path**: When db_tx is available, the document is NOT directly mutated (the event sourcing path)
- [Q4] **Selection cleared after dispatch**: After successful dispatch, `editor_state.selected_items` is cleared

## Invariants
- [I1] **Idempotent dispatch**: Pressing Delete/Backspace multiple times sends multiple events (no deduplication at this layer)
- [I2] **Non-blocking**: The key handler returns immediately after sending to db_tx (async is handled by the coroutine)
- [I3] **No panic on missing node**: If a selected item ID doesn't exist in nodes, it is skipped (no error)

## Error Taxonomy
- **Error::NoSelection** - When there are no selected nodes to delete (soft error - should be no-op, not error)
- **Error::DbTxUnavailable** - When db_tx is None and local fallback fails (rare edge case)
- **Error::EventSendFailed** - When db_tx.send() returns an error (communication failure)

## Contract Signatures
```rust
/// Handle Delete/Backspace key for node deletion via event sourcing
/// 
/// Returns: Result<bool, Error> where bool indicates if any nodes were deleted
fn handle_delete_key(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> Result<bool, Error>;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Key is Delete/Backspace | Compile-time | Match statement: `match key { "Delete" \| "Backspace" => ... }` |
| P2: Not editing | Compile-time | JS-side check: `if (editing) return;` before sending to Rust |
| P3: Has selected nodes | Runtime-checked | `if selected.is_empty() { return Ok(false); }` |
| P4: db_tx available | Runtime-checked | `if let Some(tx) = &db_tx { ... } else { fallback }` |

## Violation Examples (REQUIRED)
- **VIOLATES P3**: Pressing Delete with empty selection (no nodes selected) -- should return `Ok(false)` (no-op, not error)
- **VIOLATES Q1**: Sending DomainOp with wrong ID -- should produce `Err(Error::InvalidNodeId)`
- **VIOLATES Q2**: EventEnvelope with missing timestamp -- should produce `Err(Error::InvalidEnvelope)`
- **VIOLATES Q4**: Selection not cleared after dispatch -- should produce `Err(Error::PostconditionViolation)`

## Ownership Contracts (Rust-specific)
- **doc_signal**: `Signal<DiagramDocument>` - Exclusive borrow for reading selected_items and nodes, may mutate if fallback
- **history_signal**: `Signal<History>` - Only needed for fallback path (local mutation)
- **db_tx**: `Option<Coroutine<EventEnvelope>>` - Borrowed, no ownership transfer, cloned for send
- **Clone policy**: No cloning of document state in happy path; only node IDs extracted as Strings

## Ownership Decision
The function takes `Signal<DiagramDocument>` rather than `&mut DiagramDocument` because:
1. Dioxus Signals provide interior mutability with reactivity
2. The event sourcing path reads state without mutation
3. Fallback path uses `with_mut` for controlled mutation

## Non-goals
- [ ] Edge deletion via keyboard (handled by cascade after node deletion)
- [ ] Undo/redo integration at this layer (handled by history system downstream)
- [ ] Multi-select edge deletion via keyboard (future enhancement)
