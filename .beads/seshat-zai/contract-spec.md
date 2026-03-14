# Contract Specification: seshat-zai (SUB-008 to SUB-012: Subgraph un-grouping)

**Bead ID**: seshat-zai
**Title**: SUB-008 to SUB-012: Subgraph un-grouping
**Description**: Implement flattening a subgraph back to the root level.
**Phase**: rust-contract
**Created**: 2026-03-14

## Context

- **Feature**: Subgraph un-grouping (flattening)
- **Domain terms**:
  - `DiagramDocument` - The document containing nodes and edges
  - `NodeId` - Unique identifier for nodes
  - `NodeKind::Subgraph` - Container node type that holds children
  - `parent` - Optional reference to containing subgraph
  - `selected_items` - Currently selected node IDs
- **Assumptions**:
  - Subgraph ungroup is the inverse operation of group (seshat-t6u)
  - Children are reparented to the removed subgraph's parent (or root if none)
  - Edges connected to removed subgraphs are cleaned up
- **Open questions**:
  - Should nested subgraphs be recursively flattened? (Current: only selected subgraph, children get reparented)
  - What happens to deeply nested children when parent is removed? (Current: they get reparented to grandparent)

## Preconditions

- [P1] Selection must not be empty - at least one subgraph must be selected
- [P2] All selected items must be subgraphs (NodeKind::Subgraph)
- [P3] Document must have valid node references for all selected IDs

## Postconditions

- [Q1] All selected subgraph nodes are removed from the document
- [Q2] All children of removed subgraphs have their parent set to the removed subgraph's parent (or None if root)
- [Q3] All edges connected to removed subgraphs are removed
- [Q4] The editor state's selected_items contains all orphaned children (previously direct children of removed subgraphs)
- [Q5] Node count decreases by exactly the number of removed subgraphs
- [Q6] All remaining nodes maintain valid parent references (parent chain is valid)

## Invariants

- [INV1] Parent chain validity - every node's parent must be a valid subgraph (or None for root)
- [INV2] No orphan edges - every edge must connect to existing nodes
- [INV3] Node count consistency - after ungroup, node count = original - num_removed_subgraphs

## Error Taxonomy

- `GroupingError::EmptySelection` - When no subgraphs are selected
- `GroupingError::NodeNotFound` - When selected ID doesn't exist in document

## Contract Signatures

```rust
/// Pure calculation: Remove subgraphs and reparent their children
pub fn calculate_ungroup(
    nodes: &HashMap<NodeId, Node>,
    target_subgraphs: &BTreeSet<NodeId>,
) -> (HashMap<NodeId, Node>, BTreeSet<NodeId>)

/// Pure calculation: Remove edges connected to deleted subgraphs
pub fn calculate_edge_cleanup(
    edges: &HashMap<EdgeId, Edge>,
    deleted_subgraphs: &BTreeSet<NodeId>,
) -> HashMap<EdgeId, Edge>

/// Action: Ungroup selected subgraphs in a DiagramDocument
pub fn ungroup_selection(doc: &mut DiagramDocument) -> Result<(), GroupingError>
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Selection not empty | Runtime-checked constructor | `if selected.is_empty() { Err(GroupingError::EmptySelection) }` |
| All selected are subgraphs | Runtime-checked | `node.kind == NodeKind::Subgraph` filter |
| Node references valid | Runtime-checked | `nodes.contains_key(id)` check |

## Violation Examples (REQUIRED)

- VIOLATES P1: `ungroup_selection(&mut empty_doc)` -- should produce `Err(GroupingError::EmptySelection)`
- VIOLATES P2: `ungroup_selection(&mut doc_with_text_selected)` -- should produce `Err(GroupingError::EmptySelection)` (no subgraphs in selection)
- VIOLATES Q1: After ungroup, document still contains the removed subgraph ID
- VIOLATES Q2: Children of removed subgraph still have parent reference to removed subgraph
- VIOLATES Q3: Edge connected to removed subgraph still exists in document
- VIOLATES Q4: Selected items does not contain orphaned children after ungroup
- VIOLATES Q5: Node count changed by amount other than num_removed_subgraphs
- VIOLATES INV1: Any node has parent reference to non-existent node

## Ownership Contracts (Rust-specific)

- `ungroup_selection(doc: &mut DiagramDocument)`:
  - Mutates: `doc.document.nodes` (removes subgraphs, updates parent refs)
  - Mutates: `doc.document.edges` (removes connected edges)
  - Mutates: `doc.editor_state.selected_items` (clears and adds orphans)
  - Mutates: `doc.revision` (incremented on success)

- `calculate_ungroup(nodes: &HashMap, target_subgraphs: &BTreeSet)`:
  - Borrows: `nodes` (read-only)
  - Borrows: `target_subgraphs` (read-only)
  - Returns: New HashMap (ownership transfer)

- `calculate_edge_cleanup(edges: &HashMap, deleted_subgraphs: &BTreeSet)`:
  - Borrows: `edges` (read-only)
  - Borrows: `deleted_subgraphs` (read-only)
  - Returns: New HashMap (ownership transfer)

## Non-goals

- [ ] Visual animation of ungroup operation (UI concern)
- [ ] Undo/redo functionality (handled by history system)
- [ ] Copy/paste of subgraphs (seshat-vis)
- [ ] Nested subgraph recursive flatten (future enhancement)

## Related Specifications

- seshat-t6u (SUB-003 to SUB-007): Subgraph creation (inverse operation)
- bd-1b9: SUB-008 to SUB-010 tests (drag operations, already covered)
