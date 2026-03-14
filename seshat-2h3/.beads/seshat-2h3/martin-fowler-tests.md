# Martin Fowler Test Plan: Subgraph Deletion Cascade

## Test Organization

This test plan follows the Given-When-Then pattern from Martin Fowler's testing approach. Tests are organized into:
- Happy Path Tests (successful operations)
- Error Path Tests (failure cases)
- Edge Case Tests (boundary conditions)
- Contract Verification Tests (pre/postconditions)
- Contract Violation Tests (explicit violation examples from contract)

---

## DSL Layer (Business-Readable Specifications)

Per ATDD principles, these helper functions provide a business-readable validation language:

```rust
// DSL - Business readable specifications
fn verify_cascade_deletion(doc: &DiagramDocument, mode: CascadeMode) -> CascadeVerification
fn assert_children_reparented(doc: &DiagramDocument, children: &[NodeId], new_parent: Option<&NodeId>) -> bool
fn assert_children_deleted(doc: &DiagramDocument, children: &[NodeId]) -> bool
fn assert_valid_parent_chain(doc: &DiagramDocument) -> bool
fn assert_no_orphan_edges(doc: &DiagramDocument) -> bool
fn assert_node_count_consistent(doc: &DiagramDocument, expected: usize) -> bool
fn assert_selection_correct(doc: &DiagramDocument, mode: CascadeMode, orphaned_children: &[NodeId]) -> bool
```

---

## Happy Path Tests

### test_subgraph_deleted_in_reparent_mode
**Category**: Happy Path  
**Coverage**: Q1

**Given**: A document with a root-level subgraph
```
Document:
  - subgraph: Subgraph (NodeId="subgraph", parent=None)
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Subgraph is removed from document

---

### test_child_reparented_to_grandparent
**Category**: Happy Path  
**Coverage**: Q2

**Given**: A document with a subgraph that has a grandparent, containing a child node
```
Document:
  - grandparent: Subgraph (NodeId="grandparent", parent=None)
  - parent: Subgraph (NodeId="parent", parent=Some("grandparent"))  
  - child: Node (NodeId="child", parent=Some("parent"))
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Child's parent is updated to grandparent

---

### test_child_reparented_to_root
**Category**: Happy Path  
**Coverage**: Q2

**Given**: A document with a root-level subgraph containing a child node
```
Document:
  - subgraph: Subgraph (NodeId="subgraph", parent=None)
  - child: Node (NodeId="child", parent=Some("subgraph"))
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Child's parent is updated to None (root level)

---

### test_child_deleted_in_delete_mode
**Category**: Happy Path  
**Coverage**: Q3

**Given**: A document with a subgraph containing a child node
```
Document:
  - subgraph: Subgraph (NodeId="subgraph", parent=None)
  - child: Node (NodeId="child", parent=Some("subgraph"))
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Delete)` is called

**Then**: Child is removed from document

---

### test_nested_subgraph_deleted_in_reparent_mode
**Category**: Happy Path  
**Coverage**: Q6, SUB-032

**Given**: A document with nested subgraphs
```
Document:
  - outer: Subgraph (NodeId="outer", parent=None)
  - inner: Subgraph (NodeId="inner", parent=Some("outer"))
  - child: Node (NodeId="child", parent=Some("inner"))
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called with outer selected

**Then**: Outer and inner subgraphs are removed

---

### test_nested_child_reparented
**Category**: Happy Path  
**Coverage**: Q2, Q6

**Given**: A document with nested subgraphs
```
Document:
  - outer: Subgraph (NodeId="outer", parent=None)
  - inner: Subgraph (NodeId="inner", parent=Some("outer"))
  - child: Node (NodeId="child", parent=Some("inner"))
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called with outer selected

**Then**: Child's parent is updated to None (outer had no grandparent)

---

### test_edge_preserved_after_reparent
**Category**: Happy Path  
**Coverage**: Q4, SUB-033

**Given**: A subgraph with a child that has an external edge
```
Document:
  - subgraph: Subgraph (NodeId="subgraph", parent=None)
  - child: Node (NodeId="child", parent=Some("subgraph"))
  - external: Node (NodeId="external", parent=None)
  - edge: Edge (EdgeId="edge1", source=child, target=external)
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Edge is preserved in document

---

### test_edge_endpoint_still_exists_after_reparent
**Category**: Happy Path  
**Coverage**: Q4

