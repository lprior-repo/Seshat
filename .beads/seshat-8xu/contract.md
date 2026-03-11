# Contract Specification: seshat-8xu

## Context

- **Feature**: Toolbar Add Node button - wire button to construct `DomainOp::NodeAdd` and dispatch to `db_tx` coroutine
- **Domain Terms**:
  - `DomainOp::NodeAdd` - Domain operation variant with fields: `id: String`, `x: f64`, `y: f64`, `width: f64`, `height: f64`, `label: String`
  - `EventEnvelope` - Container with `op_id: String`, `operation: DomainOp`, `author: Author`, `timestamp: i64`
  - `db_tx` - Dioxus `Coroutine<EventEnvelope>` for persisting operations to backend
  - `Toolbar` - Dioxus UI component in `diagram_tool/src/ui/toolbar.rs`
  - `DiagramDocument` - The document state signal containing nodes and edges
  - `History` - Undo/redo state signal

- **Assumptions**:
  - The button will add a node at a default/centered position (or center of viewport)
  - Node ID will be generated via `uuid::Uuid::new_v4()`
  - Default node dimensions: 64x64 pixels
  - Default label: "Node"
  - The pattern follows existing toolbar actions (e.g., `actions::undo`, `actions::delete_selection`)

- **Open Questions**:
  - Should the node be added at a fixed position (e.g., viewport center) or at a random position?
  - Should the button always be enabled, or only in certain tool modes (e.g., Select mode)?
  - How does this differ from the existing double-click canvas node creation?

---

## Preconditions

| ID | Precondition | Enforcement Level | Type / Pattern |
|----|-------------|-------------------|----------------|
| P1 | `db_tx` coroutine is available (Some variant) | Runtime check | `Option<Coroutine<EventEnvelope>>` must be `Some` before send |
| P2 | `doc_signal` contains a valid `DiagramDocument` | Compile-time | Signal<DiagramDocument> - type enforced |
| P3 | `history_signal` contains a valid `History` | Compile-time | Signal<History> - type enforced |
| P4 | Node ID is a valid non-empty UUID string | Compile-time | `Uuid::new_v4()` returns valid UUID |
| P5 | Node position (x, y) is within valid coordinate bounds | Runtime check | Check for NaN/Infinity on coordinates |
| P6 | Node dimensions (width, height) are positive values | Runtime check | width > 0 && height > 0 |

---

## Postconditions

| ID | Postcondition | Enforcement Level |
|----|--------------|-------------------|
| Q1 | `DomainOp::NodeAdd` is constructed with valid fields | Type encoding - all fields required |
| Q2 | `EventEnvelope` is created with unique `op_id` | `Uuid::new_v4()` generates fresh ID |
| Q3 | `EventEnvelope.author` is set to local user ("local-user") | Hardcoded in construction |
| Q4 | `EventEnvelope.timestamp` is set to current Unix time | `SystemTime::now().duration_since(UNIX_EPOCH)` |
| Q5 | `EventEnvelope` is sent to `db_tx` coroutine | `.send()` called on Some variant |
| Q6 | Node is inserted into local `doc_signal.document.nodes` | `doc_signal.with_mut()` updates document |
| Q7 | History is updated with previous state via `push()` | `history_signal.write() = history.push(current)` |
| Q8 | Document revision is incremented | `d.revision = d.revision.increment()` |
| Q9 | New node is selected (added to `selected_items`) | Clear and insert node ID |

---

## Invariants

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| I1 | `db_tx` coroutine remains valid after send | Dioxus lifecycle - coroutine owned by component |
| I2 | Document nodes map maintains unique keys | `ImHashMap` guarantees uniqueness |
| I3 | History maintains push/pop consistency | `History::push()` returns new History |
| I4 | Revision number monotonically increases | `Revision::increment()` returns n+1 |

---

## Error Taxonomy

