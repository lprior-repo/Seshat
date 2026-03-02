bead_id: bd-51b
bead_title: toolbar: Node distribution tools for even spacing
phase: p0
updated_at: 2026-03-01T01:12:00Z

# Contract: Node Distribution Tools

## Overview

Implement horizontal and vertical distribution tools for evenly spacing selected nodes.

## Preconditions

1. At least **three nodes** must be selected for distribution buttons to be visible
2. Selected nodes must have valid (finite) positions
3. Distribution requires at least 3 nodes to be meaningful (unlike alignment which needs 2)

## Postconditions

### Happy Path (3+ nodes selected)

1. **Horizontal Distribution**: Gaps between distributed nodes are equal horizontally
   - Outermost nodes (leftmost and rightmost) remain at original X bounds
   - Interior nodes are repositioned to create equal spacing

2. **Vertical Distribution**: Gaps between distributed nodes are equal vertically
   - Outermost nodes (topmost and bottommost) remain at original Y bounds
   - Interior nodes are repositioned to create equal spacing

### Error Paths

1. Fewer than 3 nodes selected: Distribution buttons are not visible (no-op)
2. Invalid node positions: Operation returns false without modifying document

## Invariants

1. **Distribution does not change node size** - width and height are preserved
2. **Distribution operations are undoable** - history is updated before modification
3. **Locked nodes are skipped** - unless they are Subgraphs
4. **Z-order is preserved** - no reordering of nodes occurs
5. **Node Y positions unchanged during horizontal distribution**
6. **Node X positions unchanged during vertical distribution**

## Algorithm

### Distribution Algorithm

For N nodes distributed between bounds [min, max]:

1. Sort nodes by position (X for horizontal, Y for vertical)
2. Keep first and last nodes at their original positions (boundaries)
3. Calculate total available space between boundaries
4. Calculate equal spacing: `spacing = (max_bound - min_bound - sum_of_node_sizes) / (N - 1)`
5. Position interior nodes: `position[i] = position[i-1] + size[i-1] + spacing`

## Required Functions

### Core Functions

```rust
/// Axis for distribution operations
pub enum DistributionAxis {
    Horizontal,
    Vertical,
}

/// Distribute selected nodes evenly along the specified axis
pub fn apply_distribute_selection(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    axis: DistributionAxis,
) -> bool;
```

### Toolbar Actions

```rust
pub fn distribute_horizontal(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool;

pub fn distribute_vertical(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool;
```

## UI Requirements

1. Add "Distribute H" and "Distribute V" buttons to toolbar
2. Buttons are **hidden/disabled** when fewer than 3 nodes are selected
3. Buttons appear in a logical grouping with other alignment/distribution tools
4. Buttons follow existing toolbar styling patterns

## Test Requirements

### Unit Tests

1. `test_distribute_horizontal_three_nodes` - Basic horizontal distribution
2. `test_distribute_vertical_three_nodes` - Basic vertical distribution
3. `test_distribute_horizontal_preserves_y` - Y positions unchanged
4. `test_distribute_vertical_preserves_x` - X positions unchanged
5. `test_distribute_less_than_three_nodes_returns_false` - Precondition check
6. `test_distribute_outermost_nodes_at_bounds` - Boundary preservation
7. `test_distribute_equal_spacing` - Verify equal gaps
8. `test_distribute_locked_nodes_skipped` - Lock handling
9. `test_distribute_updates_revision` - Revision increment
10. `test_distribute_pushes_history` - Undo support

### Property Tests

1. Distribution always produces finite coordinates
2. Distribution never changes node dimensions
3. Distribution maintains z-order

## Related Files

- `/home/lewis/src/seshat/diagram_tool/src/ui/commands.rs` - Add distribution functions
- `/home/lewis/src/seshat/diagram_tool/src/ui/toolbar.rs` - Add buttons
- `/home/lewis/src/seshat/diagram_tool/src/ui/toolbar/actions.rs` - Add action handlers

## Acceptance Criteria

- [ ] `apply_distribute_selection` function implemented in `commands.rs`
- [ ] Distribution buttons visible only when 3+ nodes selected
- [ ] Horizontal distribution creates equal X spacing
- [ ] Vertical distribution creates equal Y spacing
- [ ] All tests pass
- [ ] `moon run :ci` passes
