bead_id: bd-321
bead_title: tests: Implement SUB subgraph tests - drag interactions
phase: p0
updated_at: 2026-03-01T00:40:00Z

# Contract: SUB Subgraph Drag Interaction Tests (bd-321)

## Summary

Add 5 subgraph tests focused on drag interaction behaviors within the `interaction_reducer.rs` test module (`subgraph_tests`).

## Scope

Location: `diagram_tool/src/ui/canvas/interaction_reducer.rs` within the existing `subgraph_tests` module.

## Test Specifications

### TEST-321-1: Drag Multiple Selected Nodes Into Container

**Given**: A document with:
- An unlocked container (subgraph) node at position (300, 100) with size 200x150
- Two unlocked child nodes at positions (50, 100) and (50, 150), both with no parent

**When**: Both child nodes are selected and dragged to positions inside the container bounds (e.g., (320, 120) and (320, 170))

**Then**:
- `drag_original_positions` returns positions for both selected nodes
- The dragged positions are calculated correctly relative to the drag delta
- The container exists and has proper bounds

### TEST-321-2: Drag Container Into Another Container (Nesting)

**Given**: A document with:
- An outer container at position (100, 100) with size 400x300
- An inner container at position (500, 100) with size 150x100, with no parent

**When**: The inner container is dragged to position (150, 150), which is inside the outer container

**Then**:
- Both containers exist in the document
- The inner container can be positioned within the outer container bounds
- The geometry supports valid nesting (inner fits within outer)

### TEST-321-3: Grab Parent Prevents Reparent Gesture

**Given**: A document with:
- An outer container at position (100, 100) with size 400x300
- An inner container at position (150, 150) with size 200x150, with parent = outer
- A child node at position (180, 180) with parent = inner

**When**: The inner container (which has a parent) is selected

**Then**:
- When calculating `drag_original_positions` for the inner container selection:
  - The inner container is included
  - The child node (descendant) is also included
  - The parent chain (outer -> inner -> child) is correctly established

### TEST-321-4: Container Auto-Expand When Child Crosses Boundary

**Given**: A document with:
- A container at position (100, 100) with size 200x150
- A child node at position (120, 120) with size 50x30, parent = container
- The child is selected

**When**: Calculating `drag_original_positions` for the selection

**Then**:
- Both the container and child are tracked for potential resize operations
- The child's initial position is recorded
- The container bounds are known for boundary calculations

### TEST-321-5: Drag Selection With Nested Descendants

**Given**: A document with:
- An outer container (no parent)
- An inner container (parent = outer)
- A leaf node (parent = inner)

**When**: The outer container is selected

**Then**:
- `drag_original_positions` returns positions for all three nodes (outer, inner, leaf)
- The descendant traversal correctly captures the full hierarchy
- All positions are recorded for the drag operation

## Acceptance Criteria

1. All 5 tests must be added to the `subgraph_tests` module in `interaction_reducer.rs`
2. Each test must use the existing helper functions `make_subgraph_node` and `make_child_node`
3. Tests must verify behaviors related to drag operations using `drag_original_positions` and related functions
4. All tests must pass with `moon run :test`
5. Code must comply with existing lint rules (`#![deny(clippy::unwrap_used)]`, etc.)

## Out of Scope

- Actual reparenting logic (parent field mutation during drag)
- Visual/UI rendering tests
- E2E tests
- Auto-expand animation behavior

## Dependencies

- Existing `subgraph_tests` module infrastructure
- `drag_original_positions` function from `crate::ui::interaction`
- `resize_target_ids` function from the parent module
