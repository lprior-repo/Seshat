bead_id: bd-321
bead_title: tests: Implement SUB subgraph tests - drag interactions
phase: p1
updated_at: 2026-03-01T00:50:00Z

# Implementation: SUB Subgraph Drag Interaction Tests (bd-321)

## Summary

Added 5 new tests to the `subgraph_tests` module in `diagram_tool/src/ui/canvas/interaction_reducer.rs` focusing on drag interaction behaviors with subgraph containers.

## Files Modified

### `/home/lewis/src/seshat/diagram_tool/src/ui/canvas/interaction_reducer.rs`

Added 5 tests at the end of the `subgraph_tests` module:

### TEST-321-1: `given_multiple_selected_nodes_when_drag_position_calculated_then_all_tracked`

Tests that when multiple nodes are selected outside a container, `drag_original_positions` correctly tracks all selected nodes for the drag operation.

**Key assertions:**
- Both selected nodes have positions recorded
- Position values match initial placement
- Uses `drag_original_positions` from `crate::ui::interaction`

### TEST-321-2: `given_two_containers_when_inner_positioned_in_outer_then_geometry_supports_nesting`

Tests that geometry validation supports container nesting when one container fits within another.

**Key assertions:**
- Inner container fits within outer container bounds (uses `within` helper)
- Both containers exist in the document
- Inner starts without parent (would be assigned on drop)

### TEST-321-3: `given_nested_container_with_children_when_middle_selected_then_descendants_included`

Tests the "grab parent prevents reparent" scenario - when a container that has children is selected, its descendants are included in drag positions.

**Key assertions:**
- Selected container is in drag positions
- Child of selected container is included (descendant traversal)
- Ancestor (outer) is NOT included when selecting inner

### TEST-321-4: `given_container_with_child_near_edge_when_resize_targets_then_both_included`

Tests container auto-expand boundary calculations - when a container with a child is selected, both are included in resize targets.

**Key assertions:**
- Container is in resize targets
- Child inside container is in resize targets
- Exactly 2 nodes in targets

### TEST-321-5: `given_three_level_hierarchy_when_outer_selected_then_all_descendants_in_drag_positions`

Tests drag selection with deeply nested descendants - when the outermost container is selected, all descendants at all levels are included.

**Key assertions:**
- All three nodes (outer, inner, leaf) are in drag positions
- Position values are correctly recorded for each level
- Tests the full descendant traversal logic

## Code Quality

- All tests follow existing patterns in `subgraph_tests` module
- Use existing helper functions `make_subgraph_node` and `make_child_node`
- Comply with lint rules (`#![deny(clippy::unwrap_used)]`, etc.)
- Use `expect` with descriptive messages for test assertions

## Test Execution

```
cargo test -p diagram_tool subgraph
```

Result: 43 tests passed (including 5 new tests)
