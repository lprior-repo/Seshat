# Contract Specification: SEL-021 to SEL-025 (Selection Edge Cases)

## Context
- Feature: Selection Edge Cases (SEL-021 to SEL-025)
- Domain terms:
  - Bounding Box: Visual rectangle enclosing selected nodes.
  - Long Press: Touch interaction without significant movement.
  - Edit Mode: State where a node's text label can be modified.
  - Marquee Selection: Box-select tool for capturing multiple nodes.
- Assumptions:
  - `DiagramDocument` manages the authoritative node state.
  - Interactions update `editor_state.selected_items` or `editor_state.edit_mode_target`.
- Open questions:
  - What is the exact long press duration threshold for touch? (Assuming 500ms)
  - What is the pixel distance threshold that invalidates a long press? (Assuming 5px)

## Preconditions
- [P1] For bounding box calculation (SEL-021), the node must exist in the document.
- [P2] For long press selection (SEL-022), the pointer movement distance must be less than the drag threshold.
- [P3] For double-click to edit (SEL-023), the targeted node must support text editing (not locked, not a generic untyped canvas).
- [P4] For selection persistence (SEL-024), the UI framework's rerender cycle must not drop IDs that exist in the document model.
- [P5] For cross-boundary selection (SEL-025), the marquee rectangle must have non-negative width and height.

## Postconditions
- [Q1] SEL-021: The computed `selection_bounds` accurately spans all selected nodes, including rotated geometry.
- [Q2] SEL-022: After a valid long press, the target node is added to `selected_items` without triggering a drag state.
- [Q3] SEL-023: After a valid double-click on a shape, `editor_state.edit_mode_target` is set to the shape's `NodeId`.
- [Q4] SEL-024: The set of `selected_items` remains strictly unchanged before and after camera translation/zoom or React state rerenders.
- [Q5] SEL-025: All nodes geometrically intersecting the marquee box are added to `selected_items`, regardless of parent/group hierarchy.

## Invariants
- [I1] The selection bounding box cannot have a negative width or height.
- [I2] A node cannot simultaneously be in a "dragging" state and a "long-press selection" state.
- [I3] Only one node can be in edit mode at any given time.
- [I4] `selected_items` contains only valid, existing `NodeId`s.

## Error Taxonomy
- `SelectionError::NodeNotFound` - Attempted to compute bounds or interact with a non-existent `NodeId`.
- `SelectionError::MovementExceededDragThreshold` - A long press interaction accumulated too much distance and was converted to a drag.
- `SelectionError::NodeNotEditable` - Double-clicked a node that cannot enter text edit mode (e.g., locked).
- `SelectionError::InvalidMarqueeBounds` - Provided marquee coordinates form a negative-area rectangle.

## Contract Signatures
- `fn compute_selection_bounds(doc: &DiagramDocument) -> Result<SelectionBounds, SelectionError>`
- `fn handle_long_press(doc: &mut DiagramDocument, target: NodeId, movement: f64) -> Result<(), SelectionError>`
- `fn handle_double_click(doc: &mut DiagramDocument, target: NodeId) -> Result<(), SelectionError>`
- `fn compute_marquee_selection(doc: &DiagramDocument, marquee: Rect) -> Result<HashSet<NodeId>, SelectionError>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Node exists | Error variant | `Result<SelectionBounds, SelectionError::NodeNotFound>` |
| P2: Movement < threshold | Error variant | `Result<(), SelectionError::MovementExceededDragThreshold>` |
| P3: Node editable | Compile-time | `EditableNodeId` or `Result<(), SelectionError::NodeNotEditable>` |
| P4: Persists on zoom | Debug-only | `debug_assert_eq!(selection_before, selection_after)` |
| P5: Valid marquee bounds | Compile-time | `ValidRect` struct ensuring `width >= 0.0 && height >= 0.0` |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES P1: `compute_selection_bounds(&doc_with_deleted_node_in_selection)` -- should produce `Err(SelectionError::NodeNotFound)`
- VIOLATES P2: `handle_long_press(&mut doc, id, 15.0)` (where threshold is 5.0) -- should produce `Err(SelectionError::MovementExceededDragThreshold)`
- VIOLATES P3: `handle_double_click(&mut doc, locked_node_id)` -- should produce `Err(SelectionError::NodeNotEditable)`
- VIOLATES P5: `compute_marquee_selection(&doc, Rect { width: -10.0, ... })` -- should produce `Err(SelectionError::InvalidMarqueeBounds)`
- VIOLATES Q1: `compute_selection_bounds` returns bounds smaller than a selected rotated node's geometry -- detected via test assertion matching calculated bounds against expected math.
- VIOLATES Q2: `handle_long_press` modifies node coordinates -- detected via state diff assertion.
- VIOLATES Q3: `handle_double_click` leaves `edit_mode_target` empty -- detected via test assertion.
- VIOLATES Q4: Zooming clears `selected_items` -- detected via state diff assertion.
- VIOLATES Q5: Marquee selection ignores a nested child -- detected via test assertion.

## Ownership Contracts (Rust-specific)
- Shared borrow: `compute_selection_bounds(doc: &DiagramDocument)` -- read-only, purely functional calculation of bounds.
- Shared borrow: `compute_marquee_selection(doc: &DiagramDocument, marquee: Rect)` -- read-only, purely functional intersection check.
- Exclusive borrow: `handle_long_press(doc: &mut DiagramDocument, ...)` -- mutates `doc.editor_state.selected_items`.
- Exclusive borrow: `handle_double_click(doc: &mut DiagramDocument, ...)` -- mutates `doc.editor_state.edit_mode_target`.
- Clone policy: `NodeId` cloning is acceptable (`String` wrappers), but `DiagramDocument` is not cloned during selection interactions to avoid large allocations.
