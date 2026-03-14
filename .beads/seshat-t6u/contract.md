# Contract Specification: Subgraph Creation (SUB-003 to SUB-007)

## Context
- **Bead ID**: seshat-t6u
- **Bead Title**: SUB-003 to SUB-007: Subgraph creation
- **Feature**: Implement grouping selected nodes into a new Subgraph container.
- **Domain terms**:
    - **Subgraph**: A container node (`NodeKind::Subgraph`) that can have children.
    - **Reparenting**: The process of changing a node's `parent` field to a new `Subgraph` ID.
    - **Bounding Box**: The minimum rectangle encompassing all selected nodes.
    - **Padding**: Extra space added around the bounding box (default 24.0).
    - **LCA (Lowest Common Ancestor)**: The most recent common container of all selected nodes.

## Preconditions
- [ ] **P1: Selection Non-Empty**: At least one node must be selected for grouping. (Runtime: `GroupingError::EmptySelection`)
- [ ] **P2: Nodes Exist**: All IDs in the selection must exist in the document. (Runtime: `GroupingError::NodeNotFound(NodeId)`)
- [ ] **P3: Nodes Not Locked**: None of the selected nodes can be locked. (Runtime: `GroupingError::LockedNode(Vec<NodeId>)` - returns all locked IDs found)
- [ ] **P4: Finite Coordinates**: All selected nodes must have finite coordinates (no NaN/Inf). (Runtime: `GroupingError::InvalidCoordinates`)
- [ ] **P5: Nesting Limit**: Grouping must not exceed the maximum nesting depth (5). (Runtime: `GroupingError::NestedSubgraphLimitExceeded(usize)`)

## Postconditions
- [ ] **Q1: Subgraph Created**: A new node of `NodeKind::Subgraph` is created.
- [ ] **Q2: Children Reparented**: All previously selected nodes now have their `parent` set to the new Subgraph's ID.
- [ ] **Q3: Correct Bounds**: The new Subgraph's bounds encompass all children plus a mandatory padding (default 24.0).
- [ ] **Q4: Selection Updated**: The selection is cleared and replaced with ONLY the new Subgraph node.
- [ ] **Q5: Z-Index Consistency**: The Subgraph's `z_index` is set to `min(children.z_index) - 1`, ensuring it renders behind its children.
- [ ] **Q6: Parent Assignment**: The new Subgraph's `parent` is set to the **Lowest Common Ancestor (LCA)** of all selected nodes (or `None` if they are at the root or have no common ancestor other than the root).
- [ ] **Q7: WAL Consistency**: The operation is persisted as a single `EventEnvelope` with `DomainOp::Group` in the SQLite WAL log.
- [ ] **Q8: Atomicity**: The entire operation (creation + reparenting + selection update) is applied as a single transaction (undoable as one unit).

## Invariants
- [ ] **I1: No Orphaned Children**: Every child of the new subgraph must still exist in the document.
- [ ] **I2: No Circular Parents**: The new subgraph cannot be its own ancestor.

## Error Taxonomy
- `GroupingError::EmptySelection` - when `selected.is_empty()`.
- `GroupingError::NodeNotFound(NodeId)` - when a selected ID is missing.
- `GroupingError::LockedNode(Vec<NodeId>)` - when any selected nodes are locked.
- `GroupingError::SubgraphTooSmall { width, height }` - when calculated bounds are invalid or too small.
- `GroupingError::NestedSubgraphLimitExceeded(usize)` - when depth > 5.
- `GroupingError::InvalidCoordinates` - when coordinates are NaN or Inf.

## Contract Signatures
- `fn group_selection(doc: &mut DiagramDocument, group_id: &NodeId) -> Result<(), GroupingError>`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Non-Empty | Result error variant | `GroupingError::EmptySelection` |
| P2: Exist | Result error variant | `GroupingError::NodeNotFound` |
| P3: Not Locked | Result error variant | `GroupingError::LockedNode` |
| P4: Finite | `OrderedFloat` wrapper | `crate::models::document::OrderedFloat` |
| P5: Nesting | Result error variant | `GroupingError::NestedSubgraphLimitExceeded` |

## Violation Examples (REQUIRED)
- VIOLATES <P1>: `group_selection(doc, &id)` where `doc.selected.is_empty()` -- should produce `Err(GroupingError::EmptySelection)`
- VIOLATES <P2>: `group_selection(doc, &id)` where `doc.selected = {"missing_node"}` -- should produce `Err(GroupingError::NodeNotFound("missing_node"))`
- VIOLATES <P3>: `group_selection(doc, &id)` where nodes `A` and `B` are locked -- should produce `Err(GroupingError::LockedNode(vec!["A", "B"]))`
- VIOLATES <P4>: `group_selection(doc, &id)` where one node has `x: NaN` -- should produce `Err(GroupingError::InvalidCoordinates)`
- VIOLATES <P5>: `group_selection(doc, &id)` where selection includes a node at depth 5 -- should produce `Err(GroupingError::NestedSubgraphLimitExceeded(5))`

## Ownership Contracts (Rust-specific)
- `doc: &mut DiagramDocument`: Exclusive borrow. Mutates `doc.document.nodes` (adds 1, updates N) and `doc.editor_state.selected_items`.

## Non-goals
- [ ] Does not handle edge rerouting (handled by separate cascade logic).
- [ ] Does not handle auto-layout of children within the new group.
