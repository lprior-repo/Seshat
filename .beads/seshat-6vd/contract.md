# Contract Specification: seshat-6vd

## Context

- **Feature**: UI Dispatch - Group Nodes (Ctrl+G keyboard shortcut)
- **Bead ID**: seshat-6vd
- **Domain Terms**:
  - `DomainOp::Group { ids: Vec<String> }` - operation to group selected nodes into a subgraph
  - `db_tx` - Dioxus coroutine channel for dispatching `EventEnvelope` to the backend
  - `selected_items` - `im::HashSet<String>` containing currently selected node IDs
  - `EventEnvelope` - wrapper containing `op_id`, `operation`, `author`, and `timestamp`
  - `Ctrl+G` - keyboard shortcut triggering group operation
- **Assumptions**:
  - Canvas keyboard event handler is already in place (`canvas.rs` lines 650-858)
  - `ctrl_pressed` signal tracks Ctrl key state
  - `doc_signal` provides access to document state including `selected_items`
  - `db_tx` context provides channel for dispatching to backend
  - Backend already handles `DomainOp::Group` (projection/reducer exists in `group_ops.rs`)
- **Open Questions**:
  - Should there be a toast/notification on success/failure?
  - Should selection be cleared after grouping?
  - What is the expected behavior when some selected nodes are already in a group?

---

## EARS Analysis

### Ubiquitous Language
- **UI shall notify backend of hierarchy changes**: The canvas UI dispatches `EventEnvelope` with `DomainOp::Group` to `db_tx` channel whenever the user triggers grouping.

### Event-Driven Behavior
- **Ctrl+G with selection > 1 triggers Group**: When user presses Ctrl+G and `selected_items.len() > 1`, construct and dispatch `DomainOp::Group { ids }`.

### Unwanted Behavior
- **No dispatch if < 2 nodes selected**: If `selected_items.len() < 2`, do NOT dispatch any operation (silent no-op or optional toast warning).

---

## Preconditions

| ID | Description | Enforcement Level | Type/Pattern |
|----|-------------|-------------------|--------------|
| P1 | `ctrl_pressed` signal must be `true` when processing Ctrl+G | Compile-time via Dioxus signal | `Signal<bool>` |
| P2 | `selected_items.len() >= 2` to form a valid group | Runtime check | `if selected.len() < 2 { return; }` |
| P3 | `db_tx` context must be available (Some) | Runtime check | `if let Some(tx) = &db_tx { ... }` |
| P4 | All IDs in `selected_items` must be valid node IDs (non-empty strings) | Compile-time via document model | `NodeId::new()` validates |
| P5 | Canvas must not be in editing mode (input/textarea focused) | Runtime check | Existing keyboard guard in `canvas.rs` |

---

## Postconditions

| ID | Description | Enforcement Level |
|----|-------------|-------------------|
| Q1 | If `selected_items.len() >= 2`, an `EventEnvelope` with `DomainOp::Group { ids }` is sent to `db_tx` | Runtime: verify `tx.send(...)` called |
| Q2 | The `ids` vector in `DomainOp::Group` contains all node IDs from `selected_items` | Runtime: verify vector contents |
| Q3 | Selection state remains unchanged after dispatch (UI state not mutated) | Runtime: no mutation of `selected_items` |
| Q4 | If precondition fails (P2), no error is raised; operation is silently ignored | Runtime: early return |

---

## Invariants

| ID | Description |
|----|-------------|
| I1 | `db_tx` channel exists in canvas context as `Option<Coroutine<EventEnvelope>>` |
| I2 | Keyboard event handling runs in `use_effect` with proper cleanup |
| I3 | Ctrl+G dispatch is idempotent (repeated presses create multiple group operations) |
| I4 | No panics occur regardless of selection state |

---

## Error Taxonomy

Since this is a UI event handler with graceful degradation, errors are handled by silent no-op:

