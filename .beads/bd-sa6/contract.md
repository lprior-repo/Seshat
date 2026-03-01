bead_id: bd-sa6
bead_title: tests: Implement SUB subgraph tests 4/4
phase: p0
updated_at: 2026-03-01T22:55:00Z

# Contract: SUB Subgraph Interaction Tests (4/4)

## Summary

Implement 5 subgraph/container interaction tests covering click-through selection, box-select across containers, collapse/expand behavior, and locked container interactions.

## Scope

These tests validate the SUB (subgraph/container) interaction behaviors in the canvas layer, specifically in the `interaction_reducer.rs` and related selection geometry modules.

## Test Cases

### SUB-001: Click inside container selects child vs container with modifier

**Given**: A container (subgraph) node with a child node inside it
**When**: User clicks on the child node area
**Then**:
- Without modifier key: the child node is selected
- With Shift/Ctrl modifier: selection behavior respects multi-select semantics

**Implementation Notes**:
- Test should use z_index ordering (containers have z_index=-1, children have z_index=1000)
- Verify hit testing prioritizes higher z_index nodes

### SUB-002: Box-select across container boundary

**Given**: A container node and nodes both inside and outside the container
**When**: User performs a rubber-band/box selection that crosses the container boundary
**Then**:
- All nodes within the selection rectangle are selected regardless of container membership
- The selection includes nodes inside the container and outside the container

**Implementation Notes**:
- Use `InteractionMode::RubberBand` to simulate box selection
- Verify `selected_node_ids` returns both container children and external nodes

### SUB-003: Collapse/expand container behavior

**Given**: A container (subgraph) node with `collapsed` field
**When**: User toggles the collapsed state
**Then**:
- When collapsed=true: container hides children visually (z_index/visibility)
- When collapsed=false: container shows children normally

**Implementation Notes**:
- Test the `collapsed: Option<bool>` field on the Node struct
- Verify round-trip preservation of collapsed state

### SUB-004: Locked container with unlocked children interactions

**Given**: A container node with `locked=true` containing child nodes with `locked=false`
**When**: User attempts to interact with the children
**Then**:
- Child nodes remain selectable and movable despite parent being locked
- Container's locked state does not propagate to children

**Implementation Notes**:
- Each node has its own `locked: bool` field
- Verify locked state is per-node, not inherited

### SUB-005: Parent-child relationship preservation during selection

**Given**: A container with multiple children
**When**: User selects and moves the container
**Then**:
- Children maintain their parent reference
- Children's relative positions are preserved
- The `parent: Option<NodeId>` field remains intact

**Implementation Notes**:
- Verify `within()` function correctly identifies children
- Test that `resize_target_ids` includes children when container is selected

## Acceptance Criteria

1. All 5 tests must pass without `unwrap_used`, `expect_used`, or `panic` (per existing lint rules)
2. Tests must use the existing `#![forbid(unsafe_code)]` policy
3. Tests should follow the naming convention `given_X_when_Y_then_Z`
4. Tests should be added to `diagram_tool/src/ui/canvas/interaction_reducer.rs` in a new `mod subgraph_tests` block
5. All tests must compile and pass with `moon run :test`

## Pre-conditions

- The `Node` struct supports `parent`, `locked`, `collapsed`, and `z_index` fields
- The `within()` function exists for hit testing containment
- The `resize_target_ids()` function exists for including children in resize operations
- `InteractionMode::RubberBand` exists for box selection simulation

## Post-conditions

- All 5 tests exist and pass
- Code coverage increases for subgraph interaction paths
- No regression in existing tests

## Out of Scope

- Visual/rendering tests
- E2E tests requiring full UI simulation
- Persistence/serialization tests (covered by existing `subgraph_persistence_tests.rs`)
