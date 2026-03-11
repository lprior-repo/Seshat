# Contract Specification: DomainOp NodeResize Projection

## Context
- **Feature**: Apply `NodeResize` operation to update node dimensions in `DiagramDocument`
- **Domain terms**:
  - `DiagramDocument` - Root document containing nodes and edges
  - `Projection` - Function that applies DomainOp to update document state
  - `NodeResize` - Operation to update node width/height
- **Assumptions**:
  - NodeResize variant exists in DomainOp enum (seshat-fir)
  - DiagramDocument has nodes map with width/height fields
- **Open questions**: None

## EARS Requirements

| ID | Type | Requirement |
|----|------|-------------|
| EARS-1 | Ubiquitous | The projection reducer SHALL apply NodeResize to update width/height |
| EARS-2 | Event-Driven | When NodeResize is projected, the target node's dimensions SHALL be updated |
| EARS-3 | Unwanted | If node doesn't exist, the system SHALL return an error (not panic) |
| EARS-4 | Unwanted | If dimensions are invalid (NaN, Infinity), the system SHALL return an error |

## Preconditions

| ID | Description | Type Enforcement |
|----|-------------|------------------|
| P1 | NodeResize operation is valid | Compile-time: enum variant guarantees |
| P2 | Target node exists in document | Runtime: `ProjectionError::NodeNotFound` |
| P3 | width is finite and > 0 | Runtime: `ProjectionError::InvalidDimensions` |
| P4 | height is finite and > 0 | Runtime: `ProjectionError::InvalidDimensions` |

## Postconditions

| ID | Description |
|----|-------------|
| Q1 | Node.width == operation.width |
| Q2 | Node.height == operation.height |
| Q3 | Node.x, Node.y remain unchanged |
| Q4 | Node.label remains unchanged |
| Q5 | All other nodes in document unchanged |
| Q6 | Document revision incremented |

## Invariants

| ID | Description |
|----|-------------|
| INV-1 | Document remains valid after projection: all nodes have positive finite dimensions |
| INV-2 | No nodes are deleted or added by NodeResize |
| INV-3 | Edges remain unaffected by node dimension changes |

## Error Taxonomy

- `ProjectionError::NodeNotFound(String)` - Target node ID not in document
- `ProjectionError::InvalidDimensions(String)` - width or height is NaN, Infinity, or <= 0
- `ProjectionError::InvalidOperation(String)` - Operation type mismatch

## Contract Signatures

```rust
// In models/projection.rs
pub enum ProjectionError {
    NodeNotFound(String),
    InvalidDimensions(String),
    InvalidOperation(String),
}

pub fn project_operation(
    doc: &mut DiagramDocument, 
    operation: &DomainOp
) -> Result<(), ProjectionError>;
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|--------------|-------------------|----------------|
| Node exists | Runtime error | `doc.nodes.get(&id).is_some()` |
| Dimensions valid | Runtime validation | `is_finite() && > 0.0` check |

## Violation Examples (REQUIRED)

- **VIOLATES P2**: `project_operation(&mut doc, &DomainOp::NodeResize { id: "nonexistent".into(), width: 80.0, height: 40.0 })` -- should produce `Err(ProjectionError::NodeNotFound("nonexistent"))`
- **VIOLATES P3**: `project_operation(&mut doc, &DomainOp::NodeResize { id: "n1".into(), width: f64::NAN, height: 40.0 })` -- should produce `Err(ProjectionError::InvalidDimensions(...))`
- **VIOLATES P3**: `project_operation(&mut doc, &DomainOp::NodeResize { id: "n1".into(), width: f64::INFINITY, height: 40.0 })` -- should produce `Err(ProjectionError::InvalidDimensions(...))`
- **VIOLATES P3**: `project_operation(&mut doc, &DomainOp::NodeResize { id: "n1".into(), width: -10.0, height: 40.0 })` -- should produce `Err(ProjectionError::InvalidDimensions(...))`
- **VIOLATES P4**: Same as P3 but for height field
- **VIOLATES Q1**: After projection, node.width != operation.width -- test fails
- **VIOLATES Q2**: After projection, node.height != operation.height -- test fails
- **VIOLATES Q3**: Node.x or Node.y changed -- test fails
- **VIOLATES Q4**: Node.label changed -- test fails
- **VIOLATES Q5**: Other nodes affected -- test fails

## Ownership Contracts

- `project_operation` takes `&mut DiagramDocument` - mutates document in place
- `&DomainOp` is borrowed - no ownership transferred
- Mutation postconditions: `doc.nodes[id].width`, `doc.nodes[id].height` change
- Non-mutation: `doc.nodes[id].x`, `doc.nodes[id].y`, `doc.nodes[id].label` unchanged

## Non-goals

- [ ] Multi-node resize in single operation
- [ ] Maintaining aspect ratio
- [ ] Minimum/maximum dimension enforcement (UI layer)
- [ ] Edge rerouting after resize (separate operation)
