# Martin Fowler Test Patterns: Subgraph Tests (SUB-001 to SUB-034)

**Bead ID**: bd-1b9
**Title**: subgraph: Martin Fowler test patterns for subgraph operations
**Phase**: rust-contract
**Created**: 2026-03-03T00:00:00Z

## Overview

This document defines Martin Fowler's test patterns applied to the subgraph feature implementation. Each test follows the "Given-When-Then" structure and tests behavior rather than implementation.

## Test Patterns

### Pattern 1: State Verification (SUB-001, SUB-005, SUB-021)

**Purpose**: Verify the system reaches an expected state after an operation.

**Structure**:
```
Given: A system in initial state S1
When: Operation O is performed
Then: System is in state S2
```

**Example: SUB-005 - Parent-child relationship preservation**
```rust
#[test]
fn given_container_with_children_when_container_moved_then_children_preserve_parent() {
    // Given: Container with 2 children
    let container_id = create_container(canvas, (100, 100), (200, 200));
    let child1_id = create_node_in_container(canvas, (120, 120));
    let child2_id = create_node_in_container(canvas, (160, 160));

    // When: Move container by (50, 50)
    move_nodes(canvas, vec![container_id], (50, 50));

    // Then: Children still have parent reference
    assert_eq!(get_parent(child1_id), Some(container_id));
    assert_eq!(get_parent(child2_id), Some(container_id));

    // Then: Children positions shifted correctly
    let child1 = get_node(child1_id);
    assert_eq!(child1.x, 170); // 120 + 50
    assert_eq!(child1.y, 170);
}
```

### Pattern 2: Error Handling (SUB-006, SUB-013, SUB-022)

**Purpose**: Verify system handles invalid operations gracefully.

**Structure**:
```
Given: A system where operation O would be invalid
When: Operation O is attempted
Then: System returns error E and remains in consistent state
```

**Example: SUB-006 - Delete container reparents children**
```rust
#[test]
fn given_container_with_children_when_container_deleted_then_children_reparented_to_root() {
    // Given: Container with children
    let container_id = create_container(canvas, (100, 100), (200, 200));
    let child_id = create_node_in_container(canvas, (120, 120));

    // When: Delete container
    delete_nodes(canvas, vec![container_id]);

    // Then: Children still exist
    let child = get_node(child_id);
    assert!(child.is_some());

    // Then: Children have no parent (reparented to root)
    assert_eq!(child.unwrap().parent, None);
}
```

### Pattern 3: Invariant Preservation (SUB-007, SUB-019, SUB-033)

**Purpose**: Verify system maintains important invariants.

**Structure**:
```
Given: A system satisfying invariant I
When: Operation O is performed
Then: Invariant I still holds
```

**Example: SUB-007 - Duplicate container remaps IDs**
```rust
#[test]
fn given_container_with_edges_when_duplicated_then_ids_unique_and_edges_remapped() {
    // Given: Container with nodes and edges
    let container_id = create_container(canvas, (100, 100), (200, 200));
    let node1_id = create_node_in_container(canvas, (120, 120));
    let node2_id = create_node_in_container(canvas, (160, 160));
    let edge_id = create_edge(node1_id, node2_id);

    // When: Duplicate container
    let (new_container, new_nodes, new_edges) = duplicate(canvas, vec![container_id]);

    // Then: All new IDs are unique
    assert_ne!(new_container, container_id);
    for (old_id, new_node) in new_nodes {
        assert_ne!(new_node, old_id);
    }

    // Then: Edges reference new node IDs
    let new_edge = get_edge(new_edges[0]);
    assert!(new_nodes.values().any(|id| id == new_edge.source));
    assert!(new_nodes.values().any(|id| id == new_edge.target));
}
```

### Pattern 4: Boundary Testing (SUB-011, SUB-012, SUB-013)

**Purpose**: Verify behavior at boundaries of valid ranges.

**Structure**:
```
Given: A system at boundary condition B
When: Operation O is performed
Then: System handles boundary correctly
```

