# Contract Specification: seshat-84y

## Context

- **Feature**: Wire the SendBackward toolbar button to construct `DomainOp::SendBackward` and dispatch to `db_tx`
- **Domain Terms**:
  - `DomainOp::SendBackward { ids: Vec<String> }` - Domain operation to move selected nodes one step toward back in z-order
  - `EventEnvelope` - Container with `op_id: String`, `operation: DomainOp`, `author: Author`, `timestamp: i64`
  - `db_tx` - Dioxus `Coroutine<EventEnvelope>` for persisting operations to backend
  - `Toolbar` - Dioxus UI component in `diagram_tool/src/ui/toolbar.rs`
  - `DiagramDocument` - The document state signal containing nodes and edges
  - `History` - Undo/redo state signal
  - `apply_send_backward` - Existing function in `commands.rs` that modifies local z-order

- **Assumptions**:
  - The existing `send_backward` action in `toolbar/actions.rs` needs to be modified to accept `db_tx` parameter
  - Selected node IDs come from `doc_signal.read().editor_state.selected_items`
  - The existing local z-order mutation should remain for immediate UI feedback
  - The pattern follows existing `NodeMove` dispatch in `canvas.rs` (lines 458-477)

- **Open Questions**:
  - Should the dispatch to db_tx happen before or after local mutation? (Following canvas.rs pattern: local first, then dispatch)
  - Should we skip dispatch if no nodes would actually move? (Based on existing `apply_send_backward` returning bool)

---

## Preconditions

| ID | Precondition | Enforcement Level | Type / Pattern |
|----|-------------|-------------------|----------------|
| P1 | `db_tx` coroutine is available (Some variant) OR not needed for local-only | Runtime check | `Option<Coroutine<EventEnvelope>>` - graceful handling if None |
| P2 | `doc_signal` contains a valid `DiagramDocument` | Compile-time | `Signal<DiagramDocument>` - type enforced |
| P3 | `history_signal` contains a valid `History` | Compile-time | `Signal<History>` - type enforced |
| P4 | Selected node IDs exist in document nodes | Runtime check | Filter `selected_items` against `document.nodes.keys()` |
| P5 | At least one selected node is movable (not locked, or is Subgraph) | Runtime check | Filter via `!node.locked \|\| node.kind == NodeKind::Subgraph` |

---

## Postconditions

| ID | Postcondition | Enforcement Level |
|----|--------------|-------------------|
| Q1 | `DomainOp::SendBackward` is constructed with valid `ids: Vec<String>` | Type encoding - Vec<String> |
| Q2 | `EventEnvelope` is created with unique `op_id` via `Uuid::new_v4()` | `Uuid::new_v4()` generates fresh ID |
| Q3 | `EventEnvelope.author` is set to local user ("local-user") | Hardcoded in construction |
| Q4 | `EventEnvelope.timestamp` is set to current Unix time | `SystemTime::now().duration_since(UNIX_EPOCH)` |
| Q5 | `EventEnvelope` is sent to `db_tx` coroutine (if available) | `.send()` called on Some variant |
| Q6 | Local z-order is updated via `apply_send_backward()` | Local document mutation |
| Q7 | History is updated with previous state via `push()` | `history_signal.write() = history.push(current)` |
| Q8 | Document revision is incremented | `doc.revision.increment()` |
| Q9 | Function returns `true` if any nodes moved, `false` if no change | Return value from `apply_send_backward` |

---

## Invariants

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| I1 | `db_tx` coroutine remains valid after send (if was Some) | Dioxus lifecycle - coroutine owned by component |
| I2 | Document nodes map maintains unique keys | `ImHashMap` guarantees uniqueness |
| I3 | History maintains push/pop consistency | `History::push()` returns new History |
| I4 | Revision number monotonically increases | `Revision::increment()` returns n+1 |
| I5 | Z-order of unselected nodes remains unchanged | `apply_z_order_to_ids` only swaps selected nodes |

---

## Error Taxonomy

| Error Variant | Condition | Recovery |
|--------------|-----------|----------|
| `Error::DbTxUnavailable` | `db_tx` is `None` when attempting to send | Log debug, continue with local-only update |
| `Error::NoMovableNodes` | All selected nodes are locked and not Subgraphs | Return `false`, no dispatch to db_tx |
| `Error::NoSelectedNodes` | `selected_items` is empty | Return `false`, no dispatch to db_tx |

---

