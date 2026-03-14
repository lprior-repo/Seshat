bead_id: seshat-i0u
bead_title: Implement copy/paste for single node (CLP-001)
phase: contract
updated_at: 2026-03-14T16:04:00Z

# Contract Specification

## Context
- Feature: CLP-001 - Single node copy/paste
- Domain terms: ClipboardData, Selection, NodeId, EdgeId, DiagramDocument
- Assumptions: Only handles single node copy/paste, does not handle edges yet
- Open questions: None

## Preconditions
- [P1] copy(): selection.nodes must not be empty
- [P2] cut(): selection.nodes must not be empty  
- [P3] paste(): clipboard.nodes must not be empty

## Postconditions
- [Q1] copy(): Returns ClipboardData with original node IDs and node data unchanged
- [Q2] cut(): Removes selected nodes from document, returns ClipboardData
- [Q3] paste(): Creates NEW nodes with NEW UUIDs at offset positions
- [Q4] paste(): Original nodes remain unchanged in document
- [Q5] paste(): Returns new NodeIds that are different from originals

## Invariants
- [I1] Document always contains valid node references after any operation

## Error Taxonomy
- Error::EmptySelection - when copy/cut called with empty selection
- Error::EmptyClipboard - when paste called with empty clipboard
- Error::DuplicateIdCreated - when generated UUID collides with existing
- Error::InvalidEdgeReference - when pasted edge references non-existent node
- Error::InvalidParentReference - when pasted node references non-existent parent
- Error::PostconditionViolated - when internal validation fails

## Contract Signatures
```rust
pub fn copy(selection: &Selection, doc: &DiagramDocument) -> Result<ClipboardData, Error>
pub fn cut(selection: &Selection, doc: &mut DiagramDocument) -> Result<ClipboardData, Error>
pub fn paste(clipboard: &ClipboardData, doc: &mut DiagramDocument, paste_serial: u32) -> Result<PasteResult, Error>
```

## Type Enforcement
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| selection.nodes not empty | Runtime | Result error variant |
| clipboard.nodes not empty | Runtime | Result error variant |

## Violation Examples
- VIOLATES P1: copy(&Selection::empty(), &doc) -> Err(Error::EmptySelection)
- VIOLATES P2: cut(&Selection::empty(), &mut doc) -> Err(Error::EmptySelection)
- VIOLATES P3: paste(&ClipboardData::empty(), &mut doc, 1) -> Err(Error::EmptyClipboard)
- VIOLATES Q3: After paste, new_nodes[0] != original_node_id

## Ownership Contracts
- copy(): Takes &Selection and &DiagramDocument, no mutation
- cut(): Takes &Selection and &mut DiagramDocument, mutates: removes nodes from doc.document.nodes
- paste(): Takes &ClipboardData and &mut DiagramDocument, mutates: adds nodes to doc.document.nodes

## Non-goals
- Multi-node copy/paste with edge handling (CLP-002+)
- Cut/paste edge preservation
- Parent-child relationship remapping beyond single node
