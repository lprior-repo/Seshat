# Contract Specification

## Context
- **Feature**: Add UpdateNodeStyle variant to DomainOp enum
- **Bead**: seshat-2wb
- **Domain terms**:
  - `DomainOp` - enum of all diagram operations in envelope.rs
  - `NodeStyle` - enum (Box, Cloud, Cylinder, Dashed) representing node shape
  - `EventRecord` - persisted event containing DomainOp
  - `OpKind` - categorization (Node, Edge, ZOrder, Composite)
- **Assumptions**:
  - NodeStyle enum already exists in document.rs with variants: Box, Cloud, Cylinder, Dashed
  - DomainOp uses serde tag "op_type" for serialization
  - DomainOp::kind() method needs updating to return OpKind::Node for new variant
- **Open questions**:
  - Should UpdateNodeStyle also include color (hex string)? Based on current NodeStyle enum, no - just shape variants.

## EARS Requirements
- **Ubiquitous**: THE SYSTEM SHALL allow users to change node visual appearance (shape)
- **Event-driven**: WHEN a user selects a node and modifies its style via properties panel, THE SYSTEM SHALL dispatch UpdateNodeStyle to update the document
- **Unwanted**: IF an invalid style value is passed, THE SYSTEM SHALL NOT compile (type safety via enum)

## Preconditions
- [P1] **Valid NodeStyle**: The style field must be a valid NodeStyle variant (Box, Cloud, Cylinder, Dashed)
- [P2] **Valid NodeId**: The id field must be a non-empty string representing a valid node identifier

## Postconditions
- [Q1] **Enum variant exists**: DomainOp::UpdateNodeStyle variant is constructable with id and style fields
- [Q2] **Serialization works**: UpdateNodeStyle serializes to JSON with "op_type": "update_node_style"
- [Q3] **Deserialization works**: JSON with "op_type": "update_node_style" deserializes to DomainOp::UpdateNodeStyle
- [Q4] **Kind classification**: DomainOp::UpdateNodeStyle.kind() returns OpKind::Node

## Invariants
- [I1] **DomainOp completeness**: All DomainOp variants are handled in apply_operation dispatch
- [I2] **Serialization roundtrip**: Serializing then deserializing yields equivalent DomainOp

## Error Taxonomy
- **Error::InvalidVariant** - DomainOp tag does not match any known variant (for deserialization)
- **Error::ParseError** - JSON payload is malformed

## Contract Signatures

### New DomainOp Variant (to add in envelope.rs)
```rust
UpdateNodeStyle {
    id: String,
    style: NodeStyle,  // Box, Cloud, Cylinder, Dashed
}
```

### DomainOp::kind() Update
```rust
Self::UpdateNodeStyle { .. } => OpKind::Node,
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Valid NodeStyle | Compile-time | `enum NodeStyle { Box, Cloud, Cylinder, Dashed }` |
| P2: Valid NodeId | Runtime | Non-empty string check in constructor |

## Violation Examples

### Precondition Violations
- **VIOLATES P1**: Invalid style value passed to constructor
  - Input: style = "invalid_shape" (not a NodeStyle variant)
  - Expected: Compile-time error - Rust enum prevents invalid variants

- **VIOLATES P2**: Empty or invalid node id
  - Input: id = ""
  - Expected: `Err(Error::InvalidInput)` or precondition check returns false

### Postcondition Violations
- **VIOLATES Q3**: Deserialization with unknown op_type
  - Input: JSON `{"op_type": "update_node_style_unknown", "id": "n1", "style": "box"}`
  - Expected: `Err(serde_json::Error)` - unknown variant

- **VIOLATES Q4**: Kind returns wrong OpKind
  - Input: DomainOp::UpdateNodeStyle { id: "n1", style: NodeStyle::Box }.kind()
  - Expected: OpKind::Node (not Edge, ZOrder, or Composite)

## Ownership Contracts

- **DomainOp**: Cloneable value type (via Clone derive)
- **id field**: String - owned by the variant, cloned on serialization
- **style field**: NodeStyle - Copy type (enum), no ownership concerns

## Non-goals
- [ ] Edge style updates (separate DomainOp)
- [ ] Color changes (not in current NodeStyle enum)
- [ ] Batch node style updates (single node only)

---

## Implementation Phases

### Phase 1: Add DomainOp Variant
1. Add `UpdateNodeStyle { id: String, style: NodeStyle }` to DomainOp enum
2. Add `use crate::models::document::NodeStyle` import
3. Update derive macros: Clone, Serialize, Deserialize

### Phase 2: Update kind() Method
1. Add match arm: `Self::UpdateNodeStyle { .. } => OpKind::Node`

### Phase 3: Verify Serialization
1. Test JSON roundtrip: DomainOp -> JSON -> DomainOp
2. Verify op_type tag is "update_node_style"