## Contract Signatures

### Toolbar Action Function

```rust
/// Move selected nodes one step backward in z-order via toolbar button
///
/// # Preconditions
/// - doc_signal must contain valid DiagramDocument
/// - history_signal must contain valid History
/// - At least one selected node must exist and be movable
///
/// # Postconditions
/// - DomainOp::SendBackward is constructed and dispatched to db_tx (if available)
/// - Local z-order is updated via apply_send_backward
/// - History is updated with previous state
/// - Document revision is incremented
///
/// # Errors
/// - Returns `false` if no nodes moved (precondition P5 not met)
/// - Returns `false` if no selected nodes (precondition P4 not met)
/// - If db_tx unavailable, logs debug and continues (local mutation still occurs)
pub fn send_backward(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> bool {
    // Implementation follows pattern in canvas.rs for NodeMove dispatch
}
```

### EventEnvelope Construction (internal)

```rust
/// Construct an EventEnvelope for SendBackward operation
///
/// # Preconditions
/// - ids must be non-empty Vec of valid node ID strings
///
/// # Postconditions
/// - Returns EventEnvelope with all fields populated
fn create_send_backward_envelope(
    ids: Vec<String>,
) -> EventEnvelope {
    EventEnvelope {
        op_id: Uuid::new_v4().to_string(),
        operation: DomainOp::SendBackward { ids },
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

- **VIOLATES P1**: `send_backward(doc_signal, history_signal, None)` -- should NOT panic, should continue with local update and log debug
- **VIOLATES P4**: `send_backward` with empty `selected_items` -- should return `false`, no dispatch
- **VIOLATES P5**: `send_backward` with all selected nodes locked (and not Subgraph) -- should return `false`, no dispatch

### Postcondition Violations

- **VIOLATES Q5**: When `db_tx` is `None` and we attempt `.send()` -- should not panic, should handle gracefully
- **VIOLATES Q6**: If local z-order mutation fails -- should propagate error or return `false`

---

## Ownership Contracts (Rust-specific)

| Parameter | Mode | Mutation Contract |
|-----------|------|-------------------|
| `doc_signal: Signal<DiagramDocument>` | Exclusive borrow | Mutates `document.nodes` (z_index field), `editor_state.selected_items`, `revision` |
| `history_signal: Signal<History>` | Exclusive borrow | Mutates via `write()` - replaces History with new pushed state |
| `db_tx: Option<Coroutine<EventEnvelope>>` | Shared borrow | No mutation - used to send message |
| `ids: Vec<String>` | Ownership transfer | Moved into DomainOp::SendBackward and EventEnvelope |

---

## Non-goals

- [ ] Implementing SendBackward for edges (only nodes have z-order)
- [ ] Batch z-order operations (one operation per button click)
- [ ] Animations for z-order changes (future UI enhancement)
- [ ] Alternative z-order algorithms (current algorithm swaps adjacent)

---

## Implementation Phases

### Phase 1: Modify Toolbar Action Function
- Update `actions::send_backward` in `toolbar/actions.rs` to accept `db_tx` parameter
- Extract selected node IDs from `doc_signal.read().editor_state.selected_items`
- Filter to only movable nodes (P5)

### Phase 2: EventEnvelope Construction
- Create `create_send_backward_envelope()` helper (can be inline in function)
- Validate inputs (P4, P5) and return early if invalid

### Phase 3: db_tx Dispatch
- Send EventEnvelope to db_tx coroutine if available
- Handle None case gracefully with debug log

### Phase 4: Update Toolbar Button
- Update `toolbar.rs` to pass `db_tx` to `actions::send_backward`
- Obtain db_tx via `use_context::<Option<Coroutine<EventEnvelope>>>()`

### Phase 5: Testing
- Unit test envelope construction
- Integration test button click -> db_tx message
- Verify local mutation still occurs

---

## EARS Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| EARS-1 | WHEN the user clicks the SendBackward toolbar button AND there are selected movable nodes, THEN the system SHALL construct DomainOp::SendBackward { ids } with the selected node IDs AND dispatch to db_tx | Must |
| EARS-2 | WHEN the user clicks SendBackward AND db_tx is unavailable, THEN the system SHALL still update local z-order for immediate feedback | Must |
| EARS-3 | WHEN the user clicks SendBackward AND no nodes are selected OR all selected are locked, THEN the system SHALL return false and NOT dispatch to db_tx | Must |
