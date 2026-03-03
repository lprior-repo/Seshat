# Contract Specification: bd-2qs - Selection Reliability

## Metadata
- **Bead ID**: bd-2qs
- **Title**: Fix selection reliability (SEL-001 to SEL-025 test cases)
- **Priority**: P1
- **Type**: feature
- **Created**: 2026-03-03

## Overview

This contract specifies the requirements for implementing all 25 selection test cases (SEL-001 to SEL-025) as defined in the architecture specification. Selection is a fundamental interaction pattern that must work reliably across all scenarios.

## Test Case Specifications

### SEL-001: Click Selects Node
**Given**: A diagram with nodes
**When**: User clicks on a node
**Then**: The clicked node is added to the selection set
**Contract**:
- Precondition: Node ID exists in document
- Postcondition: `selected_items.contains(node_id)`
- Invariant: Previous single selection replaced (not additive)

### SEL-002: Shift-Click Toggles Selection
**Given**: Node A is selected
**When**: User shift-clicks on node B
**Then**: Both A and B are selected
**Contract**:
- Precondition: Shift modifier is held
- Postcondition: `selected_items.len() == 2`
- Invariant: Clicking selected node with Shift removes it from selection

### SEL-003: Marquee Selects Contained Nodes
**Given**: Empty selection
**When**: User drags a rectangle on canvas
**Then**: All nodes fully inside the rectangle are selected
**Contract**:
- Precondition: Drag starts on empty canvas area
- Postcondition: `selected_items` contains all nodes where `node.bounds inside marquee`
- Invariant: Left-to-right drag = containment mode

### SEL-004: Click Empty Clears Selection
**Given**: One or more nodes are selected
**When**: User clicks on empty canvas area
**Then**: Selection set becomes empty
**Contract**:
- Precondition: Click is on empty area (no node hit)
- Postcondition: `selected_items.is_empty()`
- Invariant: No modifiers affect this behavior

### SEL-005: Marquee Direction Switches Mode
**Given**: Empty selection
**When**: User drags right-to-left vs left-to-right
**Then**: Intersect mode vs contain mode
**Contract**:
- Right-to-left (negative width): Intersection mode - partial overlap selects
- Left-to-right (positive width): Containment mode - full inclusion required
- Postcondition: Correct nodes selected based on mode

### SEL-006: Hover Shows Visual Affordances
**Given**: A node exists on canvas
**When**: User hovers over the node (without clicking)
**Then**: Visual feedback indicates interactivity
**Contract**:
- Postcondition: Border style changes (color or width)
- Performance: Hover feedback within 16ms

### SEL-007: Resize Handles Are Clickable
**Given**: A node is selected
**When**: Selection bounding box shows resize handles
**Then**: Each handle initiates resize when dragged
**Contract**:
- Precondition: Single node or multi-node selection
- Postcondition: Dragging handle resizes selection
- Handles: 8 corners/edges (NW, N, NE, E, SE, S, SW, W)

### SEL-008: Touch Has Larger Hit Area
**Given**: Touch input device
**When**: User taps near (but not exactly on) a node
**Then**: Node is selected with more forgiving hit area
**Contract**:
- Touch hit area: Extended by 8-12px beyond visual bounds
- Mouse hit area: Exact visual bounds

### SEL-009: Drag Threshold Prevents Accidental Drag
**Given**: A node is selected
**When**: User clicks and drags less than 3px
**Then**: No move operation occurs
**Contract**:
- Threshold: 3 pixels minimum movement
- Postcondition: Node position unchanged if under threshold

### SEL-010: Right-Click Context Menu Preserves Selection
**Given**: A node is selected
**When**: User right-clicks on the selected node
**Then**: Selection is preserved, context menu appears
**Contract**:
- Precondition: Right-click on selected item
- Postcondition: `selected_items` unchanged

### SEL-011: Alt-Click Selects Parent Container
**Given**: A node is inside a subgraph/container
**When**: User alt-clicks on the child node
**Then**: Parent container is selected instead
**Contract**:
- Precondition: Node has parent reference
- Postcondition: `selected_items` contains parent ID, not child ID

### SEL-012: Locked Element Not Selectable
**Given**: A node has `locked: true`
**When**: User clicks on the locked node
**Then**: Node is not added to selection
**Contract**:
- Precondition: `node.locked == true`
- Postcondition: Selection unchanged (locked nodes cannot be selected)

### SEL-013: Hidden Element Not Hit-Testable
**Given**: A node has `visibility: hidden` or `display: none`
**When**: User clicks where hidden node would be
**Then**: Hidden node is not selected (click passes through)
**Contract**:
- Hidden nodes excluded from hit testing
- Click selects node underneath if present

### SEL-014: Right-Click on Unselected Node Selects It First
**Given**: Node A is selected, Node B is not
**When**: User right-clicks on Node B
**Then**: Node B becomes selected (replacing selection of A)
**Contract**:
- Postcondition: `selected_items == {B}`
- Context menu still appears

