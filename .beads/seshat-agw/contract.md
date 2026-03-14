# Contract Specification: Moving Bound Nodes (EDG-017 to EDG-021)

## Context
- **Feature**: Edges must auto-recalculate paths when bound nodes are moved
- **Bead ID**: seshat-agw
- **Domain terms**:
  - `Edge`: A connection between two nodes, defined by `source: NodeId` and `target: NodeId`
  - `Bound nodes`: The source and target nodes that an edge is connected to
  - `Edge path`: The SVG path string computed from source/target node positions
- **Assumptions**: 
  - Edge paths are currently computed dynamically during rendering
  - Node position updates are already persisted to the document model
  - The rendering layer already queries node positions when drawing edges

## Preconditions
- [P1] The edge must have valid `source` and `target` node references pointing to existing nodes
- [P2] Both source and target nodes must have valid position data (x, y) and dimensions (width, height)
- [P3] The document must contain the nodes referenced by the edge

## Postconditions
- [Q1] After a node position change, any edge connected to that node must render with updated path coordinates
- [Q2] The edge path must use the current (updated) positions of both source and target nodes
- [Q3] Edge endpoints must remain attached to their respective nodes (center-to-center or port-based)

## Invariants
- [I1] An edge's path is always consistent with its bound nodes' current positions
- [I2] Moving a node does not change the edge's topology (source/target remain the same)

## Error Taxonomy
- `Error::SourceNodeNotFound` - when edge references a non-existent source node
- `Error::TargetNodeNotFound` - when edge references a non-existent target node
- `Error::NodePositionUnavailable` - when node exists but lacks valid position data

## Contract Signatures
```rust
// Current edge structure (existing)
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    // ... other fields
}

// The edge_path function computes path from node positions
pub fn edge_path(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> String
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Edge has valid source node | Runtime (document lookup) | Result during render |
| Edge has valid target node | Runtime (document lookup) | Result during render |
| Node has position data | Compile-time | `Node` struct always has x, y, width, height |

## Violation Examples
- **VIOLATES P1**: Edge with source="non-existent" referencing missing node -- should be handled gracefully (skipped in render)
- **VIOLATES P2**: Node exists but has NaN or Inf position values -- should produce Error::NodePositionUnavailable
- **VIOLATES P3**: Edge references node that was deleted from document -- should be handled gracefully
- **VIOLATES Q1**: Move node N1, but edge path still shows old coordinates (stale render)
- **VIOLATES Q2**: Edge path uses cached node positions instead of current positions
- **VIOLATES Q3**: Edge endpoints detach from nodes after move (incorrect anchor)

## Ownership Contracts
- `Edge` is owned by `DiagramDocument.edges: HashMap<EdgeId, Edge>`
- Node lookup: `doc.nodes.get(&edge.source)` returns `Option<&Node>`
- No mutation of edge during node move - path is recomputed on render

## Non-goals
- Automatic rerouting around obstacles (future feature)
- Preserving edge shape during node move (future feature)
- Edge animation during node drag (UI concern, not model)