**Example: SUB-011 - Container auto-expand when child crosses boundary**
```rust
#[test]
fn given_child_at_container_edge_when_dragged_past_boundary_then_container_expands_or_child_constrained() {
    // Given: Container with child at right edge
    let container_id = create_container(canvas, (100, 100), (300, 200));
    let child_id = create_node_at(canvas, (360, 150)); // At right edge

    // When: Drag child past edge
    drag_node(canvas, child_id, (50, 0)); // Move 50px right

    // Then: Either container expanded OR child constrained
    let container = get_node(container_id);
    let child = get_node(child_id);

    let child_contained = child.x < container.x + container.width;
    let container_expanded = container.width > 200;

    assert!(child_contained || container_expanded);
}
```

### Pattern 5: Interaction Testing (SUB-008, SUB-009, SUB-010)

**Purpose**: Verify interactions between related components.

**Structure**:
```
Given: Components C1 and C2 with relationship R
When: Operation on C1 affects C2
Then: Relationship R is maintained correctly
```

**Example: SUB-008 - Drag child into container**
```rust
#[test]
fn given_node_outside_container_when_dragged_into_then_becomes_child() {
    // Given: Node outside container
    let container_id = create_container(canvas, (100, 100), (200, 200));
    let node_id = create_node_at(canvas, (50, 50));

    assert_eq!(get_parent(node_id), None);

    // When: Drag node into container
    drag_node_to(canvas, node_id, (150, 150));

    // Then: Node becomes child
    assert_eq!(get_parent(node_id), Some(container_id));

    // Then: Position is relative to container
    let node = get_node(node_id);
    let container = get_node(container_id);
    assert!(node.x >= container.x);
    assert!(node.x < container.x + container.width);
}
```

### Pattern 6: Lifecycle Testing (SUB-003, SUB-015, SUB-025)

**Purpose**: Verify behavior through object lifecycle.

**Structure**:
```
Given: Object O is created
When: O transitions through states S1, S2, S3
Then: Each transition preserves correctness
```

**Example: SUB-015 - Create empty subgraph container**
```rust
#[test]
fn given_empty_canvas_when_subgraph_created_then_container_exists_with_valid_state() {
    // Given: Empty canvas
    let canvas = create_canvas();

    // When: Create subgraph container
    let container_id = create_subgraph(canvas, (100, 100), (200, 200));

    // Then: Container exists
    let container = get_node(container_id);
    assert_eq!(container.kind, NodeKind::Subgraph);

    // Then: Container has valid bounds
    assert!(container.width > 0);
    assert!(container.height > 0);

    // Then: Container has no children initially
    assert_eq!(count_children(container_id), 0);
}
```

### Pattern 7: Property-Based Testing (SUB-019, SUB-020)

**Purpose**: Verify properties hold for many random inputs.

**Structure**:
```
Given: Random input I satisfying constraints
When: Operation O is performed on I
Then: Property P holds for all I
```

**Example: SUB-020 - Subgraph z-index ordering**
```rust
#[test]
fn given_containers_at_various_depths_when_rendered_then_z_index_ordering_correct() {
    // Property: Parent containers always have lower z-index than children
    let mut rng = StdRng::seed_from_u64(12345);

    for _ in 0..100 {
        // Given: Random container hierarchy
        let containers = create_random_hierarchy(&mut rng, 5);

        // Then: Parent z-index < child z-index for all pairs
        for (parent, child) in all_parent_child_pairs(&containers) {
            let parent_node = get_node(parent);
            let child_node = get_node(child);
            assert!(parent_node.z_index < child_node.z_index);
        }
    }
}
```

### Pattern 8: Edge Case Testing (SUB-026 to SUB-029)

**Purpose**: Verify behavior in unusual but valid scenarios.

**Structure**:
```
Given: Unusual but valid configuration C
When: Operation O is performed
Then: System handles C correctly
```

**Example: SUB-026 - Create subgraph within subgraph**
```rust
#[test]
fn given_container_when_another_container_created_inside_then_nested_hierarchy_valid() {
    // Given: Parent container
    let parent_id = create_container(canvas, (100, 100), (400, 400));

    // When: Create child container inside parent
    let child_id = create_container(canvas, (150, 150), (200, 200));

    // Then: Child is nested under parent
    assert_eq!(get_parent(child_id), Some(parent_id));

    // Then: Hierarchy depth is correct
    assert_eq!(get_depth(child_id), 2);
}
```

### Pattern 9: State Restoration (SUB-003, SUB-029)

