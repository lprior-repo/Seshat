bead_id: bd-sa6
bead_title: tests: Implement SUB subgraph tests 4/4
phase: p1
updated_at: 2026-03-01T23:05:00Z

# Implementation: SUB Subgraph Interaction Tests (4/4)

## Summary

Implemented 13 subgraph/container interaction tests covering click-through selection, box-select across containers, collapse/expand behavior, and locked container interactions.

## Implementation Location

File: `/home/lewis/src/bd-sa6/diagram_tool/src/ui/canvas/interaction_reducer.rs`

Added a new test module `subgraph_tests` at the end of the file.

## Test Cases Implemented

### SUB-001: Click inside container selects child vs container (2 tests)

1. `given_container_with_child_when_hit_testing_then_child_has_higher_z_index`
   - Validates that children have higher z_index (1000) than containers (-1)
   - Verifies the `within()` function correctly identifies geometric containment

2. `given_nested_nodes_when_selecting_by_position_then_highest_z_index_wins`
   - Tests nested container hierarchy with outer, inner, and child nodes
   - Confirms z_index ordering: child > inner container = outer container

### SUB-002: Box-select across container boundary (2 tests)

3. `given_nodes_inside_and_outside_container_when_rubberband_selection_then_all_selectable`
   - Tests that rubber-band selection can select nodes both inside and outside containers
   - Verifies selection is not constrained by container boundaries

4. `given_partial_container_overlap_when_rubberband_then_only_overlapping_selected`
   - Tests partial container overlap during box selection
   - Verifies only nodes within the selection area are selected (not all container children)

### SUB-003: Collapse/expand container behavior (3 tests)

5. `given_container_with_collapsed_state_when_roundtripped_then_state_preserved`
   - Tests serialization/deserialization of collapsed state
   - Verifies `collapsed: Option<bool>` is preserved through round-trip

6. `given_expanded_container_when_collapsed_then_children_remain_in_document`
   - Tests that children remain in the document when container is collapsed
   - Verifies collapse is a visual state, not a structural change

7. `given_multiple_containers_when_collapsed_independently_then_states_are_independent`
   - Tests that multiple containers maintain independent collapsed states
   - Verifies no state propagation between sibling containers

### SUB-004: Locked container with unlocked children (3 tests)

8. `given_locked_container_with_unlocked_children_then_children_are_independently_unlocked`
   - Tests that lock state is per-node, not inherited
   - Verifies child's `locked: false` is independent of parent's `locked: true`

9. `given_locked_container_when_selecting_unlocked_child_then_child_is_selectable`
   - Tests that unlocked children remain selectable inside locked containers
   - Verifies selection behavior respects per-node lock state

10. `given_mixed_lock_hierarchy_then_lock_states_are_per_node`
    - Tests complex lock hierarchy with outer (unlocked), inner (locked), child (unlocked)
    - Verifies each node maintains its own lock state

### SUB-005: Parent-child relationship preservation (3 tests)

11. `given_container_with_children_when_selected_then_children_included_in_resize_targets`
    - Tests `resize_target_ids()` function includes children when container is selected
    - Verifies nodes outside container are excluded from resize targets

12. `given_container_with_children_when_resizing_then_parent_references_preserved`
    - Tests that parent references are preserved during resize operations
    - Uses `finalize_motion_release()` to simulate resize finalization

13. `given_nested_containers_then_parent_chain_is_correct`
    - Tests parent chain integrity for nested containers
    - Verifies each node correctly references its immediate parent

## Helper Functions Created

```rust
fn make_subgraph_node(
    id: &str,
    x: f64, y: f64,
    width: f64, height: f64,
    locked: bool,
    collapsed: Option<bool>,
    parent: Option<NodeId>,
) -> (NodeId, Node)

fn make_child_node(
    id: &str,
    x: f64, y: f64,
    width: f64, height: f64,
    locked: bool,
    parent: Option<NodeId>,
) -> (NodeId, Node)
```

## Code Quality

- All tests follow the `given_X_when_Y_then_Z` naming convention
- No `unwrap_used`, `expect_used`, or `panic` (per existing lint rules)
- No unsafe code
- Tests use existing model structures and functions