**Given**: A subgraph with a child that has an external edge
```
Document:
  - subgraph: Subgraph (NodeId="subgraph", parent=None)
  - child: Node (NodeId="child", parent=Some("subgraph"))
  - external: Node (NodeId="external", parent=None)
  - edge: Edge (EdgeId="edge1", source=child, target=external)
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Child remains in document with updated parent

---

---

## Error Path Tests

### test_returns_error_when_no_subgraph_selected
**Category**: Error Path  
**Coverage**: P1

**Given**: An empty document or document with no subgraphs selected
```
EditorState:
  selected_items: []
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Returns `Err(GroupingError::EmptySelection)`

---

### test_returns_error_when_selected_node_is_not_subgraph
**Category**: Error Path  
**Coverage**: P1

**Given**: A regular node selected (not a subgraph)
```
Document:
  - node: Node (NodeId="node", kind=NodeKind::Text)
EditorState:
  selected_items: ["node"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Returns `Err(GroupingError::EmptySelection)` (no subgraphs in selection)

---

### test_returns_error_when_subgraph_is_locked
**Category**: Error Path  
**Coverage**: P2

**Given**: A locked subgraph is selected
```
Document:
  - subgraph: Subgraph (NodeId="subgraph", locked=true)
EditorState:
  selected_items: ["subgraph"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Returns `Err(GroupingError::LockedNode(subgraph))`

---

## Edge Case Tests

### test_handles_empty_subgraph_gracefully
**Category**: Edge Case  
**Coverage**: Q1

**Given**: A subgraph with no children
```
Document:
  - empty_subgraph: Subgraph (NodeId="empty", parent=None, has no children)
EditorState:
  selected_items: ["empty"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Subgraph is removed from document

---

### test_handles_subgraph_with_only_nested_subgraphs
**Category**: Edge Case  
**Coverage**: Q3, Q6

**Given**: A subgraph containing only nested subgraphs (no leaf nodes)
```
Document:
  - parent: Subgraph (NodeId="parent")
  - child1: Subgraph (NodeId="child1", parent=Some("parent"))
  - child2: Subgraph (NodeId="child2", parent=Some("parent"))
EditorState:
  selected_items: ["parent"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: All subgraphs are removed

---

### test_handles_deeply_nested_subgraph
**Category**: Edge Case  
**Coverage**: Q2, Q6, SUB-034

**Given**: A deeply nested subgraph structure (4 levels deep)
```
Document:
  - level1: Subgraph (NodeId="level1", parent=None)
  - level2: Subgraph (NodeId="level2", parent=Some("level1"))
  - level3: Subgraph (NodeId="level3", parent=Some("level2"))
  - level4: Subgraph (NodeId="level4", parent=Some("level3"))
  - leaf: Node (NodeId="leaf", parent=Some("level4"))
EditorState:
  selected_items: ["level1"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Leaf node is reparented to None

---

### test_edge_to_deleted_node_removed_in_delete_mode
**Category**: Edge Case  
**Coverage**: Q3, Q4

**Given**: A subgraph with a child that has an edge
```
Document:
  - subgraph: Subgraph (NodeId="subgraph")
  - child: Node (NodeId="child", parent=Some("subgraph"))
  - external: Node (NodeId="external")
  - edge: Edge (source=child, target=external)
EditorState:
  selected_items: ["subgraph"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Delete)` is called

**Then**: Edge is removed from document

---

### test_handles_multiple_edges_to_subgraph_children
**Category**: Edge Case  
**Coverage**: Q4, INV2

**Given**: Multiple edges connected to children of a subgraph
```
Document:
  - subgraph: Subgraph (NodeId="subgraph")
  - child1: Node (NodeId="child1", parent=Some("subgraph"))
  - child2: Node (NodeId="child2", parent=Some("subgraph"))
  - external1: Node (NodeId="external1")
  - external2: Node (NodeId="external2")
  - edge1: Edge (source=child1, target=external1)
  - edge2: Edge (source=child2, target=external2)
EditorState:
  selected_items: ["subgraph"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Edge1 is preserved in document

---

### test_handles_multiple_edges_to_subgraph_children_2
**Category**: Edge Case  
**Coverage**: Q4, INV2

**Given**: Multiple edges connected to children of a subgraph
```
Document:
  - subgraph: Subgraph (NodeId="subgraph")
  - child1: Node (NodeId="child1", parent=Some("subgraph"))
  - child2: Node (NodeId="child2", parent=Some("subgraph"))
  - external1: Node (NodeId="external1")
  - external2: Node (NodeId="external2")
  - edge1: Edge (source=child1, target=external1)
  - edge2: Edge (source=child2, target=external2)
EditorState:
  selected_items: ["subgraph"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Edge2 is preserved in document

---

### test_handles_edge_from_child_to_subgraph_itself
**Category**: Edge Case  
**Coverage**: Q4

**Given**: An edge from a child node to the subgraph container
```
Document:
  - subgraph: Subgraph (NodeId="subgraph")
  - child: Node (NodeId="child", parent=Some("subgraph"))
  - edge: Edge (source=child, target=subgraph)
EditorState:
  selected_items: ["subgraph"]
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` is called

**Then**: Edge is removed from document

---

## Contract Verification Tests

### test_precondition_p1_no_subgraph_selected
**Category**: Contract Verification  
**Coverage**: P1

**Given**: An empty selection
```
doc.editor_state.selected_items = {}
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent)`

**Then**: Returns `Err(GroupingError::EmptySelection)`

---

### test_precondition_p2_locked_subgraph
**Category**: Contract Verification  
**Coverage**: P2

**Given**: A subgraph with `locked = true`
```
doc.document.nodes.insert(subgraph_id, Node { locked: true, .. });
doc.editor_state.selected_items.insert(subgraph_id.as_str());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent)`

**Then**: Returns `Err(GroupingError::LockedNode(subgraph_id))`

---

### test_postcondition_q1_subgraph_removed
**Category**: Contract Verification  
**Coverage**: Q1

**Given**: A document with a subgraph
```
doc.document.nodes.insert(subgraph_id, subgraph_node());
doc.editor_state.selected_items.insert(subgraph_id.as_str());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent)` succeeds

**Then**: `doc.document.nodes.get(&subgraph_id)` returns `None`

---

### test_postcondition_q2_children_reparented
**Category**: Contract Verification  
**Coverage**: Q2

**Given**: A subgraph with children, and the subgraph has a parent
```
let grandparent = NodeId::new("grandparent");
let parent = NodeId::new("parent");
let child = NodeId::new("child");