| Error Variant | Condition | Handling |
|---------------|-----------|----------|
| `Error::NoSelection` | `selected_items.len() < 2` | Silent no-op, optionally show toast |
| `Error::ChannelUnavailable` | `db_tx` is `None` | Silent no-op, log to console |
| `Error::EmptySelection` | `selected_items.is_empty()` | Silent no-op |

**Note**: These are NOT `Result<T, Error>` returns - they are early returns with no error propagation.

---

## Contract Signatures

```rust
// UI-side keyboard handler (in canvas.rs)
fn handle_ctrl_g_group(
    doc_signal: &Signal<DiagramDocument>,
    db_tx: &Option<Coroutine<EventEnvelope>>,
) {
    // Precondition checks
    // Postcondition: dispatch or no-op
}
```

```rust
// Domain operation construction
fn construct_group_operation(selected_ids: Vec<String>) -> EventEnvelope {
    EventEnvelope {
        op_id: Uuid::new_v4().to_string(),
        operation: DomainOp::Group { ids: selected_ids },
        author: Author {
            id: "local-user".to_string(),
            name: "Local User".to_string(),
            email: None,
        },
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    }
}
```

---

## Violation Examples (REQUIRED)

### Precondition Violations

- **VIOLATES P2**: User presses Ctrl+G with exactly 1 node selected
  - Input: `selected_items = {"node-1"}`
  - Expected: No dispatch occurs, function returns early
  - Actual behavior: Silent no-op (NOT an error)

- **VIOLATES P2**: User presses Ctrl+G with 0 nodes selected
  - Input: `selected_items = {}`
  - Expected: No dispatch occurs, function returns early
  - Actual behavior: Silent no-op (NOT an error)

- **VIOLATES P3**: `db_tx` context is not provided (None)
  - Input: `db_tx = None`
  - Expected: No dispatch occurs, function returns early
  - Actual behavior: Silent no-op with console log

### Postcondition Violations

- **VIOLATES Q1**: Ctrl+G pressed with valid selection but `tx.send()` fails
  - Input: `selected_items = {"node-1", "node-2", "node-3"}`, channel error
  - Expected: Operation may fail to reach backend but UI behavior is graceful
  - Actual: Should NOT panic; consider showing error toast

- **VIOLATES Q2**: Group operation sent with incorrect node IDs
  - Input: `selected_items = {"node-1", "node-2"}` but `DomainOp::Group { ids: ["node-1"] }` sent
  - Expected: All selected IDs must be in the dispatch
  - Actual: Postcondition violation - not all nodes grouped

---

## Ownership Contracts

- **`doc_signal: &Signal<DiagramDocument>`**: Shared borrow, read-only access to document. No mutation to document state.
- **`db_tx: &Option<Coroutine<EventEnvelope>>`**: Shared borrow of channel. `tx.send()` takes ownership of the `EventEnvelope`.
- **No ownership transfer**: The function does NOT take ownership of any data; it clones IDs into the `EventEnvelope`.

---

## Implementation Phases

### Phase 1: Add Ctrl+G Keyboard Handler
- Add `"G" | "g"` case to keyboard match block in `canvas.rs`
- Guard with `if ctrl_pressed` (use existing `modifier` variable)
- Extract `selected_items` from `doc_signal.read().editor_state.selected_items`

### Phase 2: Validate Preconditions
- Check `selected_items.len() >= 2`
- Check `db_tx.is_some()`

### Phase 3: Construct DomainOp::Group
- Clone selected IDs into `Vec<String>`
- Create `EventEnvelope` with proper author and timestamp

### Phase 4: Dispatch to Backend
- Send envelope via `tx.send(envelope)` pattern matching existing code (line 458-477)

### Phase 5: Error Handling
- Add console log for debugging
- Optionally show toast notification on failure

---

## Non-Goals

- [ ] Implement group operation in backend (already exists)
- [ ] Handle grouping nodes that are already in a group (edge case for future)
- [ ] Add undo support for group operation (deferred)
- [ ] Add toast notifications (optional, can be added later)
