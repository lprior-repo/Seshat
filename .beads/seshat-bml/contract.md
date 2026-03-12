# Contract Specification: seshat-bml (UI Dispatch: Node Deletion)

## Context
- **Bead**: seshat-bml
- **Feature**: Wire the Delete/Backspace key handler to construct `DomainOp::NodeDelete` for selected nodes and dispatch to `db_tx`
- **Domain terms**:
  - `DomainOp::NodeDelete { id: NodeId }` - Domain operation to delete a node by its ID
  - `db_tx` - Dioxus coroutine channel (`Coroutine<EventEnvelope>`) for dispatching event-sourced operations
  - `EventEnvelope` - Wraps a DomainOp with metadata (op_id, author, timestamp, revision)
  - `DiagramDocument.editor_state.selected_items` - Immutable set of selected element IDs (`im::HashSet<String>`)
  - `DiagramDocument.document.nodes` - Immutable map of node IDs to Node entities
  - `KeyAction::Delete` - Existing keyboard action variant for Delete/Backspace (no modifiers)
- **Assumptions**:
  - `DomainOp::NodeDelete { id: NodeId }` already exists in the DomainOp enum (confirmed at envelope.rs:125-127)
  - `db_tx` context is available in the canvas component
  - The keyboard handler infrastructure exists in canvas.rs
  - `KeyAction::Delete` is already mapped in keyboard.rs (line 43: `("Backspace" | "Delete", false, false) => KeyAction::Delete`)
  - The `dispatch_node_delete_batch` function already exists for dispatching to db_tx
  - The local `apply_delete_selected` function exists for fallback when db_tx is unavailable
- **Open questions**:
  - [Q1] Should deleted edges also dispatch `DomainOp::EdgeDisconnect` events, or is node deletion sufficient?
  - [Q2] Should we dispatch events for each node individually or batch into fewer operations?

## EARS Specification
| ID | Requirement | Priority |
|----|-------------|----------|
| EARS-1 | WHEN the user presses Delete OR Backspace AND there are selected nodes, THEN the system SHALL construct `DomainOp::NodeDelete { id }` for each selected node AND dispatch to db_tx | Must |
| EARS-2 | WHEN the user presses Delete OR Backspace AND there is no selection, THEN the system SHALL do nothing (no-op) | Must |
| EARS-3 | WHEN the user presses Delete OR Backspace AND an input/textarea is focused, THEN the system SHALL NOT trigger deletion | Must |
| EARS-4 | WHEN the db_tx coroutine is not available (None), THEN the system SHALL fall back to local mutation via `apply_delete_selected` | Should |

## Preconditions
- [P1] **Key valid**: Delete or Backspace key must be pressed without modifiers (enforced by keyboard mapping at keyboard.rs:43)
- [P2] **Not editing**: No input, textarea, or content-editable element is focused (handled by `is_editing_text` check)
- [P3] **Has selected nodes**: `doc_signal.read().editor_state.selected_items` must contain at least one node ID that exists in `doc.document.nodes`
- [P4] **db_tx available**: `db_tx` context is `Some(coroutine)` (soft precondition - fallback exists)

## Postconditions
- [Q1] **Events dispatched**: Exactly one `EventEnvelope` with `DomainOp::NodeDelete { id }` is sent to `db_tx` for each selected node ID
- [Q2] **Event envelope valid**: Each dispatched `EventEnvelope` has:
  - `op_id`: Valid UUID v4 string
  - `operation`: `DomainOp::NodeDelete { id: NodeId }` with the node's ID
  - `author`: `Author { id: "local-user", name: "Local User", email: None }`
  - `timestamp`: Current Unix epoch milliseconds (i64)
  - `revision`: Current document revision
- [Q3] **No local mutation in happy path**: When db_tx is available, the document is NOT directly mutated (the event sourcing path)
- [Q4] **Selection cleared after dispatch**: After successful dispatch, `editor_state.selected_items` is cleared

## Invariants
- [I1] **One event per node**: Each selected node results in exactly one `DomainOp::NodeDelete` dispatch (no batching)
- [I2] **Non-blocking**: The key handler returns immediately after sending to db_tx (async is handled by the coroutine)
- [I3] **Idempotent dispatch**: Pressing Delete/Backspace multiple times sends multiple events (no deduplication at this layer)
- [I4] **No panic on missing node**: If a selected item ID doesn't exist in nodes, it is skipped (no error)