doc.document.nodes.insert(grandparent.clone(), subgraph_node());
doc.document.nodes.insert(parent.clone(), subgraph_node_with_parent(Some(grandparent.clone())));
doc.document.nodes.insert(child.clone(), node_with_parent(parent.clone()));
doc.editor_state.selected_items.insert(parent.as_str());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent)` succeeds

**Then**: `doc.document.nodes.get(&child).parent` equals `Some(grandparent)`

---

### test_postcondition_q3_children_deleted
**Category**: Contract Verification  
**Coverage**: Q3

**Given**: A subgraph with children
```
doc.document.nodes.insert(subgraph_id, subgraph_node());
doc.document.nodes.insert(child_id, node_with_parent(subgraph_id.clone()));
doc.editor_state.selected_items.insert(subgraph_id.as_str());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete)` succeeds

**Then**: `doc.document.nodes.get(&child_id)` returns `None`

---

### test_postcondition_q4_edges_cleaned_up
**Category**: Contract Verification  
**Coverage**: Q4

**Given**: A subgraph with a child that has an edge to an external node
```
doc.document.nodes.insert(subgraph_id, subgraph_node());
doc.document.nodes.insert(child_id, node_with_parent(subgraph_id.clone()));
doc.document.nodes.insert(external_id, node_node());
doc.document.edges.insert(edge_id, edge(child_id, external_id));
doc.editor_state.selected_items.insert(subgraph_id.as_str());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete)` succeeds

**Then**: `doc.document.edges.get(&edge_id)` returns `None`

---

### test_postcondition_q5_selection_updated_reparent
**Category**: Contract Verification  
**Coverage**: Q5

**Given**: A document with a subgraph containing a child
```
let subgraph_id = NodeId::new("subgraph");
let child_id = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &subgraph_id, &child_id);
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent)` succeeds

**Then**: `doc.editor_state.selected_items` contains the orphaned child ID

---

### test_postcondition_q5_selection_updated_delete
**Category**: Contract Verification  
**Coverage**: Q5

**Given**: A document with a subgraph containing a child
```
let subgraph_id = NodeId::new("subgraph");
let child_id = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &subgraph_id, &child_id);
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete)` succeeds

**Then**: `doc.editor_state.selected_items` is empty

---

### test_invariant_inv1_parent_chain_valid
**Category**: Contract Verification  
**Coverage**: INV1

**Given**: A document after reparenting children
```
// Setup and perform reparent
ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent).unwrap();
```

**Then**: For all nodes `n` with `n.parent == Some(pid)`, `doc.nodes.get(pid).kind == Subgraph`

---

### test_invariant_inv2_no_orphan_edges
**Category**: Contract Verification  
**Coverage**: INV2

**Given**: A document after deletion
```
ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete).unwrap();
```

**Then**: For all edges `e`, `doc.nodes.contains_key(&e.source)` and `doc.nodes.contains_key(&e.target)`

---

### test_invariant_inv3_node_count_consistency_reparent
**Category**: Contract Verification  
**Coverage**: INV3

**Given**: A document with known node count (1 subgraph + 2 children = 3 nodes)
```
let subgraph_id = NodeId::new("subgraph");
let child1_id = NodeId::new("child1");
let child2_id = NodeId::new("child2");
let original_count = doc.document.nodes.len();
setup_subgraph_with_children(&mut doc, &subgraph_id, &[&child1_id, &child2_id]);
let expected_after_reparent = original_count - 1; // subgraph removed, children remain
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent)` succeeds

**Then**: `doc.document.nodes.len()` equals `expected_after_reparent`

---

### test_invariant_inv3_node_count_consistency_delete
**Category**: Contract Verification  
**Coverage**: INV3

**Given**: A document with known node count (1 subgraph + 2 children = 3 nodes)
```
let subgraph_id = NodeId::new("subgraph");
let child1_id = NodeId::new("child1");
let child2_id = NodeId::new("child2");
setup_subgraph_with_children(&mut doc, &subgraph_id, &[&child1_id, &child2_id]);
let original_count = doc.document.nodes.len();
let expected_after_delete = original_count - 3; // subgraph + children all removed
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete)` succeeds

**Then**: `doc.document.nodes.len()` equals `expected_after_delete`

---

## Contract Violation Tests

### test_p1_violation_returns_empty_selection_error
**Category**: Contract Violation  
**Coverage**: VIOLATES P1

**Given**: An empty selection (no subgraphs selected)
```
doc.editor_state.selected_items.clear();
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent)`

**Then**: Returns `Err(GroupingError::EmptySelection)` -- NOT a panic, NOT an unwrap failure

---

### test_p2_violation_returns_locked_node_error
**Category**: Contract Violation  
**Coverage**: VIOLATES P2

**Given**: A locked subgraph selected
```
let subgraph_id = NodeId::new("locked-subgraph");
doc.document.nodes.insert(subgraph_id.clone(), Node { locked: true, .. });
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());
```

**When**: `ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent)`

**Then**: Returns `Err(GroupingError::LockedNode(subgraph_id))` -- NOT a panic

---

### test_q2_violation_children_not_reparented
**Category**: Contract Violation  
**Coverage**: VIOLATES Q2

**Given**: A subgraph with a child, Reparent mode
```
let parent = NodeId::new("parent");
let child = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &parent, &child);

ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent).unwrap();
```

**Then (Violation)**: Child's parent should NOT still be the deleted subgraph
```
let actual_parent = doc.document.nodes.get(&child).unwrap().parent.clone();
assert_ne!(actual_parent, Some(parent), "VIOLATION: Child should be reparented");
```

---

### test_q3_violation_children_still_exist
**Category**: Contract Violation  
**Coverage**: VIOLATES Q3

**Given**: A subgraph with children, Delete mode
```
let subgraph_id = NodeId::new("subgraph");
let child_id = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &subgraph_id, &child_id);

ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete).unwrap();
```

**Then (Violation)**: Child should NOT exist in document
```
assert!(!doc.document.nodes.contains_key(&child_id), "VIOLATION: Child should be deleted");
```

---

### test_q4_violation_orphan_edge_remains
**Category**: Contract Violation  
**Coverage**: VIOLATES Q4

**Given**: A subgraph with internal edge
```
let subgraph_id = NodeId::new("subgraph");
let internal_node = NodeId::new("internal");
let edge_id = EdgeId::new("edge1");
setup_subgraph_with_edge(&mut doc, &subgraph_id, &internal_node, &edge_id);

ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent).unwrap();
```

**Then (Violation)**: Edge to deleted subgraph should be removed
```
assert!(!doc.document.edges.contains_key(&edge_id), "VIOLATION: Edge should be removed");
```

---

### test_q5_violation_selection_not_updated_reparent
**Category**: Contract Violation  
**Coverage**: VIOLATES Q5

**Given**: A subgraph with children, Reparent mode
```
let subgraph_id = NodeId::new("subgraph");
let child_id = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &subgraph_id, &child_id);
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());

