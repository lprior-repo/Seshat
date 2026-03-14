# Martin Fowler Test Plan: seshat-zai (SUB-008 to SUB-012: Subgraph un-grouping)

**Bead ID**: seshat-zai
**Feature**: Subgraph un-grouping (flattening)
**Created**: 2026-03-14

## Happy Path Tests

### test_ungroup_single_subgraph_with_children_removes_subgraph_and_reparents_children
- **Given**: A document with one subgraph containing two child nodes
- **When**: `ungroup_selection` is called
- **Then**:
  - Subgraph node is removed from document
  - Child nodes have their parent set to None (root level)
  - Selected items contains both child node IDs
  - Node count decreases by 1

### test_ungroup_nested_subgraph_reparents_to_grandparent
- **Given**: A document with parent subgraph containing a nested child subgraph, which contains a leaf node
- **When**: Child subgraph is selected and ungrouped
- **Then**:
  - Child subgraph is removed
  - Leaf node is reparented to parent subgraph (grandparent)
  - Parent subgraph remains in document
  - Leaf node selected after operation

### test_ungroup_multiple_subgraphs_simultaneously
- **Given**: A document with two sibling subgraphs, each containing children
- **When**: Both subgraphs are selected and ungrouped
- **Then**:
  - Both subgraphs are removed
  - All children are reparented to the subgraphs' parent (or root)
  - All children are selected after operation

### test_ungroup_subgraph_with_no_children
- **Given**: An empty subgraph (no children)
- **When**: Subgraph is selected and ungrouped
- **Then**:
  - Subgraph is removed from document
  - Node count decreases by 1
  - Selected items is empty (no orphans)

### test_ungroup_subgraph_at_root_level
- **Given**: A subgraph with no parent (root level), containing children
- **When**: Subgraph is selected and ungrouped
- **Then**:
  - Subgraph is removed
  - Children have parent set to None (stay at root)
  - Children are selected

## Error Path Tests

### test_ungroup_empty_selection_returns_error
- **Given**: An empty document with no selection
- **When**: `ungroup_selection` is called
- **Then**: Returns `Err(GroupingError::EmptySelection)`

### test_ungroup_no_subgraphs_in_selection_returns_error
- **Given**: A document with regular nodes (not subgraphs) selected
- **When**: `ungroup_selection` is called
- **Then**: Returns `Err(GroupingError::EmptySelection)` (no subgraphs to ungroup)

### test_ungroup_nonexistent_node_selected_returns_error
- **Given**: A document with a selected ID that doesn't exist
- **When**: `ungroup_selection` is called
- **Then**: Returns `Err(GroupingError::NodeNotFound(_))`

## Edge Case Tests

### test_ungroup_deeply_nested_chain
- **Given**: A chain of 5 nested subgraphs (depth 5), each with one child
- **When**: Innermost subgraph is ungrouped
- **Then**:
  - Innermost subgraph removed
  - Its child reparented to parent subgraph
  - INV1 (parent chain valid) holds
  - INV2 (no orphan edges) holds

### test_ungroup_subgraph_with_edges_to_external_nodes
- **Given**: A subgraph with edges connecting to nodes outside the subgraph
- **When**: Subgraph is ungrouped
- **Then**:
  - Edges connecting to the subgraph are removed
  - Edges between children within subgraph are preserved
  - INV2 (no orphan edges) holds

### test_ungroup_preserves_sibling_relationships
- **Given**: Two sibling nodes inside a subgraph with an edge between them
- **When**: Parent subgraph is ungrouped
- **Then**:
  - Edge between siblings is preserved
  - Both nodes remain connected

### test_ungroup_multiple_nested_levels_at_once
- **Given**: Nested subgraphs at multiple levels, parent selected
- **When**: Parent subgraph is ungrouped
- **Then**:
  - Parent subgraph removed
  - Direct children reparented to grandparent (or root)
  - Nested subgraphs remain intact (not recursively flattened)

## Contract Verification Tests

### test_precondition_p1_empty_selection_rejected
- **Given**: Empty document
- **When**: `ungroup_selection` called
- **Then**: Returns `Err(GroupingError::EmptySelection)`

