# Contract Specification

## Context
- Feature: Edge text inline editing
- Domain terms: 
  - `Edge Label`: The text associated with a specific connection (edge) in the diagram.
  - `Drafted Text`: The new text proposed for the edge label.
- Assumptions: 
  - The domain is strictly responsible for updating the edge label data in the document state.
  - UI state (like "inline edit mode") is managed by a separate presentation layer and is strictly isolated from the core domain contract. The domain knows nothing about the UI editor.
- Open questions: None.

## Preconditions
- `edge_id` must refer to an existing edge in the domain document.

## Postconditions
- On a successful edit application, the target edge's label in the domain document is updated to exactly match the newly provided text.
- If the target edge does not exist, the domain document state remains unchanged.
- If persistence or other underlying operations fail, the domain document state remains unchanged (atomicity).

## Invariants
- The structure and connectivity of the edge (source and target nodes) must remain unchanged during a label edit.
- Document integrity is maintained regardless of label update success or failure.

## Error Taxonomy
- `TargetNotFound` - The specified `edge_id` does not exist in the document.
- `UpdateFailed` - The system failed to persist the new label.

## Contract Signatures
- `fn apply_edge_label_edit(edge_id: EdgeId, new_label: String) -> Result<(), EditError>`
  - This is a pure domain function responsible for the state mutation of the edge label. It has absolutely no knowledge of or interaction with UI edit modes, sessions, or visual state.

## Non-goals
- Refactoring the entire editor state machine.
- Adding comprehensive rich-text support to edge labels.
- Managing UI-specific "edit sessions" or presentation states within the domain model.