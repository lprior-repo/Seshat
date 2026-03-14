# Contract Specification: Subgraph Deletion Cascade

## Context
- **Bead ID**: seshat-2h3
- **Title**: SUB-032 to SUB-034: Subgraph deletion cascade
- **Feature**: Deleting subgraph moves children to parent or deletes them depending on setting

### Domain Terms
| Term | Definition |
|------|------------|
| **Subgraph** | A `Node` with `NodeKind::Subgraph` that acts as a container for child nodes |
| **Child node** | A node with `parent: Some(NodeId)` pointing to its containing subgraph |
| **Root-level node** | A node with `parent: None` (not inside any subgraph) |
| **Nested subgraph** | A subgraph that is a child of another subgraph |
| **Edge** | A connection between two nodes (`Edge.source`, `Edge.target`) |
| **Cascade mode** | Setting that controls whether children are reparented or deleted when parent subgraph is deleted |
| **Reparent** | Move a node's `parent` reference from deleted subgraph to its grandparent or None |

### Assumptions
1. Existing `ungroup_selection` function in `diagram_tool/src/core/grouping.rs` handles the reparenting logic
2. `DiagramDocument` is the mutable state container with `document.nodes: HashMap<NodeId, Node>` and `document.edges: HashMap<EdgeId, Edge>`
3. Edges are removed when either endpoint is inside a deleted subgraph
4. The cascade setting is passed as a parameter to the deletion function

### Open Questions
- **Q1**: Should the cascade setting be a field on the subgraph itself, or a parameter to the delete operation?
  - *Decision*: Parameter to the delete/ungroup operation for flexibility
- **Q2**: What happens to nested subgraphs when parent is deleted with delete-children mode?
  - *Decision*: Nested subgraphs are treated as children - they are also deleted
- **Q3**: Should edges be preserved or deleted when children are reparented?
  - *Decision*: Edges are preserved when reparenting; edges connected to deleted nodes are removed

---

## Preconditions

### P1: Subgraph Selection Required
- **Contract**: At least one subgraph must be selected in `doc.editor_state.selected_items`
- **Enforcement Level**: Runtime (Result<T, Error>)
- **Error Variant**: `GroupingError::EmptySelection` when no subgraphs selected

### P2: No Locked Subgraphs
- **Contract**: Selected subgraphs must not be locked (`node.locked == false`)
- **Enforcement Level**: Runtime (Result<T, Error>)
- **Error Variant**: `GroupingError::LockedNode(NodeId)` with the locked node ID

### P3: Valid Cascade Mode
- **Contract**: Cascade mode must be one of: `CascadeMode::Reparent` or `CascadeMode::Delete`
- **Enforcement Level**: Compile-time (enum)
- **Type Pattern**: `enum CascadeMode { Reparent, Delete }`

### P4: Valid Document State
- **Contract**: Document must be valid before operation (no cycles, all parent references point to existing nodes)
- **Enforcement Level**: Debug-only assertion
- **Note**: Full validation is expensive; basic checks are performed

---

## Postconditions

### Q1: Subgraph Removed from Document
- **Contract**: After successful deletion, `doc.document.nodes` does NOT contain any of the deleted subgraph IDs
- **Verification**: `doc.document.nodes.get(subgraph_id)` returns `None`

### Q2: Children Reparented (Reparent Mode)
- **Contract**: When `CascadeMode::Reparent` is used, all children have their `parent` set to the deleted subgraph's parent (or None if root-level)
- **Verification**: For each child `c`, `c.parent == deleted_subgraph.parent`

### Q3: Children Deleted (Delete Mode)
- **Contract**: When `CascadeMode::Delete` is used, no child node IDs remain in `doc.document.nodes`
- **Verification**: `doc.document.nodes.keys()` does not contain any child IDs

### Q4: Orphaned Edges Removed
- **Contract**: Any edge where `source` or `target` was inside a deleted subgraph is removed from `doc.document.edges`
- **Verification**: For all edges `e`, `e.source` and `e.target` are in `doc.document.nodes`

### Q5: Selection Updated
- **Contract**: After operation, `doc.editor_state.selected_items` contains only the orphaned children (reparent mode) or is empty (delete mode)
- **Verification**: Check `selected_items` collection

