bead_id: seshat-i0u
bead_title: Implement copy/paste for single node (CLP-001)
phase: implementation
updated_at: 2026-03-14T16:05:00Z

# Implementation Summary

## Location
- Main implementation: `diagram_tool/src/models/clipboard_contract.rs`
- Tests: `diagram_tool/src/models/clipboard_contract_tests.rs`

## Implemented Functions

### copy(selection: &Selection, doc: &DiagramDocument) -> Result<ClipboardData, Error>
- Validates selection is not empty (P1)
- Copies selected nodes to ClipboardData
- Copies edges where both source and target are in selection
- Returns ClipboardData with original node IDs preserved

### cut(selection: &Selection, doc: &mut DiagramDocument) -> Result<ClipboardData, Error>
- Calls copy() to create clipboard
- Removes selected nodes from document
- Clears selection in editor_state

### paste(clipboard: &ClipboardData, doc: &mut DiagramDocument, paste_serial: u32) -> Result<PasteResult, Error>
- Validates clipboard is not empty (P3)
- Generates new UUIDs for each node
- Applies offset based on paste_serial (20.0 * serial)
- Remaps parent references if parent is in pasted set or exists in document
- Creates PasteResult with new node IDs
- Updates editor_state.selected_items to new nodes

## Data Flow
1. User selects node(s)
2. copy() serializes nodes to ClipboardData
3. paste() deserializes with new UUIDs and offset
4. New nodes are inserted into document

## Error Handling
- EmptySelection: copy/cut with no selection
- EmptyClipboard: paste with no data
- DuplicateIdCreated: UUID collision (extremely rare)
- InvalidEdgeReference: Edge references non-existent node
- InvalidParentReference: Parent node doesn't exist

## Test Coverage
All 14 contract tests pass:
- Happy path: CLP-001 single node copy/paste
- Error paths: Empty selection, empty clipboard
- Edge cases: Multiple pastes with incremental offset
- Contract violations: All pre/postconditions verified
