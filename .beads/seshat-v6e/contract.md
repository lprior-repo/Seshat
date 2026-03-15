# Contract Specification: ID Remapping on Paste (CLP-007 to CLP-010)

## Context
- **Feature**: When pasting, rigorously assign new UUIDs to copied nodes and edges, but perfectly preserve the internal edge topology (connections between the copied items) and hierarchical structures (parent-child relationships).
- **Domain terms**:
  - `ClipboardData`: A bundle containing a subset of nodes and edges, along with a paste counter.
  - `DiagramDocument`: The target workspace containing the global map of nodes and edges.
  - `PasteResult`: A calculated delta (or new document state) representing the outcome of the paste.
  - `Topology`: The structural relationship between nodes (via edges and parent/child references).
  - `ID Remapping`: The process of generating new, unique IDs for entities while updating all internal references to point to the newly generated IDs instead of the original ones.
- **Assumptions**:
  - Edges within the clipboard are assumed to be "internal" (i.e., both source and target nodes exist within the same clipboard).
  - The document provides an ID uniqueness guarantee.

## Preconditions
- [P1] The clipboard must contain at least one node to perform a meaningful paste.
- [P2] Every edge in the clipboard must reference source and target nodes that are also present in the clipboard's node list.
- [P3] Every node in the clipboard with a parent reference must point to a parent that is either in the clipboard, or exists in the target document.
- [P4] The clipboard data must not contain cyclic parent-child relationships.
- [P5] The clipboard data itself must not contain duplicate Node IDs or Edge IDs prior to insertion (internal clipboard corruption).

## Postconditions
- [Q1] All pasted nodes are successfully assigned brand new, globally unique IDs.
- [Q2] All pasted edges are successfully assigned brand new, globally unique IDs.
- [Q3] Internal Edge Topology is preserved: If an original edge connected original nodes A and B, the newly generated edge connects the new node IDs A' and B'.
- [Q4] Parent/Child Topology is preserved: If an original node had a parent A in the clipboard, the newly generated node has the new parent ID A'. If the parent was not in the clipboard but is in the document, it retains the original parent ID.
- [Q5] Spatial Offset: Pasted nodes have their coordinates immediately offset by `(serial + 1) * constant` to visually distinguish them from the originals, starting from the very first paste.
- [Q6] The calculated paste serial is strictly incremented by 1 after a successful paste.
- [Q7] Selection state in the `PasteResult` is replaced with the newly generated IDs of the pasted items.

## Invariants
- [I1] The output document delta never contains duplicate node IDs or edge IDs.
- [I2] Every edge in the output delta strictly references existing node IDs for both its source and target.
- [I3] A node cannot be its own parent, nor can there be cycles in the parent-child hierarchy (enforced globally, but relevant on paste).
- [I4] The domain calculation `calculate_paste` never panics, unwraps, or mutates inputs under any hostile condition (fuzzing invariant).

## Error Taxonomy
- `Error::EmptyClipboard` - when the clipboard contains no nodes to paste.
- `Error::InvalidEdgeReference` - when an edge in the clipboard references a node not present in the clipboard.
- `Error::InvalidParentReference` - when a node's parent reference cannot be resolved in either the clipboard or the destination document.
- `Error::CyclicParentReference` - when a node in the clipboard introduces a parent-child cycle.
- `Error::DuplicateIdCreated` - when a newly generated ID collides with an existing one in the document (rare, but contractually handled).
- `Error::CorruptClipboard` - when the incoming clipboard data contains duplicate IDs internally.

## Contract Signatures
```rust
pub fn calculate_paste(
    clipboard: &ClipboardData,
    doc: &DiagramDocument,
) -> Result<PasteResult, Error>
```
*(Note: Strict functional-rust domain contract requires a `Result` returning fallible operations and taking shared references, avoiding `&mut` to adhere to Data -> Calc -> Actions).*

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Clipboard has nodes | Runtime / Error variant | `Result<PasteResult, Error::EmptyClipboard>` |
| Edges are self-contained | Error variant | `Result<PasteResult, Error::InvalidEdgeReference>` |
| Parent is resolvable | Error variant | `Result<PasteResult, Error::InvalidParentReference>` |
| No cyclic references | Error variant | `Result<PasteResult, Error::CyclicParentReference>` |
| Internal clipboard integrity | Error variant | `Result<PasteResult, Error::CorruptClipboard>` |
| No ID collisions | Error variant | `Result<PasteResult, Error::DuplicateIdCreated>` |
| Valid Offset Application | Compile-time | `OrderedFloat` math |

## Violation Examples (REQUIRED)
- VIOLATES P1: `calculate_paste` is called with an empty clipboard -- should produce `Err(Error::EmptyClipboard)`.
- VIOLATES P2: `calculate_paste` is called with a clipboard containing a dangling edge -- should produce `Err(Error::InvalidEdgeReference)`.
- VIOLATES P3: `calculate_paste` is called with a clipboard where a node references a deleted external parent -- should produce `Err(Error::InvalidParentReference)`.
- VIOLATES P4: `calculate_paste` is called with a clipboard containing nodes that form a parent-child cycle -- should produce `Err(Error::CyclicParentReference)`.
- VIOLATES P5: `calculate_paste` is called with a clipboard containing two nodes with the exact same ID -- should produce `Err(Error::CorruptClipboard)`.
- VIOLATES Q1/I1: `calculate_paste` is called when a deterministic PRNG generates a colliding ID -- should produce `Err(Error::DuplicateIdCreated)`.

## Ownership Contracts (Rust-specific)
- Shared borrow: `fn calculate_paste(clipboard: &ClipboardData, doc: &DiagramDocument)`
  - Both `clipboard` and `doc` are read-only. The operation computes a delta and performs no in-place mutation, strictly enforcing functional purity.
