# Contract Specification

## Context
- **Feature**: DOC-006 to DOC-010: Node deletion and cleanup
- **Domain terms**: 
  - `Node` - A diagram node with id, position, dimensions, style, and lock state
  - `Edge` - A connection between two nodes (source and target)
  - `locked` - Boolean field on Node indicating it cannot be modified/deleted
  - `dangling reference` - An edge referencing a node that no longer exists
- **Assumptions**:
  - Node deletion already exists via `DiagramDocument::remove_node()`
  - Edge cascading on node deletion already implemented
  - Locked node concept exists in the system
- **Open questions**:
  - What exactly are DOC-006 through DOC-010? Need to infer from standard invariants.

## Preconditions
- **P1**: Node must exist in document before deletion
  - Enforcement: Runtime check - returns `DocumentError::NodeNotFound`
- **P2**: If node is locked, deletion should be rejected
  - Enforcement: Runtime check - returns `DocumentError::NodeLocked`

## Postconditions
- **Q1**: After successful deletion, node is no longer in document
- **Q2**: After successful deletion, all edges connected to deleted node are also removed (cascade)
- **Q3**: After failed deletion (locked node), document state unchanged (atomic)
- **Q4**: After failed deletion (node not found), document state unchanged

## Invariants
- **I1**: No edge may reference a non-existent node (dangling reference prevention)
- **I2**: Edge source and target always point to existing nodes in document

## Error Taxonomy
```rust
enum DocumentError {
    NodeNotFound(NodeId),      // P1 violation
    NodeLocked(NodeId),        // P2 violation  
    EdgeAlreadyExists(EdgeId),
    EdgeNotFound(EdgeId),
}
```

## Contract Signatures
```rust
fn remove_node(&mut self, node_id: &NodeId) -> Result<(), DocumentError>
fn is_locked(&self, node_id: &NodeId) -> bool
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Node exists | Runtime-checked constructor | `Result<T, Error::NodeNotFound>` |
| Node not locked | Runtime check | `Result<T, Error::NodeLocked>` |

## Violation Examples
- **VIOLATES P1**: `doc.remove_node(&NodeId::new("nonexistent"))` -- should produce `Err(DocumentError::NodeNotFound(...))`
- **VIOLATES P2**: `doc.remove_node(&NodeId::new("locked_node"))` where node.locked == true -- should produce `Err(DocumentError::NodeLocked(...))`
- **VIOLATES I1**: After `remove_node()`, an edge still references deleted node -- should NOT happen (cascade deletes edges)

## Ownership Contracts
- `remove_node(&mut self, ...)` - Mutates: `self.document.nodes`, `self.document.edges`
- All fields in `document.nodes` and `document.edges` HashMaps may be modified

## Non-goals
- Undo/redo for node deletion (covered by HIS-xxx tests)
- Batch deletion of multiple nodes (atomicity for multi-node ops covered by DOC-014)
- Node restoration (NodeRestore is separate operation)
