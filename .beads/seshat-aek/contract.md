# Contract Specification

## Context
- **Feature**: Update projection reducer to apply UpdateEdgeStyle changes
- **Bead**: seshat-aek
- **Domain terms**:
  - `DiagramProjection` - the domain model representing the diagram state
  - `DiagramDocument` - the DTO for serialization
  - `apply_operation` - function that dispatches DomainOp to handler functions
  - `dispatch_operation` - match statement routing to specific apply_* functions
- **Assumptions**:
  - DomainOp::UpdateEdgeStyle variant exists (from seshat-81d)
  - DiagramProjection has edges: HashMap<EdgeId, Edge> field
  - Edge struct has style: EdgeStyle field
- **Open questions**:
  - Should this return error if edge doesn't exist, or be idempotent?

## EARS Requirements
- **Ubiquitous**: THE SYSTEM SHALL persist edge style changes such that replay produces same visual result
- **Event-driven**: WHEN UpdateEdgeStyle operation is replayed, THE SYSTEM SHALL update the edge's style field
- **Unwanted**: IF the edge ID does not exist, THE SYSTEM SHALL return an error (not silently ignore)

## Preconditions
- [P1] **Operation is UpdateEdgeStyle**: The DomainOp being applied must be UpdateEdgeStyle variant
- [P2] **Edge exists**: The edge with the specified id must exist in the projection
- [P3] **Valid style**: The style field must be a valid EdgeStyle variant

## Postconditions
- [Q1] **Style updated**: After apply, edges[id].style equals the new style value
- [Q2] **Other fields unchanged**: All other edge fields (source, target, label, thickness, arrow_type) remain unchanged
- [Q3] **Other edges unchanged**: All other edges in the projection are unaffected
- [Q4] **Returns Ok**: Function returns Ok(DiagramProjection) on success

## Invariants
- [I1] **Edge count preserved**: Number of edges in projection remains constant (style is not a structural change)
- [I2] **Node integrity**: Connected nodes are not affected by edge style changes

## Error Taxonomy
- **ReplayError::EdgeNotFound** - The edge ID in UpdateEdgeStyle does not exist in the projection
- **ReplayError::InvalidOperation** - Operation variant is not handled

## Contract Signatures

### New apply_update_edge_style function
```rust
pub fn apply_update_edge_style(
    state: DiagramProjection,
    id: &str,
    style: EdgeStyle,
) -> Result<DiagramProjection, ReplayError>
```

### dispatch_operation match arm
```rust
DomainOp::UpdateEdgeStyle { id, style } => apply_update_edge_style(state, id, style),
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: UpdateEdgeStyle variant | Compile-time | Rust match exhaustiveness |
| P2: Edge exists | Runtime | `state.edges.get(id).is_some()` returning EdgeNotFound if not |
| P3: Valid style | Compile-time | `enum EdgeStyle` guarantees validity |

## Violation Examples

### Precondition Violations
- **VIOLATES P2**: Edge doesn't exist
  - Input: apply_update_edge_style(state, "e999", EdgeStyle::Dashed) where e999 not in state.edges
  - Expected: `Err(ReplayError::EdgeNotFound)`

### Postcondition Violations
- **VIOLATES Q1**: Style not updated
  - Input: UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dotted }
  - After: state.edges["e1"].style != EdgeStyle::Dotted
  - Expected: style equals Dotted

- **VIOLATES Q2**: Other fields changed
  - Input: UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed }
  - After: state.edges["e1"].source != original.source OR state.edges["e1"].target != original.target
  - Expected: All connectivity/dimension fields unchanged

## Ownership Contracts

- **state: DiagramProjection**: Passed by value, new projection returned (functional update)
- **id: &str**: Borrowed, no ownership transferred
- **style: EdgeStyle**: Copy type, no ownership concerns

## Non-goals
- [ ] Batch style updates for multiple edges
- [ ] Thickness changes (separate concern - edge.thickness field)
- [ ] Undo/redo (handled by event history)

---

## Implementation Phases

### Phase 1: Add apply_update_edge_style
1. Create function in projection/ops/edge_ops.rs or similar
2. Validate edge exists, return EdgeNotFound if not
3. Clone projection, update style field, return new projection
4. Ensure other fields are preserved (functional update pattern)

### Phase 2: Wire in dispatch_operation
1. Add match arm in dispatch_operation function
2. Destructure id and style from UpdateEdgeStyle variant
3. Call apply_update_edge_style

### Phase 3: Error Handling
1. Ensure ReplayError::EdgeNotFound is defined
2. Map error appropriately in apply_operation wrapper
