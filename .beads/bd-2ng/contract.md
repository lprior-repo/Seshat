bead_id: bd-2ng
bead_title: toolbar: Node alignment tools for selected nodes
phase: p0
updated_at: 2026-03-01T00:00:00Z

# Contract: Node Alignment Tools

## Summary
Add alignment buttons (Align Left, Align Center, Align Right, Align Top, Align Middle, Align Bottom) to the toolbar that operate on selected nodes. Buttons should only appear when 2+ nodes are selected.

## Preconditions
1. At least two nodes are selected in `doc.editor_state.selected_items`
2. Selected nodes have valid positions (x, y are finite)
3. Selected nodes are not locked (unless they are Subgraphs)

## Postconditions
1. All selected nodes share the alignment coordinate (left edge x, center x, right edge x, top edge y, center y, or bottom edge y)
2. Relative ordering of nodes is preserved (z_index unchanged)
3. Document revision is incremented
4. Operation is added to undo history
5. Node sizes are unchanged (invariant)

## Alignment Operations

### Horizontal Alignments
- **Align Left**: Set all selected nodes' x to `min(node.x for node in selected)`
- **Align Center Horizontal**: Set all selected nodes' x to `min_x + (max_right - min_x) / 2 - node.width / 2`
  - Where `min_x = min(node.x)`, `max_right = max(node.x + node.width)`
- **Align Right**: Set all selected nodes' x to `max(node.x + node.width for node in selected) - node.width`

### Vertical Alignments
- **Align Top**: Set all selected nodes' y to `min(node.y for node in selected)`
- **Align Middle Vertical**: Set all selected nodes' y to `min_y + (max_bottom - min_y) / 2 - node.height / 2`
  - Where `min_y = min(node.y)`, `max_bottom = max(node.y + node.height)`
- **Align Bottom**: Set all selected nodes' y to `max(node.y + node.height for node in selected) - node.height`

## API Design

### commands.rs
```rust
pub enum AlignmentAxis {
    Horizontal,
    Vertical,
}

pub enum AlignmentMode {
    Start,   // Left/Top
    Center,  // Center/Middle
    End,     // Right/Bottom
}

pub fn apply_align_selection(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    axis: AlignmentAxis,
    mode: AlignmentMode,
) -> bool;
```

### toolbar/actions.rs
```rust
pub fn align_left(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>);
pub fn align_center_horizontal(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>);
pub fn align_right(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>);
pub fn align_top(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>);
pub fn align_middle_vertical(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>);
pub fn align_bottom(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>);
```

## UI Requirements

1. Alignment buttons appear in toolbar between z-order buttons (Back/Forward/To Back/To Front) and Validate button
2. Buttons are disabled when `stats.selected_count < 2`
3. Each button has a `data-testid` attribute:
   - `toolbar-align-left`
   - `toolbar-align-center-h`
   - `toolbar-align-right`
   - `toolbar-align-top`
   - `toolbar-align-middle-v`
   - `toolbar-align-bottom`
4. Button labels:
   - "Left" (horizontal align to left edge)
   - "H-Center" (horizontal center)
   - "Right" (horizontal align to right edge)
   - "Top" (vertical align to top edge)
   - "V-Center" (vertical center)
   - "Bottom" (vertical align to bottom edge)

## Test Requirements

### Unit Tests (commands.rs)
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

### Property Tests
1. Alignment always produces finite coordinates
2. Node dimensions are never changed by alignment
3. Revision is always incremented on successful alignment

## Error Handling
- Return `false` if fewer than 2 nodes are selected
- Return `false` if any selected node has non-finite coordinates
- Skip locked nodes (but proceed if at least 2 non-locked nodes selected)
- Preserve undo history on failure (no push)

## Invariants
1. Node width and height are never modified
2. Z-order is preserved
3. Parent relationships are preserved
4. Only non-locked nodes (or Subgraphs) are moved