## Error Taxonomy
- **Error::NoSelection** - When there are no selected nodes to delete (soft error - should be no-op, not error)
- **Error::DbTxUnavailable** - When db_tx is None and local fallback fails (rare edge case)
- **Error::EventSendFailed** - When db_tx.send() returns an error (communication failure)
- **Error::InvalidNodeId** - When selected node ID is malformed or doesn't exist
- **Error::PostconditionViolation** - When postconditions are not satisfied after operation
- **Error::DispatchIncomplete** - When dispatch count doesn't match expected (invariant I1 violation)

## Contract Signatures
```rust
/// Handle Delete/Backspace key for node deletion via event sourcing
/// 
/// Returns: Result<DispatchResult, Error> where DispatchResult indicates nodes affected and dispatches sent
fn handle_delete_key(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> Result<DispatchResult, Error>;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Delete/Backspace pressed | Compile-time | Match in keyboard.rs: `("Backspace" \| "Delete", false, false) => KeyAction::Delete` |
| P2: Not editing | Compile-time | JS-side check: `if (editing) return;` before sending to Rust |
| P3: Has selected nodes | Runtime-checked | `if node_ids.is_empty() { return Err(Error::NoSelection); }` |
| P4: db_tx available | Runtime-checked | `if let Some(tx) = &db_tx { ... } else { fallback }` |

## Violation Examples (REQUIRED)
- **VIOLATES P1**: Pressing Delete/Backspace while editing text (is_editing_text=true) -- should return `Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })` (blocked at keyboard layer)
- **VIOLATES P2**: Handler receives request while editing -- should not reach handler (JS-side blocks)
- **VIOLATES P3**: Pressing Delete with empty selection (no nodes selected) -- should return `Ok(DispatchResult { nodes_affected: 0, dispatches_sent: 0 })` (no-op, not error)
- **VIOLATES P4**: db_tx is None AND local fallback fails -- should produce `Err(Error::DbTxUnavailable)`
- **VIOLATES Q1**: Sending DomainOp with wrong node ID -- should produce `Err(Error::InvalidNodeId)`
- **VIOLATES Q2**: EventEnvelope with missing timestamp -- should produce `Err(Error::InvalidEnvelope)`
- **VIOLATES Q3**: Document is mutated directly when db_tx is available -- should produce `Err(Error::PostconditionViolation)`
- **VIOLATES Q4**: Selection not cleared after successful dispatch -- should produce `Err(Error::PostconditionViolation)`
- **VIOLATES I1**: Sending fewer events than selected nodes -- should produce `Err(Error::DispatchIncomplete)`

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

## Implementation Phases

### Phase 1: Handler Wiring
1. Modify the match arm in `canvas.rs` for `KeyAction::Delete` (currently at line 788-790)
2. Extract selected node IDs from `doc_signal`
3. Construct `EventEnvelope` with `DomainOp::NodeDelete { id }` for each selected node
4. Call `dispatch_node_delete_batch` to send to db_tx if available, fallback to local mutation

### Phase 2: Local Mutation Fallback
1. Ensure `apply_delete_selected` is called when db_tx is unavailable
2. Verify history is properly pushed before mutation

### Phase 3: Testing & Validation
1. Write integration test for keyboard shortcut
2. Write test for db_tx dispatch
3. Write test for fallback behavior

## Non-goals
- [ ] Edge deletion dispatch (handled separately by edge deletion logic)
- [ ] Undo/redo integration at this layer (handled by history system downstream)
- [ ] Container deletion with child reparenting (handled by projection layer)
- [ ] Locked node handling (handled by selection filtering in `selected_node_ids`)

## Existing Code Reference
- `DomainOp::NodeDelete` defined at: `diagram_tool/src/models/envelope.rs:125-127`
- `apply_delete_selected` function at: `diagram_tool/src/ui/commands.rs:520-562`
- `dispatch_node_delete_batch` function at: `diagram_tool/src/ui/dispatch/send/node.rs:45-66`
- `KeyAction::Delete` mapping at: `diagram_tool/src/core/keyboard.rs:43`
- Key handler in canvas at: `diagram_tool/src/ui/canvas.rs:788-790`
- `DispatchResult` struct at: `diagram_tool/src/ui/dispatch/errors.rs:31-37`
