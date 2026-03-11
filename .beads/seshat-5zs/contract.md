# Contract Specification: UI Dispatch - Edge Disconnect

## Context

- **Bead ID**: seshat-5zs
- **Feature**: Wire edge removal events to construct `DomainOp::EdgeDisconnect` and dispatch to `db_tx`
- **Domain terms**:
  - `DomainOp::EdgeDisconnect { id: String }` - Domain operation to remove an edge by ID
  - `db_tx` - Dioxus coroutine for transmitting `EventEnvelope` to the event store
  - `EventEnvelope` - Wrapper containing `op_id`, `operation` (DomainOp), `author`, `timestamp`
  - `DiagramDocument` - The in-memory document state with `nodes` and `edges` HashMaps
  - `selected_items` - Set of selected node/edge IDs in the editor state

- **Assumptions**:
  - `DomainOp::EdgeDisconnect` variant already exists in `envelope.rs` (line 125-127)
  - `parse_edge_disconnect` parsing function already exists (line 308-316)
  - `apply_edge_disconnect_checked` projection function already exists
  - The canvas event handlers have access to `db_tx` coroutine via context
  - Edge selection is tracked in `doc.editor_state.selected_items`

- **Open questions**:
  - Q1: What specific UI events should trigger edge disconnect? (Delete key on selected edge, context menu, drag to disconnect?)
  - Q2: Should the UI dispatch happen BEFORE or AFTER local document mutation?
  - Q3: Is there any confirmation dialog needed before disconnecting?

---

## EARS Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| EARS-1 | WHEN user presses Delete/Backspace key AND an edge is selected, THEN construct `DomainOp::EdgeDisconnect { id }` with the selected edge ID | Must |
| EARS-2 | WHEN edge disconnect operation is constructed, THEN dispatch `EventEnvelope` containing the operation to `db_tx` coroutine | Must |
| EARS-3 | WHEN `db_tx` is not available (None), THEN the operation should fail gracefully without panic | Must |
| EARS-4 | WHEN the selected edge ID does not exist in the document, THEN do not dispatch the operation | Must |
| EARS-5 | WHEN multiple edges are selected, THEN dispatch separate `EdgeDisconnect` operations for each edge | Should |

---

## Preconditions

| ID | Precondition | Enforcement Level | Type / Pattern |
|----|--------------|-------------------|----------------|
| P1 | Edge ID is present in `selected_items` | Runtime - check before dispatch | `selected_items.contains(&edge_id)` |
| P2 | Edge ID exists in `document.edges` | Runtime - check before dispatch | `document.edges.contains_key(&edge_id)` |
| P3 | `db_tx` coroutine is available | Runtime - Option type | `if let Some(tx) = &db_tx` |
| P4 | Edge ID is a non-empty string | Compile-time | `String` (cannot be empty in selection) |

---

## Postconditions

| ID | Postcondition | Notes |
|----|---------------|-------|
| Q1 | EventEnvelope with `DomainOp::EdgeDisconnect { id }` is sent to `db_tx` | Must be sent exactly once |
| Q2 | EventEnvelope contains valid `op_id` (UUID v4) | Each dispatch generates unique ID |
| Q3 | EventEnvelope contains valid `author` with id "local-user" | Default author for UI operations |
| Q4 | EventEnvelope contains valid Unix timestamp | Current time in milliseconds |
| Q5 | Selected edge is removed from `selected_items` after successful dispatch | UI state cleanup |

---

## Invariants

| ID | Invariant | Notes |
|----|-----------|-------|
| I1 | `db_tx` is always accessed via `Option<&Coroutine<EventEnvelope>>` pattern | No unwrap on coroutine |
| I2 | All dispatched `EventEnvelope` operations have valid, non-empty `op_id` | UUID format |
| I3 | Edge disconnect operations are dispatched atomically per edge | One envelope per edge ID |

---

## Error Taxonomy

| Error Variant | Condition | Handling |
|---------------|-----------|----------|
| `DispatchError::NoTx` | `db_tx` is None | Log warning, return early (no panic) |
| `DispatchError::EdgeNotFound` | Edge ID not in document.edges | Skip dispatch, clear from selection |
| `DispatchError::NotSelected` | Edge ID not in selected_items | Skip dispatch entirely |
| `DispatchError::SendFailed` | `tx.send()` returns Err | Log error, handle gracefully |

---

## Contract Signatures

```rust
/// Dispatches edge disconnect operation for a single edge ID
/// Returns Ok(()) on successful send, Err(DispatchError) on failure
fn dispatch_edge_disconnect(
    edge_id: &str,
    doc: &DiagramDocument,
    db_tx: &Option<Coroutine<EventEnvelope>>,
) -> Result<(), DispatchError>;

/// Dispatches edge disconnect operations for all selected edges
/// Returns count of successfully dispatched operations
fn dispatch_selected_edge_disconnects(
    doc: &DiagramDocument,
    db_tx: &Option<Coroutine<EventEnvelope>>,
) -> Result<usize, DispatchError>;
```

---

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|--------------|-------------------|----------------|
| P1: Edge in selection | Runtime check | `if selected_items.contains(&id)` |
| P2: Edge exists | Runtime check | `if document.edges.contains_key(&id)` |
| P3: db_tx available | Runtime Option | `if let Some(tx) = db_tx` |
| P4: Non-empty ID | Compile-time | `String` (selection uses non-empty IDs) |

---

## Violation Examples

### Precondition Violations

- **VIOLATES P1**: `dispatch_edge_disconnect("edge-not-selected", doc, db_tx)` where "edge-not-selected" is NOT in `selected_items` -- should produce `Err(DispatchError::NotSelected)`

- **VIOLATES P2**: `dispatch_edge_disconnect("nonexistent-edge", doc, db_tx)` where "nonexistent-edge" is NOT in `document.edges` -- should produce `Err(DispatchError::EdgeNotFound)`

- **VIOLATES P3**: `dispatch_edge_disconnect("edge-1", doc, &None)` with `db_tx = None` -- should produce `Err(DispatchError::NoTx)`

### Postcondition Violations

- **VIOLATES Q1**: After calling `dispatch_edge_disconnect`, verify `db_tx` received envelope -- if not sent, postcondition violated

---

## Ownership Contracts

- **Shared borrow**: `doc: &DiagramDocument` - read-only access to document state, no mutation
- **Shared borrow**: `db_tx: &Option<Coroutine<EventEnvelope>>` - no ownership, just borrowing the coroutine
- **No mutation**: This function does not mutate any document state; it only dispatches events
- **Clone policy**: `EventEnvelope` is cloned when sent to the channel (interior clone)

---

## Non-goals

- [ ] Implementing edge deletion via drag-to-disconnect visual feedback
- [ ] Implementing multi-select edge deletion via marquee selection
- [ ] Implementing undo/redo for edge disconnect operations (handled by history system)
- [ ] Implementing edge reconnection (separate DomainOp)
- [ ] Implementing edge deletion confirmation dialogs

---

## Implementation Phases

### Phase 1: Basic Dispatch
1. Add `DispatchError` enum to `canvas.rs` or new module
2. Implement `dispatch_edge_disconnect` function
3. Wire Delete/Backspace key handler to call dispatch when edge selected
4. Test with single edge selection

### Phase 2: Multi-Edge Support
1. Implement `dispatch_selected_edge_disconnects` for multiple edges
2. Update key handler to handle multiple selected edges
3. Test with multiple edge selection

### Phase 3: Error Handling
1. Add graceful handling for `DispatchError::SendFailed`
2. Add logging for dispatch failures
3. Verify no panics occur in edge cases