**Purpose**: Verify state can be restored after temporary change.

**Structure**:
```
Given: System in state S1
When: Operation changes to S2 then reversed
Then: System returns to S1
```

**Example: SUB-003 - Collapse/expand container behavior**
```rust
#[test]
fn given_container_with_children_when_collapsed_then_expanded_then_state_restored() {
    // Given: Expanded container with children
    let container_id = create_container(canvas, (100, 100), (200, 200));
    let child_id = create_node_in_container(canvas, (120, 120));
    set_collapsed(container_id, false);

    let child_visible_before = is_visible(child_id);

    // When: Collapse
    set_collapsed(container_id, true);
    let child_visible_collapsed = is_visible(child_id);

    // Then: Children hidden
    assert!(!child_visible_collapsed);

    // When: Expand
    set_collapsed(container_id, false);
    let child_visible_after = is_visible(child_id);

    // Then: Children visible again
    assert!(child_visible_after);
    assert_eq!(child_visible_before, child_visible_after);
}
```

### Pattern 10: Integration Testing (SUB-030 to SUB-034)

**Purpose**: Verify multiple features work together correctly.

**Structure**:
```
Given: Features F1, F2, F3 integrated
When: Combined operation O uses all features
Then: All features work together correctly
```

**Example: SUB-033 - Edge updates when nodes reparented**
```rust
#[test]
fn given_edge_between_nodes_when_nodes_reparented_then_edge_routing_updated() {
    // Given: Two nodes with edge, both outside container
    let node1_id = create_node_at(canvas, (100, 100));
    let node2_id = create_node_at(canvas, (300, 100));
    let edge_id = create_edge(node1_id, node2_id);

    let edge_before = get_edge(edge_id);

    // When: Move both nodes into container
    let container_id = create_container(canvas, (50, 50), (400, 200));
    reparent_node(canvas, node1_id, container_id);
    reparent_node(canvas, node2_id, container_id);

    // Then: Edge still connects same nodes
    let edge_after = get_edge(edge_id);
    assert_eq!(edge_after.source, node1_id);
    assert_eq!(edge_after.target, node2_id);

    // Then: Edge routing updated for new container context
    // (e.g., bend points adjusted for container offset)
    assert_ne!(edge_before.bend_points, edge_after.bend_points);
}
```

## Test Naming Convention

All tests follow the pattern:
```
given_<precondition>_when_<action>_then_<expected_outcome>
```

This convention makes tests self-documenting and easier to understand.

## Test Organization

### By Feature
- Selection tests: `subgraph_selection_tests.rs`
- Reparenting tests: `subgraph_reparenting_tests.rs`
- Container behavior tests: `subgraph_container_tests.rs`
- Edge routing tests: `subgraph_edge_tests.rs`

### By Category
- Unit tests: Test individual functions
- Integration tests: Test component interactions
- E2E tests: Test complete user workflows

## Test Helpers

Common test utilities to reduce duplication:
```rust
mod test_helpers {
    pub fn create_container(canvas: &Canvas, x: f64, y: f64, w: f64, h: f64) -> NodeId;
    pub fn create_node_in_container(canvas: &Canvas, container: NodeId, x: f64, y: f64) -> NodeId;
    pub fn get_parent(node_id: NodeId) -> Option<NodeId>;
    pub fn set_collapsed(node_id: NodeId, collapsed: bool);
    pub fn is_visible(node_id: NodeId) -> bool;
    pub fn reparent_node(canvas: &Canvas, node: NodeId, new_parent: NodeId);
}
```

## Quality Checklist

Each test must satisfy:
- [ ] Descriptive name following convention
- [ ] Clear Given-When-Then structure
- [ ] Tests behavior, not implementation
- [ ] Independent of other tests
- [ ] Deterministic (no randomness or race conditions)
- [ ] Fast (< 1 second per test)
- [ ] Clear assertion messages
- [ ] No hardcoded values that might change
- [ ] Handles errors appropriately
- [ ] Documents non-obvious behavior

## References

- Martin Fowler's "Test Patterns": https://martinfowler.com/bliki/TestPattern.html
- Given-When-Then: https://martinfowler.com/bliki/GivenWhenThen.html
- Test-Driven Development: https://martinfowler.com/bliki/TestDrivenDevelopment.html
