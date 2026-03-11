# Contract Specification for seshat-4w4: Cut and Duplicate

## Context
- **Feature**: Cut and Duplicate operations for the Seshat diagram tool (CLP-006 to CLP-010).
- **Domain terms**:
  - **Cut**: Copies the currently selected nodes and edges to the clipboard and removes them from the document.
  - **Duplicate**: Clones the currently selected nodes and edges within the document, applying a spatial offset. The clipboard state is unaffected.
- **Assumptions**:
  - Cut operation modifies both the clipboard and the document state.
  - Duplicate operation bypasses the clipboard entirely to prevent overwriting user's copied data.
  - Offset logic for Duplicate matches the Paste logic (e.g., 20px offset).
- **Open questions**:
  - Should Duplicate place the new nodes exactly 20px offset from the originals, or offset based on a duplicate-serial if executed multiple times in a row? (Assuming 20px offset for the first duplicate).

## Preconditions
- [P1] `SelectionNotEmpty`: Both `cut_selection` and `duplicate_selection` require the document's selection state to contain at least one node.

## Postconditions
- [Q1] `CutReturnsData`: `cut_selection` returns the copied `ClipboardData`.
- [Q2] `CutRemovesNodes`: `cut_selection` removes all previously selected nodes and their connected edges from the document.
- [Q3] `CutClearsSelection`: `cut_selection` leaves the document with an empty selection.
- [Q4] `DuplicateAddsNodes`: `duplicate_selection` increases the document's node and edge count by the number of selected nodes and internal edges.
- [Q5] `DuplicateAssignsNewIds`: `duplicate_selection` generates new unique IDs for all duplicated nodes and edges.
- [Q6] `DuplicateSelectsNew`: `duplicate_selection` updates the document's selection to contain only the newly duplicated nodes.
- [Q7] `DuplicateAppliesOffset`: `duplicate_selection` shifts the coordinates of the duplicated nodes by a fixed offset (e.g., +20px X, +20px Y).

## Invariants
- [I1] `ReferentialIntegrity`: Edges in the document must only connect existing nodes.
- [I2] `ClipboardUntouchedByDuplicate`: The external clipboard state must not be mutated during a duplicate operation.

## Error Taxonomy
- `ClipboardError::EmptySelection` - when attempting to cut or duplicate with an empty selection.

## Contract Signatures
```rust
pub fn cut_selection(doc: &mut DiagramDocument) -> Result<ClipboardData, ClipboardError>;
pub fn duplicate_selection(doc: &mut DiagramDocument) -> Result<(), ClipboardError>;
```

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| `SelectionNotEmpty` | Error variant | `Result<T, ClipboardError::EmptySelection>` |
| `ReferentialIntegrity` | Debug-only / Tests | Property tests checking all edges against nodes map |

## Violation Examples
- VIOLATES P1: `cut_selection(&mut doc)` where `doc.editor_state.selected_items` is empty -- should produce `Err(ClipboardError::EmptySelection)`
- VIOLATES P1: `duplicate_selection(&mut doc)` where `doc.editor_state.selected_items` is empty -- should produce `Err(ClipboardError::EmptySelection)`
- VIOLATES Q2: `cut_selection(&mut doc)` leaves a selected node in `doc.document.nodes` -- test fails.
- VIOLATES Q6: `duplicate_selection(&mut doc)` leaves the original nodes in `doc.editor_state.selected_items` -- test fails.

## Ownership Contracts
- `cut_selection(doc: &mut DiagramDocument)`
  - Exclusive borrow: Mutates `doc.document.nodes` (removes entries), `doc.document.edges` (removes entries), and `doc.editor_state.selected_items` (clears it).
- `duplicate_selection(doc: &mut DiagramDocument)`
  - Exclusive borrow: Mutates `doc.document.nodes` (inserts new entries), `doc.document.edges` (inserts new entries), and `doc.editor_state.selected_items` (replaces with new IDs).

## Non-goals
- Modifying the system clipboard directly (handled by the UI shell, the core returns `ClipboardData`).