### SEL-015: Edge/Connector Selection
**Given**: An edge connects two nodes
**When**: User clicks on the edge line
**Then**: Edge is selected (not the nodes)
**Contract**:
- Precondition: Edge ID exists in document
- Postcondition: `selected_items.contains(edge_id)`
- Edge hit area: 6px tolerance from line

### SEL-016: Select All With Keyboard
**Given**: A diagram with multiple nodes
**When**: User presses Ctrl/Cmd+A
**Then**: All nodes are selected
**Contract**:
- Postcondition: `selected_items == all_node_ids`
- Scope: Current viewport or entire document

### SEL-017: Selection Z-Order/Stacking
**Given**: Two overlapping nodes, A on top of B
**When**: User clicks on the overlap area
**Then**: Top-most node (A) is selected
**Contract**:
- Selection respects z_index (higher = on top)
- Tie-breaker: Later creation order

### SEL-018: Selection With Overlapping Nodes
**Given**: Multiple overlapping nodes
**When**: User clicks on overlap area
**Then**: Only top-most node is selected (not all)
**Contract**:
- Single click = single selection
- Shift-click on overlap = add top-most to selection

### SEL-019: Selection of Grouped Items
**Given**: A group/subgraph containing child nodes
**When**: User clicks on a child node
**Then**: Child is selected (not the group)
**Contract**:
- Direct selection of children allowed
- Alt-click selects parent group

### SEL-020: Selection Visual Feedback
**Given**: One or more nodes are selected
**When**: Selection state is active
**Then**: Visual feedback is shown
**Contract**:
- Bounding box around selection
- Resize handles at corners/edges
- Selection color highlight on nodes
- Performance: Visual update within 16ms

### SEL-021: Selection Bounding Box Matches Node Geometry
**Given**: A node is selected
**When**: Selection bounding box is rendered
**Then**: Box exactly matches node bounds
**Contract**:
- Postcondition: `selection_bounds == node.bounds`
- Handles positioned at corners

### SEL-022: Long Press Selects Without Drag
**Given**: A node exists
**When**: User presses and holds without moving
**Then**: Node is selected (no drag initiated)
**Contract**:
- Hold duration threshold: 100ms
- No movement beyond drag threshold

### SEL-023: Double-Click Enters Edit Mode
**Given**: A node is selected or unselected
**When**: User double-clicks on it
**Then**: Node enters text edit mode
**Contract**:
- Timing: Two clicks within 500ms
- Postcondition: Text input focused on node label

### SEL-024: Selection Persists After Zoom
**Given**: A node is selected
**When**: User zooms in or out
**Then**: Selection is preserved
**Contract**:
- Postcondition: `selected_items` unchanged
- Selection handles update position

### SEL-025: Box-Select Through Parent Boundaries
**Given**: Nodes inside and outside a subgraph
**When**: User draws marquee across subgraph boundary
**Then**: All nodes inside marquee are selected (regardless of parent)
**Contract**:
- Selection ignores parent boundaries
- Cross-boundary selection works

## Error Handling

| Error | Condition | Response |
|-------|-----------|----------|
| `SelectionError::NodeNotFound` | Select non-existent node | Return error, no selection change |
| `SelectionError::EdgeNotFound` | Select non-existent edge | Return error, no selection change |
| `SelectionError::EmptySelection` | Operation requires selection | Show user message |

## Performance Requirements

| Operation | Max Latency |
|-----------|-------------|
| Single click to selection | 16ms |
| Marquee selection (500 nodes) | 100ms |
| Selection visual update | 16ms |
| Select all (3000 nodes) | 100ms |

## Invariants

1. **I1**: Selection never contains deleted IDs (cleanup on delete)
2. **I2**: Selection is consistent across undo/redo
3. **I3**: Single-select replaces previous selection
4. **I4**: Selection set is always a subset of document IDs
5. **I5**: Selection persists across viewport changes

## Verification Criteria

All 25 SEL test cases must pass:
- [ ] SEL-001: Click selects node
- [ ] SEL-002: Shift-click toggles
- [ ] SEL-003: Marquee selects contained
- [ ] SEL-004: Click empty clears
- [ ] SEL-005: Marquee direction switches mode
- [ ] SEL-006: Hover shows visual affordances
- [ ] SEL-007: Resize handles are clickable
- [ ] SEL-008: Touch has larger hit area
- [ ] SEL-009: Drag threshold prevents accidental drag
- [ ] SEL-010: Right-click context menu preserves selection
- [ ] SEL-011: Alt-click selects parent container
- [ ] SEL-012: Locked element not selectable
- [ ] SEL-013: Hidden element not hit-testable
- [ ] SEL-014: Right-click on unselected node selects it first
- [ ] SEL-015: Edge/connector selection
- [ ] SEL-016: Select all with keyboard
- [ ] SEL-017: Selection z-order/stacking
- [ ] SEL-018: Selection with overlapping nodes
- [ ] SEL-019: Selection of grouped items
- [ ] SEL-020: Selection visual feedback
- [ ] SEL-021: Selection bounding box matches node geometry
- [ ] SEL-022: Long press selects without drag
- [ ] SEL-023: Double-click enters edit mode
- [ ] SEL-024: Selection persists after zoom
- [ ] SEL-025: Box-select through parent boundaries
