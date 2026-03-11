# Contract Specification: seshat-zrx

## Context
- **Feature**: UI Dispatch - Properties Panel Node Shape Selection
- **Description**: Wire the PropertiesPanel node shape change to dispatch UpdateNodeStyle to db_tx
- **Domain terms**:
  - `PropertiesPanel` - Dioxus component for editing node/edge properties
  - `NodeStyle` - Enum with variants: Box, Cloud, Cylinder, Dashed
  - `EventEnvelope` - Container for diagram operations with metadata
  - `DomainOp` - Operation types (NodeAdd, NodeMove, etc.)
  - `db_tx` - Coroutine<EventEnvelope> for persisting operations
- **Assumptions**:
  - NodeStyle enum exists with 4 variants (Box, Cloud, Cylinder, Dashed)
  - db_tx is available as a context in PropertiesPanel
  - DomainOp needs a new variant for NodeStyleUpdate
- **Open questions**:
  - Should the dispatch be immediate on change or on blur/enter?
  - Should history be pushed before style change (for undo)?

## Preconditions

| ID | Precondition | Enforcement Level | Type/Pattern |
|----|--------------|-------------------|--------------|
| P1 | Node must be selected (single node selection) | Compile-time | `single_node: Option<(NodeId, Node)>` extracted from selected_items |
| P2 | NodeStyle value must be a valid variant | Compile-time | `NodeStyle` enum - only valid variants accepted |
| P3 | db_tx context must be available | Runtime | `use_context::<Option<Coroutine<EventEnvelope>>>()` |
| P4 | Node must exist in document | Runtime | Document lookup by NodeId |

## Postconditions

| ID | Postcondition | Enforcement Level |
|----|--------------|-------------------|
| Q1 | EventEnvelope is sent to db_tx | Runtime - tx.send() call |
| Q2 | EventEnvelope.operation is DomainOp::NodeStyleUpdate { id, style } | Type system |
| Q3 | Node.style in document is updated to new style | Runtime - with_mut updates |
| Q4 | Document revision is incremented | Runtime - doc.revision.increment() |

## Invariants

| ID | Invariant |
|----|-----------|
| I1 | Exactly one node selected when shape UI is visible |
| I2 | NodeStyle always matches one of: Box, Cloud, Cylinder, Dashed |
| I3 | Document revision increments monotonically |
| I4 | History is pushed before mutation (for undo support) |

## Error Taxonomy

| Error | Condition |
|-------|-----------|
| Error::NoNodeSelected | When selected_node_count != 1 |
| Error::NodeNotFound | When node ID not in document.nodes |
| Error::DbTxUnavailable | When db_tx context is None |
| Error::InvalidStyleValue | When style string doesn't parse to NodeStyle |

## Contract Signatures

```rust
// In DomainOp enum:
NodeStyleUpdate {
    id: String,
    style: NodeStyle,
}

// In PropertiesPanel:
fn on_node_style_change(node_id: NodeId, style: NodeStyle) {
    // 1. Push history for undo
    // 2. Update doc_signal with new style
    // 3. Dispatch EventEnvelope to db_tx
}

// Function signature for event dispatch:
fn dispatch_style_update(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_id: String,
    style: NodeStyle,
) -> Result<(), Error>
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|--------------|-------------------|----------------|
| P1: Single node selected | Runtime check | `if selected_node_count == 1` guard |
| P2: Valid NodeStyle | Compile-time | `NodeStyle` enum - invalid values are compile errors |
| P3: db_tx available | Runtime | Option handling with `if let Some(tx)` |
| P4: Node exists | Runtime | `doc.document.nodes.get(&node_id)` lookup |

## Violation Examples

### Precondition Violations

- **VIOLATES P1**: User has 0 nodes selected or multiple nodes selected
  - Input: `selected_node_count = 0` OR `selected_node_count > 1`
  - Expected: Shape selector UI hidden, no event dispatched
  - Behavior: Guard clause prevents dispatch

- **VIOLATES P3**: db_tx context is None (not initialized)
  - Input: `db_tx = None`
  - Expected: `Err(Error::DbTxUnavailable)` - but in practice UI silently fails
  - Behavior: `if let Some(tx)` guard prevents panic

### Postcondition Violations

- **VIOLATES Q1**: EventEnvelope not sent to db_tx
  - Input: Valid style change with missing tx
  - Expected: `Err(Error::DbTxUnavailable)`
  - Actual: Local state updates but no persistence

- **VIOLATES Q2**: Wrong operation type in envelope
  - Input: Any valid dispatch
  - Expected: `DomainOp::NodeStyleUpdate { id, style }`
  - Actual: Wrong variant would be type error at compile time

- **VIOLATES Q3**: Node style not updated in document
  - Input: Style change dispatch
  - Expected: `doc.document.nodes.get_mut(&id).style = Some(new_style)`
  - Actual: Would require code bug

## Ownership Contracts

### PropertiesPanel Component
- **doc_signal**: `Signal<DiagramDocument>` - exclusive borrow via `with_mut`
- **history**: `Signal<History>` - exclusive borrow for push operation
- **db_tx**: `Option<Coroutine<EventEnvelope>>` - shared borrow, clone for closure

### Mutation Postconditions (for &mut parameters)
- `doc_signal.with_mut(|doc| ...)` mutates:
  - `doc.document.nodes.get_mut(&node_id).style` - the node's style field
  - `doc.revision` - incremented by 1
- `history.write()` mutates:
  - Adds current document state to history stack

## Non-goals
- [ ] Adding node shape to multi-select (single node only)
- [ ] Persisting style defaults (handled elsewhere)
- [ ] Changing edge styles via this dispatch (different operation)

## Implementation Phases

### Phase 1: Domain Model
1. Add `NodeStyleUpdate { id: String, style: NodeStyle }` to DomainOp enum
2. Add `NodeStyleUpdate` case to `DomainOp::kind()` match
3. Add parser for `node_style_update` in `parse_domain_op`

### Phase 2: Properties Panel UI
1. Add `NodeStyle` import to properties.rs
2. Add helper function: `fn node_style_str(style: &NodeStyle) -> &'static str`
3. Add helper function: `fn parse_node_style(s: &str) -> NodeStyle`
4. Add shape selector UI in single_node block (after Lock button)

### Phase 3: Event Dispatch
1. Get db_tx context in PropertiesPanel
2. Wire onchange handler to:
   - Push history (if style actually changed)
   - Update doc_signal with new style
   - Dispatch EventEnvelope to db_tx

### Phase 4: Testing
1. Verify shape selector appears for single node
2. Verify style changes persist to document
3. Verify events dispatch to db_tx
4. Verify history push for undo