| Error Variant | Condition | Recovery |
|--------------|-----------|----------|
| `Error::DbTxUnavailable` | `db_tx` is `None` when attempting to send | Log warning, continue with local-only update |
| `Error::InvalidPosition` | Node x/y coordinates are NaN or Infinity | Use default position (0.0, 0.0) |
| `Error::InvalidDimensions` | width or height <= 0 | Use default dimensions (64.0, 64.0) |
| `Error::DocumentMutationFailed` | Local document update fails | Show error toast, do not dispatch to db_tx |

---

## Contract Signatures

### Toolbar Action Function

```rust
/// Add a new node to the diagram via toolbar button
///
/// # Preconditions
/// - db_tx must be Some(Coroutine)
/// - doc_signal must contain valid DiagramDocument
/// - history_signal must contain valid History
///
/// # Postconditions
/// - DomainOp::NodeAdd is constructed and dispatched to db_tx
/// - Node is added to local document state
/// - History is updated with previous state
/// - New node is selected
///
/// # Errors
/// - Returns `()` (silent failure) if db_tx unavailable or mutation fails
///   (follows existing toolbar action pattern - errors logged, not propagated)
pub fn add_node(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) {
    // Implementation follows pattern in canvas.rs for NodeMove dispatch
}
```

### EventEnvelope Construction (internal)

```rust
/// Construct an EventEnvelope for NodeAdd operation
///
/// # Preconditions
/// - id must be valid non-empty string
/// - x, y must be finite f64
/// - width, height must be positive
///
/// # Postconditions
/// - Returns EventEnvelope with all fields populated
fn create_node_add_envelope(
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    label: String,
) -> EventEnvelope {
    EventEnvelope {
        op_id: Uuid::new_v4().to_string(),
        operation: DomainOp::NodeAdd { id, x, y, width, height, label },
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

- **VIOLATES P1**: `add_node(doc_signal, history_signal, None)` -- should produce silent log warning, local update may still occur
- **VIOLATES P5**: `create_node_add_envelope(id, f64::NAN, y, w, h, label)` -- should produce `Err(Error::InvalidPosition)`
- **VIOLATES P6**: `create_node_add_envelope(id, x, y, -10.0, h, label)` -- should produce `Err(Error::InvalidDimensions)`

### Postcondition Violations

- **VIOLATES Q5**: When `db_tx` is `None` and we attempt `.send()` -- should not panic, should handle gracefully
- **VIOLATES Q6**: If node insertion fails -- should propagate error, not silently fail

---

## Ownership Contracts (Rust-specific)

| Parameter | Mode | Mutation Contract |
|-----------|------|-------------------|
| `doc_signal: Signal<DiagramDocument>` | Exclusive borrow | Mutates `document.nodes`, `editor_state.selected_items`, `revision` |
| `history_signal: Signal<History>` | Exclusive borrow | Mutates via `write()` - replaces History with new pushed state |
| `db_tx: Option<Coroutine<EventEnvelope>>` | Shared borrow | No mutation - used to send message |
| `id: String` | Ownership transfer | Moved into DomainOp::NodeAdd and EventEnvelope |
| `label: String` | Ownership transfer | Moved into DomainOp::NodeAdd |

---

## Non-goals

- [ ] Adding node at specific user-specified position (future feature)
- [ ] Node customization dialog (width, height, label input)
- [ ] Drag-and-drop node creation from palette
- [ ] Alternative node types (Subgraph, etc.)

---

## Implementation Phases

### Phase 1: EventEnvelope Construction
- Create `create_node_add_envelope()` helper in `toolbar/actions.rs`
- Validate inputs (P5, P6) and return Result

### Phase 2: Toolbar Button
- Add button to Toolbar component in `toolbar.rs`
- Wire onclick to call `add_node()` action

### Phase 3: Local Document Update
- Implement node insertion into `doc_signal.document.nodes`
- Update history and revision

### Phase 4: db_tx Dispatch
- Send EventEnvelope to db_tx coroutine
- Handle None case gracefully

### Phase 5: Testing
- Unit test envelope creation
- Integration test button click -> db_tx message
