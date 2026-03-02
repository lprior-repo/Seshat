bead_id: bd-51b
bead_title: toolbar: Node distribution tools for even spacing
phase: p1
updated_at: 2026-03-01T01:45:00Z

# Implementation: Node Distribution Tools

## Summary

Implemented horizontal and vertical distribution tools for evenly spacing selected nodes in the diagram toolbar.

## Changes Made

### 1. Added DistributionAxis Enum (`/home/lewis/src/bd-51b/diagram_tool/src/ui/commands.rs`)

```rust
/// Axis for distribution operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistributionAxis {
    Horizontal,
    Vertical,
}
```

### 2. Added apply_distribute_selection Function (`/home/lewis/src/bd-51b/diagram_tool/src/ui/commands.rs`)

Core distribution function that:
- Requires 3+ selected nodes (vs 2+ for alignment)
- Calculates equal spacing between nodes
- Preserves outermost node positions
- Handles locked nodes (skips them)
- Supports undo via history

### 3. Added Toolbar Actions (`/home/lewis/src/bd-51b/diagram_tool/src/ui/toolbar/actions.rs`)

```rust
pub fn distribute_horizontal(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_distribute_selection(doc_signal, history_signal, DistributionAxis::Horizontal);
}

pub fn distribute_vertical(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_distribute_selection(doc_signal, history_signal, DistributionAxis::Vertical);
}
```

### 4. Added Toolbar Buttons (`/home/lewis/src/bd-51b/diagram_tool/src/ui/toolbar.rs`)

- "Dist H" button: Distributes nodes horizontally
- "Dist V" button: Distributes nodes vertically
- Buttons are disabled when fewer than 3 nodes are selected
- Located after alignment buttons in the toolbar

### 5. Added Comprehensive Tests

10 unit tests covering:
- Basic horizontal/vertical distribution
- Y preservation during horizontal distribution
- X preservation during vertical distribution
- Less than 3 nodes returns false
- Outermost nodes stay at bounds
- Equal spacing verification
- Node size preservation
- Locked nodes handling
- Revision increment

## Algorithm

For N nodes distributed between bounds [min, max]:

1. Sort nodes by position (X for horizontal, Y for vertical)
2. Keep first and last nodes at their original positions (boundaries)
3. Calculate total available space between boundaries
4. Calculate equal spacing: `spacing = (max_bound - min_bound - sum_of_node_sizes) / (N - 1)`
5. Position interior nodes sequentially

## Test Results

```
running 10 tests
test ui::commands::distribution_tests::test_distribute_less_than_three_nodes_returns_false ... ok
test ui::commands::distribution_tests::test_distribute_updates_revision ... ok
test ui::commands::distribution_tests::test_distribute_outermost_nodes_at_bounds ... ok
test ui::commands::distribution_tests::test_distribute_vertical_preserves_x ... ok
test ui::commands::distribution_tests::test_distribute_vertical_three_nodes ... ok
test ui::commands::distribution_tests::test_distribute_horizontal_three_nodes ... ok
test ui::commands::distribution_tests::test_distribute_horizontal_preserves_y ... ok
test ui::commands::distribution_tests::test_distribute_locked_nodes_skipped ... ok
test ui::commands::distribution_tests::test_distribute_preserves_node_size ... ok
test ui::commands::distribution_tests::test_distribute_equal_spacing ... ok

test result: ok. 10 passed; 0 failed
```

Full test suite: 970 tests passed (957 unit + 13 e2e)

## Files Modified

- `/home/lewis/src/bd-51b/diagram_tool/src/ui/commands.rs` - Added `DistributionAxis` enum and `apply_distribute_selection` function
- `/home/lewis/src/bd-51b/diagram_tool/src/ui/toolbar.rs` - Added distribution buttons
- `/home/lewis/src/bd-51b/diagram_tool/src/ui/toolbar/actions.rs` - Added `distribute_horizontal` and `distribute_vertical` functions