### Q6: Nested Subgraphs Handled
- **Contract**: When a parent subgraph is deleted, all nested child subgraphs are also processed according to the cascade mode
- **Verification**: Recursive processing of all descendant subgraphs

---

## Invariants

### INV1: Parent Chain Validity
- **Invariant**: For every node `n` with `n.parent == Some(pid)`, there exists a node with ID `pid` that is a subgraph
- **Must Hold**: After any reparenting operation

### INV2: No Orphan Edges
- **Invariant**: For every edge `e` in `doc.document.edges`, both `e.source` and `e.target` exist in `doc.document.nodes`
- **Must Hold**: After any deletion operation

### INV3: Node Count Consistency
- **Invariant**: `doc.document.nodes.len() == original_nodes - deleted_subgraphs - (deleted_children if Delete mode)`
- **Must Hold**: After any subgraph deletion operation

---

## Error Taxonomy

### GroupingError (Existing)
| Variant | Meaning | Recovery |
|---------|---------|----------|
| `EmptySelection` | No nodes selected, or no subgraphs in selection | Select at least one subgraph |
| `LockedNode(NodeId)` | Attempted to modify a locked node | Unlock the node first |
| `SubgraphTooSmall { width, height }` | Subgraph bounds below minimum | Resize before ungrouping |
| `NestedSubgraphLimitExceeded(usize)` | Nesting depth would exceed limit (5) | Reduce nesting depth |

### New Error Variants for This Feature
| Variant | Meaning | Recovery |
|---------|---------|----------|
| (Use existing `GroupingError` variants) | Feature extends existing ungroup behavior | N/A |

---

## Contract Signatures

### Primary Function: Extended Ungroup with Cascade
```rust
/// Delete subgraphs with configurable child handling.
///
/// # Arguments
/// * `doc` - Mutable diagram document
/// * `cascade_mode` - Whether to reparent or delete children
///
/// # Errors
/// Returns `GroupingError::EmptySelection` if no subgraphs selected.
/// Returns `GroupingError::LockedNode` if any subgraph is locked.
pub fn ungroup_selection_with_cascade(
    doc: &mut DiagramDocument,
    mode: CascadeMode,
) -> Result<(), GroupingError>
```

### Supporting Function: Nested Subgraph Deletion
```rust
/// Process nested subgraphs recursively according to cascade mode.
///
/// # Arguments
/// * `doc` - Mutable diagram document  
/// * `subgraph_id` - The subgraph to process
/// * `mode` - Cascade mode for children
/// * `parent_of_deleted` - The parent to reparent children to (if Reparent mode)
fn process_subgraph_deletion(
    doc: &mut DiagramDocument,
    subgraph_id: &NodeId,
    mode: CascadeMode,
    parent_of_deleted: Option<&NodeId>,
) -> Vec<NodeId>  // Returns orphaned child IDs
```

### Helper: Edge Cleanup
```rust
/// Remove edges connected to nodes in the given set.
///
/// # Arguments
/// * `doc` - Mutable diagram document
/// * `affected_nodes` - Set of node IDs whose edges should be removed
fn remove_edges_connected_to(
    doc: &mut DiagramDocument,
    affected_nodes: &HashSet<NodeId>,
)
```

---

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| At least one subgraph selected | Runtime-checked | `Result<(), GroupingError::EmptySelection>` |
| Subgraphs not locked | Runtime-checked | `Result<(), GroupingError::LockedNode(NodeId)>` |
| Valid cascade mode | Compile-time (strongest) | `enum CascadeMode { Reparent, Delete }` |
| No cycles in parent chain | Debug-only | `debug_assert!(validate_no_cycles(doc))` |

---

## Violation Examples

### VIOLATES P1 (No Subgraph Selected)
```rust
// Given: Empty selection
doc.editor_state.selected_items.clear();
let result = ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent);

// Then: Returns Err(GroupingError::EmptySelection)
assert!(matches!(result, Err(GroupingError::EmptySelection)));
```

