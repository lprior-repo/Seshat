# Contract Specification

## Context
- **Feature**: Wire PropertiesPanel node style change to dispatch UpdateNodeStyle to db_tx
- **Bead**: seshat-x0s
- **Domain terms**:
  - `PropertiesPanel` - Dioxus component for editing selected node/edge properties
  - `NodeStyle` - enum (Box, Cloud, Cylinder, Dashed) representing node shape
  - `db_tx` - Coroutine<EventEnvelope> channel for persisting operations
  - `EventEnvelope` - envelope containing DomainOp, author, timestamp
  - `DomainOp` - enum of all diagram operations (NodeAdd, NodeMove, etc.)
- **Assumptions**:
  - A new DomainOp variant `NodeStyleUpdate` will be added to envelope.rs
  - PropertiesPanel already has access to doc_signal and history signal
  - db_tx is available via use_context::<Option<Coroutine<EventEnvelope>>>()
- **Open questions**:
  - Should NodeStyleUpdate include color, or just the shape enum?
  - What is the error handling if db_tx is None (not available)?

## Preconditions
- [P1] **Single node selected**: PropertiesPanel must have exactly one node selected (selected_node_count == 1)
- [P2] **Valid NodeStyle**: The style value must be a valid NodeStyle variant (Box, Cloud, Cylinder, Dashed)
- [P3] **Node exists**: The node with the selected ID must exist in the document
- [P4] **db_tx available**: The db_tx coroutine must be Some (not None) for persistence

## Postconditions
- [Q1] **Document updated**: After change, node.style equals the new NodeStyle value
- [Q2] **Revision incremented**: doc.revision is incremented after mutation
- [Q3] **History pushed**: History signal is updated with previous state before mutation
- [Q4] **Event dispatched**: An EventEnvelope with DomainOp::NodeStyleUpdate is sent to db_tx
- [Q5] **Idempotent check**: Only push history and dispatch event if style actually changed

## Invariants
- [I1] **Document consistency**: document.nodes must contain valid Node entries
- [I2] **Revision monotonicity**: revision must only increment, never decrement
- [I3] **History non-empty**: After any mutation, history must contain the previous state

## Error Taxonomy
- **Error::NoDbTx** - db_tx is None, cannot persist the operation
- **Error::NodeNotFound** - Selected node ID does not exist in document
- **Error::InvalidStyle** - Style value is not a valid NodeStyle variant
- **Error::PreconditionViolation** - P1, P2, or P3 not met

## Contract Signatures

### New DomainOp Variant (to add in envelope.rs)
```rust
NodeStyleUpdate {
    id: String,
    style: NodeStyle,  // Box, Cloud, Cylinder, Dashed
}
```

### PropertiesPanel Handler (pseudo-signature)
```rust
// In PropertiesPanel, when node style select changes:
fn on_node_style_change(node_id: NodeId, new_style: NodeStyle) -> Result<(), Error>
where
    Error: From<NoDbTx> + From<NodeNotFound>,
{
    // P1: Check single node selected
    // P2: Validate new_style is valid NodeStyle
    // P3: Check node exists
    // Q5: Check if style actually changed
    // Q3: Push history
    // Q1: Update doc_signal.node.style
    // Q2: Increment revision
    // Q4: Send EventEnvelope to db_tx (if Some)
}
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Single node selected | Runtime (UI logic) | Guard: `if selected_node_count != 1 { return }` |
| P2: Valid NodeStyle | Compile-time | `enum NodeStyle { Box, Cloud, Cylinder, Dashed }` |
| P3: Node exists | Runtime | `doc.document.nodes.get(&node_id).is_some()` |
| P4: db_tx available | Runtime | `if let Some(tx) = &db_tx { ... }` |
| Q5: Idempotent check | Runtime | `if current_style != new_style { ... }` |

## Violation Examples

### Precondition Violations
- **VIOLATES P1**: User has 0 or 2+ nodes selected, tries to change style
  - Input: selected_node_count = 0, user interacts with style selector
  - Expected: No-op (style selector hidden or disabled)

- **VIOLATES P2**: Invalid style value passed
  - Input: style = "invalid_style"
  - Expected: `Err(Error::InvalidStyle)` - caught by parse function

- **VIOLATES P3**: Node was deleted between selection and mutation
  - Input: node_id = "n999" (doesn't exist), style = NodeStyle::Box
  - Expected: `Err(Error::NodeNotFound)`

- **VIOLATES P4**: db_tx is None (e.g., async-db feature disabled)
  - Input: db_tx = None, user changes style
  - Expected: Update local doc only, log warning (or Err based on config)

### Postcondition Violations
- **VIOLATES Q1**: After change, node.style is still old value
  - State: doc_signal.read().document.nodes[target_id].style == old_style
  - Expected: Should equal new_style

- **VIOLATES Q2**: After change, revision unchanged
  - State: doc.revision == old_revision
  - Expected: doc.revision == old_revision + 1

- **VIOLATES Q3**: History not pushed before mutation
  - State: history does not contain pre-mutation document state
  - Expected: history.push() called before doc mutation

- **VIOLATES Q4**: Event not dispatched to db_tx
  - State: db_tx never receives EventEnvelope
  - Expected: db_tx.send(...) called with NodeStyleUpdate envelope

## Ownership Contracts

- **doc_signal: &mut DiagramDocument** (via doc_signal.with_mut)
  - Mutation: node.style field updated
  - Mutation: doc.revision incremented
  - Mutation: history.push() adds previous state

- **db_tx: Option<Coroutine<EventEnvelope>>**
  - Shared borrow: Only used to send(), no mutation
  - Clone policy: tx.send() clones the EventEnvelope (required for channel)

- **history: &mut History**
  - Mutation: new entry pushed to history stack
  - Ownership: Passed as Signal, caller retains ownership

## Non-goals
- [ ] Edge style changes (separate concern)
- [ ] Multi-node style batch updates (future enhancement)
- [ ] Undo/redo implementation (history already exists)
- [ ] Color picker for custom colors (only enum values)

---

## Implementation Phases

### Phase 1: Domain Model Update
1. Add `NodeStyleUpdate` variant to `DomainOp` enum in `envelope.rs`
2. Ensure `NodeStyle` implements Serialize/Deserialize

### Phase 2: PropertiesPanel UI
1. Add `<select>` element for NodeStyle in PropertiesPanel
2. Show only when single node is selected
3. Display current style as selected option

### Phase 3: Event Dispatch Wiring
1. Get db_tx via use_context
2. In onchange handler:
   - Check P1 (single node)
   - Check P5 (style changed)
   - Push history (Q3)
   - Update doc_signal (Q1, Q2)
   - Send to db_tx (Q4)

### Phase 4: Error Handling
1. Handle db_tx = None gracefully (log warning)
2. Add error boundary for malformed input
