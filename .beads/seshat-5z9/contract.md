# Contract Specification

## Context
- Feature: Copy and Paste (CLP-001 to CLP-005)
- Domain terms: 
  - ClipboardData: The payload containing copied nodes and edges.
  - Node ID: Unique identifier for a node.
  - Edge ID: Unique identifier for an edge.
  - Topology: The structure of nodes and their connections via edges.
  - Offset: The coordinate displacement applied to pasted nodes to prevent stacking perfectly on originals.
- Assumptions:
  - Clipboard is thread-local or part of the application state.
  - Selecting an edge without both connected nodes does not copy the edge (or it handles partial edges safely).
- Open questions:
  - What happens if a node is copied, then its parent is deleted before paste? (Assuming paste creates a new parentless node or maps to root).
  - Is the offset cumulative per paste from the same clipboard state? (Assuming yes, `paste_serial` increments).

## Preconditions
- [P1] `copy`: The selection must contain at least one valid node.
- [P2] `copy_with_edges`: If multiple nodes are selected, edges between them must be included.
- [P3] `cut`: The selection must contain at least one valid node to copy and then delete.
- [P4] `paste`: The clipboard must contain at least one valid node.

## Postconditions
- [Q1] `copy`: Original nodes remain unchanged and their IDs are untouched.
- [Q2] `copy_with_edges`: Edges connecting copied nodes are stored in the clipboard alongside nodes.
- [Q3] `cut`: The originally selected nodes and their connected edges are removed from the document.
- [Q4] `paste`: Pasted nodes are assigned completely new, unique IDs.
- [Q5] `paste`: Pasted nodes are offset by `(20.0 * paste_serial)` on both X and Y axes.
- [Q6] `paste`: If edges were copied, the pasted edges correctly reference the new node IDs.
- [Q7] `paste`: Parent-child subgraph relationships are preserved but point to new parent IDs.

## Invariants
- [I1] Edge Reference Validity: Edges in the document and clipboard must always reference existing nodes within their respective contexts.
- [I2] Geometry Validity: Node coordinates and dimensions must be finite numbers (no NaN or Infinity), and dimensions must be strictly positive.
- [I3] ID Uniqueness: No two nodes or edges in the document may share the same ID.

## Error Taxonomy
- `Error::EmptySelection` - when attempting to copy or cut without any selected nodes.
- `Error::EmptyClipboard` - when attempting to paste but the clipboard is empty.
- `Error::InvalidClipboardData` - when clipboard data is malformed (e.g., edges referencing non-existent nodes).
- `Error::DuplicateIdCreated` - when a paste operation generates an ID that already exists.
- `Error::InvalidEdgeReference` - when an edge points to an invalid node.
- `Error::InvalidParentReference` - when a child points to an invalid parent subgraph.
- `Error::PostconditionViolated` - for general postcondition validation failures in tests.

## Contract Signatures
- `fn copy(selection: &Selection, doc: &Document) -> Result<ClipboardData, Error>`
- `fn cut(selection: &Selection, doc: &mut Document) -> Result<ClipboardData, Error>`
- `fn paste(clipboard: &ClipboardData, doc: &mut Document, paste_serial: u32) -> Result<PasteResult, Error>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| [P1] At least one node in selection | Compile-time | `NonEmptyVec<NodeId>` |
| [P2] Edges included if nodes match | Compile-time | `Subgraph` / `ValidatedSelection` |
| [P3] At least one node to cut | Compile-time | `NonEmptyVec<NodeId>` |
| [P4] Clipboard contains data | Compile-time | `NonEmptyClipboardData` |
| [I2] Positive dimensions | Compile-time | `NonZeroU64` / `PositiveF64` |
| [I3] Unique IDs | Debug-only | `debug_assert!(doc.contains_unique_ids())` |

## Violation Examples
- VIOLATES [P1]: `copy(Selection::empty(), &doc)` -- should produce `Err(Error::EmptySelection)`
- VIOLATES [P3]: `cut(Selection::empty(), &mut doc)` -- should produce `Err(Error::EmptySelection)`
- VIOLATES [P4]: `paste(&ClipboardData::empty(), &mut doc, 1)` -- should produce `Err(Error::EmptyClipboard)`
- VIOLATES [Q1]: `copy` implementation modifies original node ID -- should produce `Err(Error::PostconditionViolated("Original node ID changed"))`
- VIOLATES [Q2]: `copy` implementation excludes edges between selected nodes -- should produce `Err(Error::PostconditionViolated("Edges missing from clipboard"))`
- VIOLATES [Q3]: `cut` implementation leaves original nodes in document -- should produce `Err(Error::PostconditionViolated("Nodes not deleted"))`
- VIOLATES [Q4]: `paste` implementation assigns original Node ID to pasted node -- should produce `Err(Error::DuplicateIdCreated)` when validated against document IDs.
- VIOLATES [Q5]: `paste` implementation applies 0 offset when `paste_serial > 0` -- should produce `Err(Error::PostconditionViolated("Incorrect offset applied"))` in test assertions.
- VIOLATES [Q6]: `paste` implementation leaves old edge references in new edges -- should produce `Err(Error::InvalidEdgeReference)` when validated.
- VIOLATES [Q7]: `paste` implementation leaves old parent ID in new child node -- should produce `Err(Error::InvalidParentReference)` when validated.

## Ownership Contracts (Rust-specific)
- Shared borrow: `fn copy(selection: &Selection, doc: &Document)` -- Reads state, does not mutate. `Selection` and `Document` are read-only.
- Exclusive borrow: `fn cut(selection: &Selection, doc: &mut Document)` -- Mutates `doc` by removing nodes and edges present in `selection`.
- Exclusive borrow: `fn paste(clipboard: &ClipboardData, doc: &mut Document, serial: u32)` -- Mutates `doc` by adding new nodes and edges.
- Clone policy: `ClipboardData` is intentionally created by deep-cloning nodes and edges from the document. `paste` also performs deep clones of `ClipboardData` items into the document.