### VIOLATES P2 (Locked Subgraph)
```rust
// Given: A locked subgraph selected
let subgraph_id = NodeId::new("locked-subgraph");
doc.document.nodes.insert(subgraph_id.clone(), locked_subgraph());
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());

let result = ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent);

// Then: Returns Err(GroupingError::LockedNode(subgraph_id))
assert!(matches!(result, Err(GroupingError::LockedNode(id)) if id == subgraph_id));
```

### VIOLATES Q2 (Children Not Reparented)
```rust
// Given: Subgraph with children, Reparent mode
let parent_subgraph = NodeId::new("parent");
let child_node = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &parent_subgraph, &child_node);

ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent).unwrap();

// Then (VIOLATION): Child's parent should be None (parent's parent was None)
let actual_parent = doc.document.nodes.get(&child_node).unwrap().parent.clone();
assert_ne!(actual_parent, Some(parent_subgraph), "Child should be reparented to None");
```

### VIOLATES Q3 (Children Not Deleted)
```rust
// Given: Subgraph with children, Delete mode
let subgraph_id = NodeId::new("subgraph");
let child_id = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &subgraph_id, &child_id);

ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete).unwrap();

// Then (VIOLATION): Child should NOT exist in document
assert!(doc.document.nodes.contains_key(&child_id), "Child should be deleted");
```

### VIOLATES Q4 (Edges Not Cleaned Up)
```rust
// Given: Subgraph with a child that has an external edge
let subgraph_id = NodeId::new("subgraph");
let internal_node = NodeId::new("internal");
let external_node = NodeId::new("external");
let edge_id = EdgeId::new("edge1");
setup_subgraph_with_edge(&mut doc, &subgraph_id, &internal_node, &external_node, &edge_id);

ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent).unwrap();

// Then (VIOLATION): Edge should be removed
assert!(doc.document.edges.contains_key(&edge_id), "Edge to internal node should be removed");
```

### VIOLATES Q5 (Selection Not Updated)
```rust
// Given: Subgraph with children, Reparent mode
let subgraph_id = NodeId::new("subgraph");
let child_id = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &subgraph_id, &child_id);

ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent).unwrap();

// Then (VIOLATION): Selection should contain orphaned child
assert!(!doc.editor_state.selected_items.contains(&child_id.to_string()), 
    "Selection should contain orphaned child in reparent mode");
```

---

## Ownership Contracts

### ungroup_selection_with_cascade
- **Input**: `&mut DiagramDocument` (exclusive borrow)
- **Mutations**:
  - `doc.document.nodes`: Subgraphs and possibly children removed or reparented
  - `doc.document.edges`: Edges to deleted nodes removed
  - `doc.editor_state.selected_items`: Updated to contain orphaned children
- **Output**: `Result<(), GroupingError>` - caller receives ownership of success/failure

### process_subgraph_deletion
- **Input**: `&mut DiagramDocument`, `&NodeId` (reference)
- **Mutations**: Same as above, but scoped to a single subgraph and its descendants
- **Output**: `Vec<NodeId>` - IDs of children that were reparented (for selection update)

### remove_edges_connected_to
- **Input**: `&mut DiagramDocument`, `&HashSet<NodeId>`
- **Mutations**: `doc.document.edges` - edges filtered to remove those connected to affected nodes
- **Output**: Unit (modifies in place)

---

## Non-goals
- [ ] Visual rendering of subgraph collapse/expand (handled by UI layer)
- [ ] Persistence/serialization of cascade settings (handled by IO bead)
- [ ] Undo/Redo for cascade operations (handled by History bead)
- [ ] Multi-document operations (single document scope)
- [ ] Concurrent document modifications (single-threaded assumption)

---

## Test Case Mapping (SUB-032 to SUB-034)

| Test ID | Description | Contract Coverage |
|---------|-------------|-------------------|
| SUB-032 | Edge between nested subgraphs | Q4, INV2 - Edges crossing nested subgraph boundaries must be handled |
| SUB-033 | Edge updates when nodes reparented | Q2, Q4 - Edges must remain valid after reparenting |
| SUB-034 | Edge routing respects collapsed state | Q6, INV1 - Nested subgraphs in collapsed parents handled correctly |
