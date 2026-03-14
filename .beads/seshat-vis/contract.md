# Contract: seshat-vis (CLP-002 to CLP-006)

## Scope Map
- **Feature**: Multi-node copy/paste with edge bindings
- **Location**: `diagram_tool/src/models/clipboard_contract.rs`
- **Tests**: `diagram_tool/src/models/clipboard_contract_tests.rs`

## Contract Clauses

### Preconditions
- **P1**: For `copy()`, selection must not be empty
- **P2**: For `cut()`, selection must not be empty
- **P3**: For `paste()`, clipboard must not be empty

### Postconditions
- **Q1**: Copy operation does not modify the original document
- **Q2**: Paste generates new unique IDs for all nodes and edges
- **Q3**: Paste remaps edge references to point to new node IDs
- **Q4**: Cut removes selected nodes from the document
- **Q5**: Paste applies incremental offset (20.0 * paste_serial)
- **Q6**: Paste validates edge references are valid after remapping
- **Q7**: Paste validates parent references are valid after remapping

### Invariants
- **I1**: Document structure remains valid after any clipboard operation
- **I2**: Selected items state is updated correctly after cut/paste

## Error Taxonomy
- `Error::EmptySelection` - selection.nodes.is_empty()
- `Error::EmptyClipboard` - clipboard.nodes.is_empty()
- `Error::DuplicateIdCreated` - generated UUID collides
- `Error::InvalidEdgeReference` - edge points to non-existent node
- `Error::InvalidParentReference` - parent points to non-existent node
- `Error::PostconditionViolated` - internal contract violation

## Function Signatures
```rust
pub fn copy(selection: &Selection, doc: &DiagramDocument) -> Result<ClipboardData, Error>
pub fn cut(selection: &Selection, doc: &mut DiagramDocument) -> Result<ClipboardData, Error>
pub fn paste(clipboard: &ClipboardData, doc: &mut DiagramDocument, paste_serial: u32) -> Result<PasteResult, Error>
```

## Traceability
| Requirement | Function | Test |
|---|---|---|
| CLP-002 | copy, paste | test_clp002_copy_paste_multiple_nodes_preserves_edges_and_remaps_ids |
| CLP-003 | paste | test_clp003_copy_paste_subgraph_preserves_parent_child_relationships |
| CLP-004 | cut | test_clp004_cut_operation_removes_original_nodes_and_places_in_clipboard |
| CLP-005 | paste | test_clp005_paste_operation_applies_incremental_offset_based_on_serial |
| CLP-006 | paste | test_q6_violation_returns_invalid_edge_reference_error |

## Evaluation Protocol
1. All 14 tests in clipboard_contract_tests.rs must pass
2. Run: `moon run diagram_tool:test`
