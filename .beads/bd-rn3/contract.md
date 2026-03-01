bead_id: bd-rn3
bead_title: tests: Implement SEL selection tests 4/5
phase: p0
updated_at: 2026-03-01T22:04:00Z

# Contract: SEL Selection Tests (bd-rn3)

## Scope

Implement 5 selection tests for the diagram tool, covering multi-type selection,
selection persistence across view transformations, undo/redo behavior, and edit mode entry.

## Test Location

Tests shall be added to `diagram_tool/src/ui/canvas/selection_geometry.rs` in the `#[cfg(test)] mod tests` block,
or in `diagram_tool/src/ui/canvas/interaction_reducer.rs` as appropriate for the interaction behavior being tested.

## Required Tests

### SEL-001: Multi-type selection (shape+text+connector)

**Given:** A document containing:
- A node (shape) with id "shape_node"
- A text element (NodeKind::Text) with id "text_node"
- An edge (connector) with id "connector_edge" connecting the two

**When:** All three items are added to `editor_state.selected_items`

**Then:**
- `selected_node_ids()` returns both node IDs (shape and text)
- `selection_bounds()` returns a bounding box that encompasses all selected nodes
- The bounds correctly account for the positions and dimensions of both nodes

### SEL-002: Selection persists across pan/zoom

**Given:** A document with selected items in `editor_state.selected_items`

**When:** Camera transform changes:
- `camera_x` changes from 0.0 to 100.0
- `camera_y` changes from 0.0 to 50.0
- `zoom` changes from 1.0 to 2.0

**Then:**
- `selected_items` set remains unchanged
- `selected_node_ids()` returns the same IDs after transform
- `selection_bounds()` returns the same document-space bounds (not screen-space)

### SEL-003: Selection box after undo/redo

**Given:**
- A document with nodes "n1" and "n2"
- An initial empty selection
- User selects "n1" (pushing history)
- User selects "n2" (pushing history)

**When:**
- Undo is called (should restore selection to just "n1")
- Redo is called (should restore selection to "n2")

**Then:**
- After undo: `selected_items` contains only "n1"
- After redo: `selected_items` contains only "n2"
- History correctly restores `editor_state.selected_items` state

### SEL-004: Selection box handles negative coordinates

**Given:** A document with nodes at negative coordinates:
- Node "neg_x" at x=-100, y=50
- Node "neg_y" at x=50, y=-100
- Node "neg_both" at x=-200, y=-200

**When:** All three nodes are selected

**Then:**
- `selection_bounds()` returns correct min/max values
- Bounds correctly calculate: min_x = -200, min_y = -200
- Width and height are positive values representing the span

### SEL-005: Double-click enters edit mode

**Given:** A document with a node "editable" that is currently selected

**When:** Double-click interaction is initiated on the node (simulated via interaction mode)

**Then:**
- The interaction system transitions to edit mode
- `editing_node` signal is set to Some(node_id)
- The node's label becomes editable

Note: This test validates the interaction reducer behavior for edit mode entry.

## Implementation Requirements

1. All tests must use `#[test]` attribute
2. Tests must follow existing patterns in the codebase (see `interaction_reducer.rs` tests)
3. Use `#[allow(clippy::unwrap_used, clippy::expect_used)]` for test code
4. Helper functions may be created to reduce boilerplate
5. Tests must be deterministic and not rely on external state

## Acceptance Criteria

- [ ] All 5 tests pass with `cargo test`
- [ ] Tests follow naming convention `given_<precondition>_when_<action>_then_<outcome>`
- [ ] No clippy warnings in test code
- [ ] Tests are in the appropriate module (selection_geometry or interaction_reducer)

## Dependencies

- `diagram_tool/src/ui/canvas/selection_geometry.rs`
- `diagram_tool/src/ui/canvas/interaction_reducer.rs`
- `diagram_tool/src/models/document.rs` (DiagramDocument, EditorState)
- `diagram_tool/src/history.rs` (History)
