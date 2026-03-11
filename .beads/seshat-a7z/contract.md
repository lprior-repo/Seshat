# Contract Specification: seshat-a7z (Marquee Selection / SEL-011 to SEL-015)

## Context
- Feature: Implement test cases SEL-011 through SEL-015 for selection (Alt-click parent, Locked element, Hidden element, Right-click select, Edge/Connector selection).
- Domain terms:
  - `Node`: A graphical element on the canvas.
  - `Edge`/`Connector`: A line connecting two nodes.
  - `Container`/`Subgraph`: A node that can contain other nodes.
  - `SelectionState` / `selected_items`: The set of currently selected element IDs.
  - `Modifiers`: Keyboard state (Alt, Shift, Ctrl/Cmd) during click.
- Assumptions:
  - Hit testing considers element z-order and state (`locked`, `visibility`).
  - Edges have IDs and can be part of `selected_items`.
- Open questions:
  - Do edges have a `locked` or `hidden` state? Assumed yes or inherited.
  - What happens if the parent container is locked but the child is clicked with Alt? Assumed the parent is not selected.

## Preconditions
- [P1] Alt-click parent selection requires the target node to have a parent container.
- [P2] Elements must be unlocked (`locked == false`) to be selected.
- [P3] Elements must be visible (`visibility != hidden`) to be hit-testable.
- [P4] Edge selection requires the edge ID to exist in the document.

## Postconditions
- [Q1] `alt_click`: `selected_items` contains only the parent ID, not the child ID.
- [Q2] `click_locked`: `selected_items` remains completely unchanged.
- [Q3] `click_hidden`: Hidden element is ignored; click passes through to the node underneath (if any).
- [Q4] `right_click_unselected`: Node becomes selected (replacing previous selection).
- [Q5] `click_edge`: Edge is selected (not the nodes).

## Invariants
- [I1] `selected_items` never contains a locked element.
- [I2] `selected_items` never contains a hidden element.

## Error Taxonomy
- `SelectionError::ElementLocked` - when attempting to explicitly select an element with `locked: true`.
- `SelectionError::ElementHidden` - when attempting to interact with a hidden element.
- `SelectionError::ElementNotFound` - when attempting to select an edge or node ID that doesn't exist in the document.
- `SelectionError::NoParentContainer` - when alt-clicking a node that has no parent container.
- `SelectionError::PreconditionViolated` - fallback when a postcondition state is invalid.

## Contract Signatures
- `fn select_element(id: ElementId, modifiers: SelectModifiers) -> Result<SelectionState, SelectionError>`
- `fn hit_test(point: Point, document: &Document) -> Result<Option<ElementId>, SelectionError>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Node has parent | Error variant | `Result<SelectionState, SelectionError::NoParentContainer>` |
| P2: Element unlocked | Compile-time / Error variant | `UnlockedElement` or `Result<..., SelectionError::ElementLocked>` |
| P3: Element visible | Compile-time / Error variant | `VisibleElement` or `Result<..., SelectionError::ElementHidden>` |
| P4: Edge exists | Error variant | `Result<..., SelectionError::ElementNotFound>` |

## Violation Examples
- VIOLATES P1: `select_element(root_node_id, SelectModifiers { alt: true, .. })` -- should produce `Err(SelectionError::NoParentContainer)`
- VIOLATES P2: `select_element(locked_node_id, SelectModifiers::default())` -- should produce `Err(SelectionError::ElementLocked)`
- VIOLATES P3: `select_element(hidden_node_id, SelectModifiers::default())` -- should produce `Err(SelectionError::ElementHidden)`
- VIOLATES P4: `select_element(non_existent_edge_id, SelectModifiers::default())` -- should produce `Err(SelectionError::ElementNotFound)`
- VIOLATES Q1: `selected_items.contains(child_id)` after Alt-click -- should produce `Err(SelectionError::PreconditionViolated)`
- VIOLATES Q2: `selected_items.contains(locked_id)` after clicking locked node -- should produce `Err(SelectionError::ElementLocked)`
- VIOLATES Q3: `selected_items.contains(hidden_id)` after clicking hidden node -- should produce `Err(SelectionError::ElementHidden)`
- VIOLATES Q4: `selected_items` is empty after right-clicking an unselected node -- should produce `Err(SelectionError::PreconditionViolated)`
- VIOLATES Q5: `selected_items` does not contain edge ID after clicking edge -- should produce `Err(SelectionError::PreconditionViolated)`

## Ownership Contracts (Rust-specific)
- Exclusive borrow: `fn select_element(state: &mut SelectionState, document: &Document, id: ElementId, modifiers: &SelectModifiers)` -- Mutates `state.selected_items`
- Shared borrow: `fn hit_test(point: Point, document: &Document)` -- Read-only access to document geometry and properties.

## Non-goals
- Multi-selection of mixed locked and unlocked items via marquee (covered in other SEL cases).
