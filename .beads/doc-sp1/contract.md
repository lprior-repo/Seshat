# Contract Specification: Edge Label Inline Editing

## Context
- **Feature**: Double-click to edit edge labels on canvas
- **Domain terms**:
  - `Edge`: Graph edge with source, target, label, and styling
  - `EdgeId`: Unique identifier for edges
  - `Editing mode`: State where input overlay is shown for label editing
  - `History`: Undo/redo stack for document changes
- **Assumptions**:
  - Canvas coordinate system is already established
  - Edge hit detection (find_edge_at) works correctly
  - History system (mutate_doc_with_history) correctly records changes
- **Open questions**:
  - What zoom threshold should trigger label display? (Currently 0.3)
  - Should empty label input show placeholder text inside the input?

## Preconditions
- **P1**: Double-click must occur on an existing edge in the document
  - Enforcement: Runtime check via `find_edge_at()` returning `Some(EdgeId)`
- **P2**: Edge must exist in document's edge collection
  - Enforcement: Runtime check via `doc.document.edges.get(&eid)`
- **P3**: Document must not be in a read-only state
  - Enforcement: Implicit - mutations are allowed when not locked

## Postconditions
- **Q1**: After double-click on edge, `editing_edge` signal contains the target edge ID
  - Violation: `editing_edge.read()` returns `None` after double-click on valid edge
- **Q2**: After double-click, `edit_value` signal contains the current label (or empty string)
  - Violation: `edit_value.read()` does not match `edge.label`
- **Q3**: After Enter key in editing mode, document's edge label is updated
  - Violation: `doc.document.edges.get(&eid).label` unchanged after Enter
- **Q4**: After Enter key, history is updated for undo/redo
  - Violation: History stack unchanged after label change
- **Q5**: After Escape key, editing mode is exited (editing_edge = None)
  - Violation: `editing_edge.read()` still contains edge ID after Escape
- **Q6**: Empty labels display placeholder when edge is selected
  - Violation: No placeholder text shown for selected edge with empty label

## Invariants
- **I1**: At most one element (node OR edge) can be in editing mode at a time
  - When `editing_node` is Some, `editing_edge` must be None (and vice versa)
- **I2**: edit_value signal reflects the current input, not the saved label
- **I3**: History records every successful label change (not cancelations)

## Error Taxonomy
- **Error::EdgeNotFound**: Edge does not exist in document
  - Happens when: Edge was deleted between hit detection and commit
- **Error::PreconditionViolation**: Double-click occurred on non-edge element
  - Happens when: find_edge_at returns None
- **Error::HistoryFull**: Cannot push to history (unlikely with truncation)
  - Happens when: History push fails

## Contract Signatures

```rust
/// Commits inline label edit for node or edge
/// Returns: ()
/// Errors: None (errors are logged, not returned)
pub fn commit_inline_edit(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    editing_node: Signal<Option<NodeId>>,
    editing_edge: Signal<Option<EdgeId>>,
    edit_value: Signal<String>,
) {
    // Preconditions:
    // - Either editing_node or editing_edge is Some (not both)
    // - If editing_edge is Some, edge exists in doc
    
    // Postconditions:
    // - If label changed: document updated AND history updated
    // - editing_node set to None
    // - editing_edge set to None
}
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Edge exists | Runtime-checked | `find_edge_at()` returns `Option<EdgeId>` |
| Edge in document | Runtime-checked | `doc.edges.get(&eid)` returns `Option<&Edge>` |
| At most one editing | Runtime invariant | Mutual exclusion in signal updates |

## Violation Examples

- **VIOLATES P1**: Double-click on canvas background (no edge hit)
  - Expected: `find_edge_at()` returns `None`, no editing mode started
- **VIOLATES Q1**: After double-click, editing_edge remains None
  - Expected: `editing_edge.set(Some(eid))` called with correct EdgeId
- **VIOLATES Q3**: After Enter, edge.label unchanged
  - Expected: `commit_inline_edit` updates edge and calls `mutate_doc_with_history`
- **VIOLATES Q4**: After Enter, history not updated
  - Expected: `mutate_doc_with_history` is called to record change
- **VIOLATES Q5**: After Escape, still in editing mode
  - Expected: `editing_edge.set(None)` called on Escape key
- **VIOLATES Q6**: Empty label shows no placeholder
  - Expected: Placeholder text rendered when `edge.label.is_empty() && is_selected`

## Ownership Contracts

- **doc_signal**: `&mut Signal<DiagramDocument>` - mutations to document nodes/edges
  - Mutates: `doc.document.edges` (label field), `doc.revision`
- **history_signal**: `&mut Signal<History>` - mutations to undo/redo stacks
  - Mutates: `history.undo_stack` (via push)
- **editing_node**: `&mut Signal<Option<NodeId>>` - cleared after commit/cancel
  - Mutates: Sets to `None`
- **editing_edge**: `&mut Signal<Option<EdgeId>>` - cleared after commit/cancel
  - Mutates: Sets to `None`
- **edit_value**: `&mut Signal<String>` - updated on user input
  - Mutates: Sets to user input string

## Non-goals
- [ ] Multi-line label editing (labels are single-line)
- [ ] Rich text editing (plain text only)
- [ ] Label position adjustment (label always at edge midpoint)
- [ ] Direct canvas text editing without double-click trigger
