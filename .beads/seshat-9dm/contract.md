# Contract Specification

## Context
- **Feature**: Update projection reducer to apply UpdateNodeStyle changes
- **Bead**: seshat-9dm
- **Domain terms**:
  - `DiagramProjection` - the domain model representing the diagram state
  - `DiagramDocument` - the DTO for serialization
  - `apply_operation` - function that dispatches DomainOp to handler functions
  - `dispatch_operation` - match statement routing to specific apply_* functions
- **Assumptions**:
  - DomainOp::UpdateNodeStyle variant exists (from seshat-2wb)
  - DiagramProjection has nodes: HashMap<NodeId, Node> field
  - Node struct has style: NodeStyle field
- **Open questions**:
  - Should this return error if node doesn't exist, or be idempotent?

## EARS Requirements
- **Ubiquitous**: THE SYSTEM SHALL persist node style changes such that replay produces same visual result
- **Event-driven**: WHEN UpdateNodeStyle operation is replayed, THE SYSTEM SHALL update the node's style field
- **Unwanted**: IF the node ID does not exist, THE SYSTEM SHALL return an error (not silently ignore)

## Preconditions
- [P1] **Operation is UpdateNodeStyle**: The DomainOp being applied must be UpdateNodeStyle variant
- [P2] **Node exists**: The node with the specified id must exist in the projection
- [P3] **Valid style**: The style field must be a valid NodeStyle variant

## Postconditions
- [Q1] **Style updated**: After apply, nodes[id].style equals the new style value
- [Q2] **Other fields unchanged**: All other node fields (x, y, width, height, label) remain unchanged
- [Q3] **Other nodes unchanged**: All other nodes in the projection are unaffected
- [Q4] **Returns Ok**: Function returns Ok(DiagramProjection) on success

## Invariants
- [I1] **Node count preserved**: Number of nodes in projection remains constant (style is not a structural change)
- [I2] **Edge integrity**: Connected edges are not affected by node style changes

## Error Taxonomy
- **ReplayError::NodeNotFound** - The node ID in UpdateNodeStyle does not exist in the projection
- **ReplayError::InvalidOperation** - Operation variant is not handled

## Contract Signatures

### New apply_update_node_style function
```rust
pub fn apply_update_node_style(
    state: DiagramProjection,
    id: &str,
    style: NodeStyle,
) -> Result<DiagramProjection, ReplayError>
```

### dispatch_operation match arm
```rust
DomainOp::UpdateNodeStyle { id, style } => apply_update_node_style(state, id, style),
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: UpdateNodeStyle variant | Compile-time | Rust match exhaustiveness |
| P2: Node exists | Runtime | `state.nodes.get(id).is_some()` returning NodeNotFound if not |
| P3: Valid style | Compile-time | `enum NodeStyle` guarantees validity |

## Violation Examples

### Precondition Violations
- **VIOLATES P2**: Node doesn't exist
  - Input: apply_update_node_style(state, "n999", NodeStyle::Box) where n999 not in state.nodes
  - Expected: `Err(ReplayError::NodeNotFound)`

### Postcondition Violations
- **VIOLATES Q1**: Style not updated
  - Input: UpdateNodeStyle { id: "n1", style: NodeStyle::Cloud }
  - After: state.nodes["n1"].style != NodeStyle::Cloud
  - Expected: style equals Cloud

- **VIOLATES Q2**: Other fields changed
  - Input: UpdateNodeStyle { id: "n1", style: NodeStyle::Dashed }
  - After: state.nodes["n1"].x != original.x OR state.nodes["n1"].y != original.y
  - Expected: All position/dimension fields unchanged

## Ownership Contracts

- **state: DiagramProjection**: Passed by value, new projection returned (functional update)
- **id: &str**: Borrowed, no ownership transferred
- **style: NodeStyle**: Copy type, no ownership concerns

## Non-goals
- [ ] Batch style updates for multiple nodes
- [ ] Color changes (separate concern)
- [ ] Undo/redo (handled by event history)

---

## Implementation Phases

### Phase 1: Add apply_update_node_style
1. Create function in projection/ops/node_ops.rs or new file
2. Validate node exists, return NodeNotFound if not
3. Clone projection, update style field, return new projection
4. Ensure other fields are preserved (functional update pattern)

### Phase 2: Wire in dispatch_operation
1. Add match arm in dispatch_operation function
2. Destructure id and style from UpdateNodeStyle variant
3. Call apply_update_node_style

### Phase 3: Error Handling
1. Ensure ReplayError::NodeNotFound is defined
2. Map error appropriately in apply_operation wrapper
