# Contract Specification: Clipboard Contract (CLP-002 to CLP-006)

## Context
- **Feature**: Multi-node copy/paste with edge bindings and subgraph preservation
- **Domain terms**:
  - `Selection`: set of node IDs to operate on
  - `ClipboardData`: serialized nodes and edges for clipboard storage
  - `PasteResult`: newly created node and edge IDs after paste
  - `paste_serial`: counter for incremental offset calculation
- **Assumptions**:
  - NodeId and EdgeId are generated using UUID v4
  - Document is valid before and after operations
- **Open questions**: None

## Preconditions
- [P1] `copy(selection, _)` requires `!selection.nodes.is_empty()`
- [P2] `cut(selection, _)` requires `!selection.nodes.is_empty()`
- [P3] `paste(_, _, _)` requires `!clipboard.nodes.is_empty()`

## Postconditions
- [Q1] `copy` does not mutate the document (pure function)
- [Q2] `paste` generates new unique IDs for all nodes: `new_id != old_id`
- [Q3] `paste` generates new unique IDs for all edges: `new_edge_id != old_edge_id`
- [Q4] `cut` removes all selected nodes from `doc.document.nodes`
- [Q5] `paste` applies offset: `node.x = original.x + (20.0 * paste_serial)`
- [Q6] `paste` remaps edge source/target to new node IDs
- [Q7] `paste` clears `selected_items` after paste

## Invariants
- [I1] After `paste`, all edges reference nodes that exist in `doc.document.nodes`
- [I2] After `paste`, all node parent references exist in `doc.document.nodes` or are None

## Error Taxonomy
```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Empty selection")]
    EmptySelection,                    // P1, P2 violated
    #[error("Empty clipboard")]
    EmptyClipboard,                    // P3 violated
    #[error("Invalid clipboard data")]
    InvalidClipboardData,
    #[error("Duplicate ID created")]
    DuplicateIdCreated,                // ID collision
    #[error("Invalid edge reference")]
    InvalidEdgeReference,              // Q6 violated
    #[error("Invalid parent reference")]
    InvalidParentReference,            // Q7 violated
    #[error("Postcondition violated: {0}")]
    PostconditionViolated(String),     // Q1 violated
}
```

## Contract Signatures
```rust
pub fn copy(selection: &Selection, doc: &DiagramDocument) -> Result<ClipboardData, Error>
pub fn cut(selection: &Selection, doc: &mut DiagramDocument) -> Result<ClipboardData, Error>
pub fn paste(clipboard: &ClipboardData, doc: &mut DiagramDocument, paste_serial: u32) -> Result<PasteResult, Error>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| selection.nodes not empty | Runtime-checked constructor | `if selection.nodes.is_empty() { return Err(Error::EmptySelection) }` |
| clipboard.nodes not empty | Runtime-checked constructor | `if clipboard.nodes.is_empty() { return Err(Error::EmptyClipboard) }` |
| node exists in document | Runtime-checked with Result | `.get(id).ok_or_else(...)` |
| new ID does not collide | Runtime-checked with Result | `.contains_key(&new_id)` |
| edge references valid nodes | Runtime-checked with Result | check in paste function |

## Violation Examples (REQUIRED)
- VIOLATES P1: `copy(&Selection { nodes: vec![] }, &doc)` -- returns `Err(Error::EmptySelection)`
- VIOLATES P2: `cut(&Selection { nodes: vec![] }, &mut doc)` -- returns `Err(Error::EmptySelection)`
- VIOLATES P3: `paste(&ClipboardData::empty(), &mut doc, 1)` -- returns `Err(Error::EmptyClipboard)`
- VIOLATES Q1: Calling copy and observing doc.nodes changed -- returns `Err(Error::PostconditionViolated)`
- VIOLATES Q2: paste() results in new node ID equal to original -- returns `Err(DuplicateIdCreated)` or same ID
- VIOLATES Q6: paste() with edge pointing to non-existent node -- returns `Err(Error::InvalidEdgeReference)`
- VIOLATES Q7: paste() with parent pointing to non-existent node -- returns `Err(Error::InvalidParentReference)`

## Ownership Contracts (Rust-specific)
- `copy(selection: &Selection, doc: &DiagramDocument)`:
  - Shared borrow of both parameters
  - No mutation, pure function
  - Returns owned ClipboardData
- `cut(selection: &Selection, doc: &mut DiagramDocument)`:
  - Exclusive borrow of doc
  - Mutates: `doc.document.nodes` (removes nodes), `doc.editor_state.selected_items` (removes selections)
  - Returns owned ClipboardData
- `paste(clipboard: &ClipboardData, doc: &mut DiagramDocument, paste_serial: u32)`:
  - Exclusive borrow of doc
  - Mutates: `doc.document.nodes` (adds nodes), `doc.document.edges` (adds edges), `doc.editor_state.selected_items` (sets to pasted nodes)
  - Returns owned PasteResult

## Non-goals
- [ ] Copy/paste across different documents
- [ ] Undo/redo functionality
- [ ] Clipboard persistence across sessions
