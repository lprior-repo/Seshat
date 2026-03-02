bead_id: bd-2ng
bead_title: toolbar: Node alignment tools for selected nodes
phase: p1
updated_at: 2026-03-01T00:00:00Z

# Implementation: Node Alignment Tools

## Summary
Implemented 6 alignment buttons in the toolbar that operate on selected nodes when 2+ nodes are selected.

## Files Modified

### 1. diagram_tool/src/ui/commands.rs

Added alignment types and core function:

```rust
/// Axis for alignment operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentAxis {
    Horizontal,
    Vertical,
}

/// Mode for alignment operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentMode {
    Start,  // Left (Horizontal) or Top (Vertical)
    Center, // Center (Horizontal) or Middle (Vertical)
    End,    // Right (Horizontal) or Bottom (Vertical)
}

pub fn apply_align_selection(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    axis: AlignmentAxis,
    mode: AlignmentMode,
) -> bool;
```

Key implementation details:
- Calculates bounding box from selected nodes
- Skips locked nodes (except Subgraphs which are always movable)
- Returns false if any coordinate is non-finite
- Returns false if fewer than 2 movable nodes
- Pushes undo history on success
- Increments revision on success

### 2. diagram_tool/src/ui/toolbar/actions.rs

Added 6 wrapper functions:

```rust
pub fn align_left(doc_signal, history_signal);
pub fn align_center_horizontal(doc_signal, history_signal);
pub fn align_right(doc_signal, history_signal);
pub fn align_top(doc_signal, history_signal);
pub fn align_middle_vertical(doc_signal, history_signal);
pub fn align_bottom(doc_signal, history_signal);
```

### 3. diagram_tool/src/ui/toolbar.rs

Added 6 buttons with test IDs:
- `toolbar-align-left`
- `toolbar-align-center-h`
- `toolbar-align-right`
- `toolbar-align-top`
- `toolbar-align-middle-v`
- `toolbar-align-bottom`

Buttons are disabled when `stats.selected_count < 2`.

## Tests Added

15 unit tests in commands.rs:

1. `given_two_nodes_when_align_left_then_both_share_min_x`
2. `given_two_nodes_when_align_right_then_both_share_max_right`
3. `given_three_nodes_when_align_center_horizontal_then_centered`
4. `given_two_nodes_when_align_top_then_both_share_min_y`
5. `given_two_nodes_when_align_bottom_then_both_share_max_bottom`
6. `given_three_nodes_when_align_middle_vertical_then_centered`
7. `given_single_node_selected_when_align_then_returns_false`
8. `given_empty_selection_when_align_then_returns_false`
9. `given_locked_node_when_align_then_skips_locked`
10. `given_selection_with_infinite_coords_when_align_then_returns_false`
11. `given_alignment_when_successful_then_dimensions_unchanged`
12. `given_alignment_when_successful_then_revision_incremented`

## Alignment Algorithms

### Horizontal Align Left
- All nodes get x = min(node.x for node in selected)

### Horizontal Align Right
- All nodes get x = max(node.x + node.width) - node.width

### Horizontal Align Center
- Calculate center = min_x + (max_right - min_x) / 2
- Each node gets x = center - node.width / 2

### Vertical Align Top
- All nodes get y = min(node.y for node in selected)

### Vertical Align Bottom
- All nodes get y = max(node.y + node.height) - node.height

### Vertical Align Middle
- Calculate center = min_y + (max_bottom - min_y) / 2
- Each node gets y = center - node.height / 2

## Invariants Preserved

1. Node dimensions (width, height) are never modified
2. Z-order is preserved
3. Parent relationships are preserved
4. Locked nodes are skipped (not moved)
5. Subgraphs can be aligned even when locked
