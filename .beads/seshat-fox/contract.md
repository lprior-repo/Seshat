# Contract Specification: Edge-Node Binding (EDG-011 to EDG-015)

## Context
- **Feature**: Edge-Node Binding in Document model.
- **Domain terms**:
  - `NodeId`: Unique identifier for a node in a document.
  - `EdgeId`: Unique identifier for an edge in a document.
  - `Document`: The core data structure holding nodes and edges.
  - `Edge`: A connection between a `source` Node and a `target` Node.
- **Assumptions**:
  - An edge must connect two valid nodes that exist in the document at the time of insertion.
  - Removing a node cascades to remove any edges bound to it to maintain referential integrity.
- **Open questions**:
  - Are self-loops permitted? (Assuming yes, but requires validation).
  - Can multiple parallel edges exist between the same source and target? (Assuming yes).

## Preconditions
- **P1**: The `source` node of a new edge must exist in the document.
- **P2**: The `target` node of a new edge must exist in the document.
- **P3**: The `edge_id` for a new edge must not already exist in the document.

## Postconditions
- **Q1**: After creating an edge, the edge exists in the document's edge collection.
- **Q2**: After creating an edge, its `source` and `target` match the provided `NodeId`s.
- **Q3**: After deleting a node, any edge where the deleted node was the `source` or `target` is removed from the document.
- **Q4**: After deleting an edge, the edge is removed but its `source` and `target` nodes remain intact.

## Invariants
- **I1**: For every edge in the document, its `source` node exists in the document.
- **I2**: For every edge in the document, its `target` node exists in the document.

## Error Taxonomy
- `Error::NodeNotFound(NodeId)` - when a referenced node (source or target) does not exist in the document.
- `Error::EdgeAlreadyExists(EdgeId)` - when attempting to insert an edge with an ID that is already in use.
- `Error::EdgeNotFound(EdgeId)` - when attempting to modify or delete a non-existent edge.

## Contract Signatures
- `fn add_edge(doc: &mut Document, edge_id: EdgeId, edge: Edge) -> Result<(), Error>`
- `fn remove_edge(doc: &mut Document, edge_id: &EdgeId) -> Result<(), Error>`
- `fn remove_node(doc: &mut Document, node_id: &NodeId) -> Result<(), Error>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Edge references existing nodes | Result | `Result<(), Error::NodeNotFound>` (Runtime check against Document state) |
| Edge ID is unique | Result | `Result<(), Error::EdgeAlreadyExists>` (Runtime check against Document state) |
| `source` and `target` types | Compile-time | `NodeId` newtype wrapper |

## Violation Examples (REQUIRED)
- VIOLATES P1: `add_edge(&mut doc, id, Edge { source: missing_node, target: valid_node, .. })` -- should produce `Err(Error::NodeNotFound(missing_node))`
- VIOLATES P2: `add_edge(&mut doc, id, Edge { source: valid_node, target: missing_node, .. })` -- should produce `Err(Error::NodeNotFound(missing_node))`
- VIOLATES P3: `add_edge(&mut doc, existing_id, edge)` -- should produce `Err(Error::EdgeAlreadyExists(existing_id))`
- VIOLATES Q3: Attempting to assert `doc.edges.contains_key(e1)` after `remove_node(&mut doc, n1)` where `e1` connects `n1` to `n2` -- should fail because `e1` must be deleted.

## Ownership Contracts (Rust-specific)
- `fn add_edge(doc: &mut Document, edge_id: EdgeId, edge: Edge)`
  - Exclusive borrow: `&mut Document` -- mutates `doc.edges`.
  - Ownership transfer: `edge_id` and `edge` are moved into the document.
- `fn remove_edge(doc: &mut Document, edge_id: &EdgeId)`
  - Exclusive borrow: `&mut Document` -- mutates `doc.edges`.
  - Shared borrow: `&EdgeId` -- read-only identifier for lookup.
- `fn remove_node(doc: &mut Document, node_id: &NodeId)`
  - Exclusive borrow: `&mut Document` -- mutates both `doc.nodes` and `doc.edges` (for cascading removals).
  - Shared borrow: `&NodeId` -- read-only identifier for lookup.

## Non-goals
- Validating the geometric position or visual layout of the edge during binding logic.
