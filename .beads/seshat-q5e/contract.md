# Contract Specification: seshat-q5e (UI Dispatch: Ungroup Nodes)

## Context
- **Bead**: seshat-q5e
- **Feature**: Wire the Ctrl+Shift+G ungrouping command to construct `DomainOp::Ungroup` and dispatch to `db_tx`
- **Domain terms**:
  - `DomainOp::Ungroup { id: String }` - Domain operation to ungroup a subgraph by its ID
  - `db_tx` - Dioxus coroutine channel (`Coroutine<EventEnvelope>`) for dispatching event-sourced operations
  - `EventEnvelope` - Wraps a DomainOp with metadata (op_id, author, timestamp)
  - `DiagramDocument.editor_state.selected_items` - Immutable set of selected element IDs (`im::HashSet<String>`)
  - `DiagramDocument.document.nodes` - Immutable map of node IDs to Node entities
  - `KeyAction::Ungroup` - New keyboard action variant for Ctrl+Shift+G
- **Assumptions**:
  - `DomainOp::Ungroup { id: String }` already exists in the DomainOp enum (confirmed at envelope.rs:145-147)
  - `db_tx` context is available in the canvas component
  - The keyboard handler infrastructure exists in canvas.rs
  - The `ungroup_selection` function exists in core/grouping.rs for local fallback
- **Open questions**:
  - [Q1] Should ungroup work on multiple selected groups or just one?
  - [Q2] Should this replace or coexist with the local `ungroup_selection` mutation?

## EARS Specification
| ID | Requirement | Priority |
|----|-------------|----------|
| EARS-1 | WHEN the user presses Ctrl+Shift+G AND there is a selected subgraph/group, THEN the system SHALL construct `DomainOp::Ungroup { id }` with the group ID AND dispatch to db_tx | Must |
| EARS-2 | WHEN the user presses Ctrl+Shift+G AND there are no selected subgraphs, THEN the system SHALL do nothing (no-op) | Must |
| EARS-3 | WHEN the user presses Ctrl+Shift+G AND an input/textarea is focused, THEN the system SHALL NOT trigger ungroup | Must |
| EARS-4 | WHEN the db_tx coroutine is not available (None), THEN the system SHALL fall back to local mutation | Should |

## Preconditions
- [P1] **Key combo valid**: Ctrl+Shift+G must be pressed (enforced by keyboard mapping)
- [P2] **Not editing**: No input, textarea, or content-editable element is focused (handled by `is_editing_text` check)
- [P3] **Has selected subgraph**: `doc_signal.read().editor_state.selected_items` must contain at least one node ID that exists in `doc.document.nodes` and is a Subgraph kind
- [P4] **db_tx available**: `db_tx` context is `Some(coroutine)` (soft precondition - fallback exists)

## Postconditions
- [Q1] **Event dispatched**: Exactly one `EventEnvelope` with `DomainOp::Ungroup { id }` is sent to `db_tx` where `id` is the selected subgraph's ID
- [Q2] **Event envelope valid**: The dispatched `EventEnvelope` has:
  - `op_id`: Valid UUID v4 string
  - `operation`: `DomainOp::Ungroup { id: String }` with the subgraph's ID
  - `author`: `Author { id: "local-user", name: "Local User", email: None }`
  - `timestamp`: Current Unix epoch milliseconds (i64)
- [Q3] **No local mutation in happy path**: When db_tx is available, the document is NOT directly mutated (the event sourcing path)
- [Q4] **Selection cleared after dispatch**: After successful dispatch, `editor_state.selected_items` is cleared

## Invariants
- [I1] **Idempotent dispatch**: Pressing Ctrl+Shift+G multiple times sends multiple events (no deduplication at this layer)
- [I2] **Non-blocking**: The key handler returns immediately after sending to db_tx (async is handled by the coroutine)
- [I3] **No panic on missing node**: If a selected item ID doesn't exist in nodes, it is skipped (no error)

