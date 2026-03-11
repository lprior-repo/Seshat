# Contract Specification: DomainOp UpdateLabel Projection

## Context
- **Feature**: Apply `UpdateLabel` operation to update label text in `DiagramDocument`
- **Domain terms**:
  - `DiagramDocument` - Root document containing nodes and edges
  - `Projection` - Function that applies DomainOp to update document state
  - `UpdateLabel` - Operation to update node/edge label text
- **Assumptions**:
  - UpdateLabel variant exists in DomainOp enum (seshat-rqy)
  - DiagramDocument has nodes map with label field
- **Open questions**: Should UpdateLabel work for both nodes and edges?

## EARS Requirements

| ID | Type | Requirement |
|----|------|-------------|
| EARS-1 | Ubiquitous | The projection reducer SHALL apply UpdateLabel to update label text |
| EARS-2 | Event-Driven | When UpdateLabel is projected, the target's label SHALL be replaced |
| EARS-3 | Unwanted | If target doesn't exist, the system SHALL return an error (not panic) |
| EARS-4 | Unwanted | If label is empty string, this is valid (clearing label) |

## Preconditions

| ID | Description | Type Enforcement |
|----|-------------|------------------|
| P1 | UpdateLabel operation is valid | Compile-time: enum variant guarantees |
| P2 | Target node exists in document | Runtime: `ProjectionError::TargetNotFound` |
| P3 | Label is valid UTF-8 | Compile-time: String guarantees |

## Postconditions

| ID | Description |
|----|-------------|
| Q1 | Node.label == operation.label (exact string match) |
| Q2 | Node.x, Node.y remain unchanged |
| Q3 | Node.width, Node.height remain unchanged |
| Q4 | All other nodes in document unchanged |
| Q5 | Document revision incremented |

## Invariants

| ID | Description |
|----|-------------|
| INV-1 | Document remains valid after projection |
| INV-2 | No nodes are deleted or added by UpdateLabel |
| INV-3 | Edges remain unaffected by label changes |

## Error Taxonomy

- `ProjectionError::TargetNotFound(String)` - Target ID not in document
- `ProjectionError::InvalidOperation(String)` - Operation type mismatch

## Contract Signatures

```rust
// In models/projection.rs
pub enum ProjectionError {
    TargetNotFound(String),
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
| Target exists | Runtime error | `doc.nodes.get(&id).is_some()` |

## Violation Examples (REQUIRED)

- **VIOLATES P2**: `project_operation(&mut doc, &DomainOp::UpdateLabel { id: "nonexistent".into(), label: "New".into() })` -- should produce `Err(ProjectionError::TargetNotFound("nonexistent"))`
- **VIOLATES Q1**: After projection, node.label != operation.label -- test fails
- **VIOLATES Q2**: Node.x or Node.y changed -- test fails
- **VIOLATES Q3**: Node.width or Node.height changed -- test fails
- **VIOLATES Q4**: Other nodes affected -- test fails

## Ownership Contracts

- `project_operation` takes `&mut DiagramDocument` - mutates document in place
- `&DomainOp` is borrowed - no ownership transferred
- Mutation postconditions: `doc.nodes[id].label` changes
- Non-mutation: `doc.nodes[id].x`, `doc.nodes[id].y`, `doc.nodes[id].width`, `doc.nodes[id].height` unchanged

## Non-goals

- [ ] Rich text formatting (plain text only)
- [ ] Label length validation (UI layer)
- [ ] Multi-label update in single operation
- [ ] Edge label updates (future enhancement)