### test_precondition_p2_non_subgraph_selection_rejected
- **Given**: Document with text nodes selected
- **When**: `ungroup_selection` called
- **Then**: Returns `Err(GroupingError::EmptySelection)` (no subgraphs)

### test_postcondition_q1_subgraph_removed
- **Given**: Document with subgraph selected
- **When**: `ungroup_selection` succeeds
- **Then**: `doc.document.nodes.contains_key(&subgraph_id)` is false

### test_postcondition_q2_children_reparented
- **Given**: Document with subgraph containing children
- **When**: `ungroup_selection` succeeds
- **Then**: All children have parent == original_subgraph.parent

### test_postcondition_q3_edges_cleaned_up
- **Given**: Document with subgraph that has edges to external nodes
- **When**: `ungroup_selection` succeeds
- **Then**: No edges have source or target == removed_subgraph_id

### test_postcondition_q4_orphans_selected
- **Given**: Document with subgraph containing children
- **When**: `ungroup_selection` succeeds
- **Then**: All direct children are in `editor_state.selected_items`

### test_postcondition_q5_node_count_correct
- **Given**: Document with N subgraphs selected
- **When**: `ungroup_selection` succeeds
- **Then**: `doc.document.nodes.len() == original_len - N`

### test_postcondition_q6_parent_chain_valid
- **Given**: Any valid document state
- **When**: `ungroup_selection` succeeds
- **Then**: All nodes have valid parent references (parent is None or exists as Subgraph)

### test_invariant_inv1_parent_chain_valid_after_ungroup
- **Given**: Complex nested document
- **When**: Multiple ungroup operations
- **Then**: INV1 holds: every node.parent is None or points to existing Subgraph

### test_invariant_inv2_no_orphan_edges_after_ungroup
- **Given**: Document with edges to subgraphs
- **When**: Subgraphs ungrouped
- **Then**: INV2 holds: all edges connect to existing nodes

## Contract Violation Tests

### test_violation_p1_empty_selection
```
Given: empty document (no nodes, no selection)
When: ungroup_selection(&mut doc)
Then: returns Err(GroupingError::EmptySelection)
```

### test_violation_q1_subgraph_not_removed
```
Given: doc with subgraph "sg1" containing child "c1"
When: ungroup_selection(&mut doc) succeeds
Then: !doc.document.nodes.contains_key(&NodeId::new("sg1"))
```

### test_violation_q2_child_still_has_old_parent
```
Given: doc with subgraph "sg1" (parent=None) containing child "c1"
When: ungroup_selection(&mut doc) succeeds
Then: doc.document.nodes.get(&NodeId::new("c1")).parent == None
```

### test_violation_q3_edge_not_removed
```
Given: doc with edge from "sg1" to "n1", subgraph "sg1" selected
When: ungroup_selection(&mut doc) succeeds
Then: !doc.document.edges.values().any(|e| e.source == "sg1" || e.target == "sg1")
```

### test_violation_inv1_invalid_parent_reference
```
Given: doc where node has parent pointing to deleted subgraph
When: calculate_ungroup produces result
Then: all parent references point to existing Subgraph nodes or None
```

## Given-When-Then Scenarios

### Scenario 1: Basic Ungroup
Given: A diagram with a group "G1" containing nodes "N1" and "N2", where G1.parent = None
When: User selects G1 and invokes ungroup
Then:
- G1 is removed from the document
- N1.parent = None
- N2.parent = None
- N1 and N2 are selected
- Edges to/from G1 are removed
- Edge between N1 and N2 (if any) is preserved

### Scenario 2: Nested Ungroup
Given: A diagram with parent group "P" containing child group "C", which contains node "N"
When: User selects C and invokes ungroup
Then:
- C is removed from the document
- N.parent = P (reparented to grandparent)
- P remains in document
- N is selected
- INV1 holds: N.parent is valid Subgraph P

### Scenario 3: Ungroup with External Edges
Given: A diagram with group "G1" containing N1, with edge from G1 to N2 (outside)
When: User selects G1 and invokes ungroup
Then:
- Edge (G1 -> N2) is removed
- Edge (N1 -> N2) if exists is preserved
- INV2 holds: all edges connect to existing nodes