## Error Taxonomy
- **Error::NoSelection** - When there are no selected subgraphs to ungroup (soft error - should be no-op, not error)
- **Error::DbTxUnavailable** - When db_tx is None and local fallback fails (rare edge case)
- **Error::EventSendFailed** - When db_tx.send() returns an error (communication failure)
- **Error::InvalidSelection** - When selected item is not a subgraph (cannot ungroup non-subgraph nodes)

## Contract Signatures
```rust
/// Handle Ctrl+Shift+G key for node ungrouping via event sourcing
/// 
/// Returns: Result<bool, Error> where bool indicates if any nodes were ungrouped
fn handle_ungroup_key(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> Result<bool, Error>;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Ctrl+Shift+G pressed | Compile-time | Match in keyboard.rs: `("g" \| "G", true, true) => KeyAction::Ungroup` |
| P2: Not editing | Compile-time | JS-side check: `if (editing) return;` before sending to Rust |
| P3: Has selected subgraph | Runtime-checked | `if selected.is_empty() { return Ok(false); }` + node kind check |
| P4: db_tx available | Runtime-checked | `if let Some(tx) = &db_tx { ... } else { fallback }` |

## Violation Examples (REQUIRED)
- **VIOLATES P3**: Pressing Ctrl+Shift+G with empty selection (no nodes selected) -- should return `Ok(false)` (no-op, not error)
- **VIOLATES P3**: Pressing Ctrl+Shift+G with selected nodes that are not subgraphs -- should return `Ok(false)` (no-op, not error)
- **VIOLATES Q1**: Sending DomainOp with wrong ID -- should produce `Err(Error::InvalidNodeId)`
- **VIOLATES Q2**: EventEnvelope with missing timestamp -- should produce `Err(Error::InvalidEnvelope)`
- **VIOLATES Q4**: Selection not cleared after dispatch -- should produce `Err(Error::PostconditionViolation)`

## Ownership Contracts (Rust-specific)
- **doc_signal**: `Signal<DiagramDocument>` - Exclusive borrow for reading selected_items and nodes, may mutate if fallback
- **history_signal**: `Signal<History>` - Only needed for fallback path (local mutation)
- **db_tx**: `Option<Coroutine<EventEnvelope>>` - Borrowed, no ownership transfer, cloned for send
- **Clone policy**: No cloning of document state in happy path; only node ID extracted as String

## Ownership Decision
The function takes `Signal<DiagramDocument>` rather than `&mut DiagramDocument` because:
1. Dioxus Signals provide interior mutability with reactivity
2. The event sourcing path reads state without mutation
3. Fallback path uses `with_mut` for controlled mutation

## Implementation Phases

### Phase 1: Keyboard Infrastructure
1. Add `Ungroup` variant to `KeyAction` enum in `core/keyboard.rs`
2. Add Ctrl+Shift+G mapping in `map_key_to_action` function
3. Add test for keyboard mapping in `keyboard_tests.rs`

### Phase 2: UI Handler Wiring
1. Add match arm for `KeyAction::Ungroup` in `canvas.rs` key handler
2. Extract selected subgraph ID from `doc_signal`
3. Construct `EventEnvelope` with `DomainOp::Ungroup { id }`
4. Send to `db_tx` if available, fallback to local mutation

### Phase 3: Testing & Validation
1. Write integration test for keyboard shortcut
2. Write test for db_tx dispatch
3. Write test for fallback behavior

## Non-goals
- [ ] Grouping via keyboard (handled by Ctrl+G - separate feature)
- [ ] Undo/redo integration at this layer (handled by history system downstream)
- [ ] Multi-subgraph ungrouping (ungroups one at a time)

## Existing Code Reference
- `DomainOp::Ungroup` defined at: `diagram_tool/src/models/envelope.rs:145-147`
- `ungroup_selection` function at: `diagram_tool/src/core/grouping.rs:325-346`
- `KeyAction` enum at: `diagram_tool/src/core/keyboard.rs:3-15`
- `db_tx` usage example at: `diagram_tool/src/ui/canvas.rs:458-477` (NodeMove dispatch)
