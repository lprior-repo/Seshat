# Contract Specification: SEL-001 to SEL-005 (Click Selection)

## Context
- **Feature**: Click-to-select interaction scenarios (SEL-001 through SEL-005)
- **Domain terms**: 
  - `NodeId`: Unique identifier for an item in the diagram
  - `SelectionSet`: A collection of currently selected `NodeId`s
  - `SelectionMode`: Enum representing how a selection is applied (`Replace`, `Toggle`)
  - `HitTestResult`: Enum for what was clicked (`Item(NodeId)`, `Empty`)
  - `MarqueeMode`: Enum for drag direction (`Contain` for L-to-R, `Intersect` for R-to-L)
- **Assumptions**: 
  - Event processing maps raw UI events to explicit explicit selection commands.
  - The document provides a way to query geometry/bounds for marquee intersection.
- **Open questions**:
  - Should dragging a marquee with Shift held toggle the nodes, or add them? (Assuming `Toggle` based on general additive logic, but scope only includes base cases).

## Preconditions
- [P1] When selecting a specific node by ID, the node MUST exist in the document.
- [P2] When starting a marquee selection, the start point MUST hit an empty canvas area, not an interactive node.

## Postconditions
- [Q1] **SEL-001**: After `select_item` with `SelectionMode::Replace`, `selected_items` contains exactly that single `node_id`.
- [Q2] **SEL-002**: After `select_item` with `SelectionMode::Toggle`, if `node_id` was previously selected, it is removed. If it was not selected, it is added.
- [Q3] **SEL-003/SEL-005**: After `marquee_select`, `selected_items` contains nodes based on the marquee bounds and the direction of the drag (Containment for L->R, Intersection for R->L).
- [Q4] **SEL-004**: After `clear_selection`, `selected_items` is empty.

## Invariants
- [I1] `selected_items` MUST NOT contain duplicate `NodeId`s.
- [I2] `selected_items` MUST only contain `NodeId`s that currently exist in the document.

## Error Taxonomy
- `Error::ItemNotFound(NodeId)` - when attempting to select a node that doesn't exist in the document.
- `Error::InvalidInteractionState` - when attempting to start a marquee on a node instead of an empty area.

## Contract Signatures
- `fn select_item(state: &mut DiagramState, id: NodeId, mode: SelectionMode) -> Result<(), Error>`
- `fn clear_selection(state: &mut DiagramState) -> Result<(), Error>`
- `fn marquee_select(state: &mut DiagramState, start: Point, end: Point) -> Result<(), Error>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Node exists | Result / Error variant | `Error::ItemNotFound` |
| P2: Start on empty canvas | Type System | Construct marquee command only from `HitTestResult::Empty` |
| I1: Unique selection | Compile-time | `HashSet<NodeId>` for `selected_items` |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES P1: `select_item(state, NodeId("non-existent"), SelectionMode::Replace)` -- should produce `Err(Error::ItemNotFound)`
- VIOLATES P2: `marquee_select(state, Point::on_node(), end)` -- should produce `Err(Error::InvalidInteractionState)` if explicitly asserting point is empty.
- VIOLATES Q1: `state.selected_items` contains multiple items after `SelectionMode::Replace` -- should fail postcondition debug_assert.
- VIOLATES Q2: `state.selected_items` still contains the item after `SelectionMode::Toggle` on an already selected item -- should fail postcondition debug_assert.
- VIOLATES Q4: `state.selected_items` is not empty after `clear_selection` -- should fail postcondition debug_assert.

## Ownership Contracts (Rust-specific)
- Exclusive borrow: `fn select_item(state: &mut DiagramState, ...)` -- Mutates `state.selected_items`
- Exclusive borrow: `fn clear_selection(state: &mut DiagramState)` -- Clears `state.selected_items`
- Exclusive borrow: `fn marquee_select(state: &mut DiagramState, ...)` -- Mutates `state.selected_items`

## Non-goals
- [ ] Multi-selection via keyboard (Ctrl+A)
- [ ] Selecting edges vs nodes (handled transparently if both use `NodeId` or `ItemId` abstraction)