ungroup_selection_with_cascade(&mut doc, CascadeMode::Reparent).unwrap();
```

**Then (Violation)**: Selection should contain orphaned child
```
assert!(!doc.editor_state.selected_items.contains(&child_id.to_string()), 
    "VIOLATION: Selection should contain orphaned child in reparent mode");
```

---

### test_q5_violation_selection_not_updated_delete
**Category**: Contract Violation  
**Coverage**: VIOLATES Q5

**Given**: A subgraph with children, Delete mode
```
let subgraph_id = NodeId::new("subgraph");
let child_id = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &subgraph_id, &child_id);
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());

ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete).unwrap();
```

**Then (Violation)**: Selection should be empty
```
assert!(!doc.editor_state.selected_items.is_empty(), 
    "VIOLATION: Selection should be empty in delete mode");
```

---

### test_inv3_violation_node_count_inconsistent
**Category**: Contract Violation  
**Coverage**: VIOLATES INV3

**Given**: A document with known node count
```
let subgraph_id = NodeId::new("subgraph");
let child_id = NodeId::new("child");
setup_subgraph_with_child(&mut doc, &subgraph_id, &child_id);
doc.editor_state.selected_items.insert(subgraph_id.as_str().to_string());

ungroup_selection_with_cascade(&mut doc, CascadeMode::Delete).unwrap();
```

**Then (Violation)**: Node count should be reduced by 2 (subgraph + child)
```
let expected = doc.document.nodes.len() + 2;
assert_ne!(expected, doc.document.nodes.len(), "VIOLATION: Node count should decrease");
```

---

## Given-When-Then Scenarios

### Scenario 1: Complete Delete Cascade Flow
**Given**: A complex document with nested subgraphs, children, and edges
```
- outer: Subgraph (parent=None)
  - inner: Subgraph (parent=Some("outer"))
    - leaf: Node (parent=Some("inner"))
  - sibling: Node (parent=Some("outer"))
  - edge: Edge (source=leaf, target=sibling)
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Delete)`

**Then**: Document is empty

---

### Scenario 2: Complete Reparent Cascade Flow
**Given**: A complex document with nested subgraphs, children, and edges
```
- outer: Subgraph (parent=None)
  - inner: Subgraph (parent=Some("outer"))
    - leaf: Node (parent=Some("inner"))
  - sibling: Node (parent=Some("outer"))
  - edge: Edge (source=leaf, target=sibling)
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)`

**Then**: 
- outer and inner subgraphs are removed
- leaf and sibling nodes have parent=None
- edge is preserved
- document contains leaf, sibling, and edge

---

### Scenario 3: Edge Routing SUB-032 (Nested Subgraph Edges)
**Given**: An edge between nodes in different nested subgraphs
```
- outer: Subgraph
  - inner1: Subgraph
    - node1: Node (parent=Some("inner1"))
  - inner2: Subgraph  
    - node2: Node (parent=Some("inner2"))
  - edge: Edge (source=node1, target=node2)
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` on outer

**Then**: Edge is preserved

---

### Scenario 4: Edge Updates SUB-033 (Reparent Edge Validity)
**Given**: An edge from a subgraph child to an external node
```
- subgraph: Subgraph
  - child: Node (parent=Some("subgraph"))
