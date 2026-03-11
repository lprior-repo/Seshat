# Contract Specification

## Context
- **Feature**: Advanced Multi-select Operations (MUL-031 to MUL-037)
- **Domain terms**:
  - `Multi-select`: A state where multiple document nodes are targeted for an operation.
  - `Selection Bounds`: The combined bounding box of all selected nodes.
- **Assumptions**:
  - Operations are atomic (either all selected items are updated, or none).
  - Copy/Paste uses an internal clipboard structure independent of the system clipboard.
- **Open questions**:
  - Does resizing a mixed selection of lines and rectangles proportionally scale the lines' endpoints?

## Preconditions
- **P1**: Operations (Move, Resize, Delete, Copy) require a non-empty selection.
- **P2**: Destructive operations (Delete, Move, Resize) must reject if any selected item is locked.
- **P3**: Cannot mutate a selection containing both a parent container and its child if the operation applies recursively.

## Postconditions
- **Q1**: After `delete_selection`, all selected items are removed from the document and the selection is cleared.
- **Q2**: After `move_selection`, the relative spacing and positions of all selected items are strictly preserved.
- **Q3**: After `paste_selection`, copies of clipboard items are instantiated with new IDs, offset by a defined delta, and become the active selection.
- **Q4**: After Undo/Redo, the document state AND the selection state are perfectly restored.

## Invariants
- **I1**: Selection contains no duplicate `NodeId`s.
- **I2**: Selection bounds strictly equal the bounding box of all selected items.
- **I3**: Locked items are never mutated by multi-select operations.

## Error Taxonomy
- `Error::EmptySelection` - Attempted an operation requiring items but the selection was empty.
- `Error::ItemLocked` - Attempted to mutate a selection containing one or more locked items.
- `Error::InvalidHierarchy` - Attempted an operation on a selection containing conflicting parent-child nodes.
- `Error::PostconditionViolated` - An operation completed but failed to satisfy postconditions.

## Contract Signatures
- `fn move_selection(doc: &mut Document, selection: NonEmptyVec<NodeId>, delta: Vector2D) -> Result<(), Error>`
- `fn resize_selection(doc: &mut Document, selection: NonEmptyVec<NodeId>, new_bounds: Rect) -> Result<(), Error>`
- `fn delete_selection(doc: &mut Document, selection: NonEmptyVec<NodeId>) -> Result<(), Error>`
- `fn copy_selection(doc: &Document, selection: NonEmptyVec<NodeId>) -> Result<ClipboardData, Error>`
- `fn paste_selection(doc: &mut Document, clipboard: &ClipboardData, offset: Vector2D) -> Result<Vec<NodeId>, Error>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Selection not empty | Compile-time (strongest) | `NonEmptyVec<NodeId>` |
| P2: Items not locked | Error variant | `Result<T, Error::ItemLocked>` |
| P3: Valid hierarchy | Error variant | `Result<T, Error::InvalidHierarchy>` |

## Violation Examples
- VIOLATES P1: `move_selection(doc, empty_vec, delta)` -- should produce a compile error (requires `NonEmptyVec`).
- VIOLATES P2: `delete_selection(doc, selection_with_locked_node)` -- should produce `Err(Error::ItemLocked)`.
- VIOLATES P3: `move_selection(doc, selection_with_parent_and_child, delta)` -- should produce `Err(Error::InvalidHierarchy)`.
- VIOLATES Q1: `doc.nodes.contains(deleted_node)` is true after `delete_selection` -- should produce `Err(Error::PostconditionViolated)`.

## Ownership Contracts (Rust-specific)
- Exclusive borrow: `fn move_selection(doc: &mut Document, ...)` -- mutates `doc.nodes`.
- Exclusive borrow: `fn resize_selection(doc: &mut Document, ...)` -- mutates `doc.nodes`.
- Exclusive borrow: `fn delete_selection(doc: &mut Document, ...)` -- mutates `doc.nodes` and `doc.selection`.
- Shared borrow: `fn copy_selection(doc: &Document, ...)` -- reads `doc.nodes`, no mutation.
- Exclusive borrow: `fn paste_selection(doc: &mut Document, ...)` -- mutates `doc.nodes` and `doc.selection`.

## Non-goals
- Implementing system clipboard integration (only internal clipboard is covered).
- Complex constraint solving for multi-select resize.
