# Contract Specification

## Context
- **Feature**: Self-loop edges render without crash (EDG-032)
- **Bead ID**: oya-r98
- **Domain terms**:
  - Self-loop edge: An edge where source_node_id == target_node_id
  - CyclePolicy: Enum with Allow/Deny modes controlling cycle detection
  - RoutingError: Error type for edge operations
- **Assumptions**:
  - Self-loops should be allowed in non-DAG mode (CyclePolicy::Allow)
  - The edge should render as a loop visual (e.g., a small curve returning to the same node)
- **Open questions**:
  - Should self-loops be rejected in DAG mode (CyclePolicy::Deny)?
  - What visual representation should be used for self-loops?

## Preconditions
- [P1] For `create_edge`: Source and target nodes must exist in document
- [P2] For `create_edge` with CyclePolicy::Deny: Edge must not create a cycle (including self-loop)
- [P3] For rendering: Edge must have valid source and target node references

## Postconditions
- [Q1] After `create_edge` with self-loop: Edge is stored in document with source == target
- [Q2] After `create_edge` with self-loop and CyclePolicy::Allow: Operation succeeds
- [Q3] Self-loop edge renders without panic/crash in canvas rendering

## Invariants
- [I1] Document edges map maintains valid Edge objects
- [I2] Edge source/target references remain valid while edge exists
- [I3] Self-loop edge can be selected, moved, and deleted like normal edges

## Error Taxonomy
- `RoutingError::SourceNotFound(NodeId)` - when source node doesn't exist
- `RoutingError::TargetNotFound(NodeId)` - when target node doesn't exist
- `RoutingError::SelfLoop(NodeId)` - when source == target in strict mode (DAG)
- `RoutingError::CycleDetected` - when edge creates a cycle in DAG mode
- `RenderingError::InvalidEdgeGeometry` - when edge has invalid geometry for rendering

## Contract Signatures
```rust
// In routing.rs
pub fn create_edge(
    doc: &mut DiagramDocument,
    source: NodeId,
    target: NodeId,
    edge_id: EdgeId,
    allow_self_loop: bool,
) -> Result<(), RoutingError>

// In rendering
pub fn render_edge(edge: &Edge, source_node: &Node, target_node: &Node) -> Result<RenderedEdge, RenderingError>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Nodes exist | Runtime (Result) | `RoutingError::SourceNotFound/TargetNotFound` |
| P2: No cycles in DAG mode | Runtime (Result) | `RoutingError::CycleDetected` |
| P3: Valid references | Compile-time | `&Node` references in render |

## Violation Examples (REQUIRED)
- VIOLATES P2: `create_edge(doc, node_a, node_a, edge_id, false)` with CyclePolicy::Deny -- should produce `Err(RoutingError::SelfLoop(node_a))`
- VIOLATES Q2: `create_edge(doc, node_a, node_a, edge_id, false)` with CyclePolicy::Allow -- should succeed but currently fails with SelfLoop error
- VIOLATES Q3: Rendering edge with source == target without self-loop handling -- should NOT panic, should render loop visual

## Ownership Contracts
- `create_edge` takes `&mut DiagramDocument` - mutates `document.edges` hashmap
- Rendering functions take shared references `&Edge`, `&Node` - no mutation

## Non-goals
- [ ] Self-loop visual design (default curve is acceptable)
- [ ] Self-loop specific interactions (drag, resize behavior)