- external: Node
- edge: Edge (source=child, target=external)
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)`

**Then**: Edge is still valid and preserved

---

### Scenario 5: Collapsed State SUB-034 (Nested Handling)
**Given**: A collapsed parent subgraph containing a nested subgraph with children
```
- collapsed_parent: Subgraph (collapsed=Some(true))
  - nested: Subgraph (parent=Some("collapsed_parent"))
    - leaf: Node (parent=Some("nested"))
```

**When**: `ungroup_selection_with_cascade(doc, CascadeMode::Reparent)` on collapsed_parent

**Then**: Invariant INV1 is maintained

---

## Test Implementation Notes

### Test File Location
- Unit tests: `diagram_tool/src/core/grouping_tests.rs`
- Integration tests: `diagram_tool/tests/subgraph_cascade_tests.rs`

### Test Utilities Needed
```rust
// Helper to create test nodes
fn test_node(id: &str, parent: Option<NodeId>) -> Node
fn test_subgraph(id: &str, parent: Option<NodeId>, collapsed: Option<bool>) -> Node

// Helper to setup documents
fn setup_subgraph_with_child(doc: &mut DiagramDocument, subgraph_id: &NodeId, child_id: &NodeId)
fn setup_subgraph_with_children(doc: &mut DiagramDocument, subgraph_id: &NodeId, children: &[&NodeId])
fn setup_subgraph_with_edge(doc: &mut DiagramDocument, subgraph_id: &NodeId, internal_id: &NodeId, edge_id: &EdgeId)

// Cascade mode enum (to be added)
enum CascadeMode { Reparent, Delete }
```

### Running Tests
```bash
# Run all subgraph cascade tests
cargo test --lib grouping::tests

# Run specific test
cargo test --lib test_subgraph_deleted_in_reparent_mode
```
