# Contract Specification

## Context
- **Feature**: Add UpdateEdgeStyle variant to DomainOp enum
- **Bead**: seshat-81d
- **Domain terms**:
  - `DomainOp` - enum of all diagram operations in envelope.rs
  - `EdgeStyle` - enum (Solid, Dashed, Dotted) representing edge line style
  - `EventRecord` - persisted event containing DomainOp
  - `OpKind` - categorization (Node, Edge, ZOrder, Composite)
- **Assumptions**:
  - EdgeStyle enum already exists in document.rs with variants: Solid, Dashed, Dotted
  - DomainOp uses serde tag "op_type" for serialization
  - DomainOp::kind() method needs updating to return OpKind::Edge for new variant
- **Open questions**:
  - Should UpdateEdgeStyle also include thickness? Current Edge struct has separate thickness field.

## EARS Requirements
- **Ubiquitous**: THE SYSTEM SHALL allow users to change edge visual appearance (line style)
- **Event-driven**: WHEN a user selects an edge and modifies its style via properties panel, THE SYSTEM SHALL dispatch UpdateEdgeStyle to update the document
- **Unwanted**: IF an invalid style value is passed, THE SYSTEM SHALL NOT compile (type safety via enum)

## Preconditions
- [P1] **Valid EdgeStyle**: The style field must be a valid EdgeStyle variant (Solid, Dashed, Dotted)
- [P2] **Valid EdgeId**: The id field must be a non-empty string representing a valid edge identifier

## Postconditions
- [Q1] **Enum variant exists**: DomainOp::UpdateEdgeStyle variant is constructable with id and style fields
- [Q2] **Serialization works**: UpdateEdgeStyle serializes to JSON with "op_type": "update_edge_style"
- [Q3] **Deserialization works**: JSON with "op_type": "update_edge_style" deserializes to DomainOp::UpdateEdgeStyle
- [Q4] **Kind classification**: DomainOp::UpdateEdgeStyle.kind() returns OpKind::Edge

## Invariants
- [I1] **DomainOp completeness**: All DomainOp variants are handled in apply_operation dispatch
- [I2] **Serialization roundtrip**: Serializing then deserializing yields equivalent DomainOp

## Error Taxonomy
- **Error::InvalidVariant** - DomainOp tag does not match any known variant (for deserialization)
- **Error::ParseError** - JSON payload is malformed

## Contract Signatures

### New DomainOp Variant (to add in envelope.rs)
```rust
UpdateEdgeStyle {
    id: String,
    style: EdgeStyle,  // Solid, Dashed, Dotted
}
```

### DomainOp::kind() Update
```rust
Self::UpdateEdgeStyle { .. } => OpKind::Edge,
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Valid EdgeStyle | Compile-time | `enum EdgeStyle { Solid, Dashed, Dotted }` |
| P2: Valid EdgeId | Runtime | Non-empty string check in constructor |

## Violation Examples

### Precondition Violations
- **VIOLATES P1**: Invalid style value passed to constructor
  - Input: style = "invalid_style" (not an EdgeStyle variant)
  - Expected: Compile-time error - Rust enum prevents invalid variants

- **VIOLATES P2**: Empty or invalid edge id
  - Input: id = ""
  - Expected: `Err(Error::InvalidInput)` or precondition check returns false

### Postcondition Violations
- **VIOLATES Q3**: Deserialization with unknown op_type
  - Input: JSON `{"op_type": "update_edge_style_unknown", "id": "e1", "style": "solid"}`
  - Expected: `Err(serde_json::Error)` - unknown variant

- **VIOLATES Q4**: Kind returns wrong OpKind
  - Input: DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed }.kind()
  - Expected: OpKind::Edge (not Node, ZOrder, or Composite)

## Ownership Contracts

- **DomainOp**: Cloneable value type (via Clone derive)
- **id field**: String - owned by the variant, cloned on serialization
- **style field**: EdgeStyle - Copy type (enum), no ownership concerns

## Non-goals
- [ ] Node style updates (separate DomainOp)
- [ ] Thickness changes (separate field on Edge struct)
- [ ] Batch edge style updates (single edge only)

---

## Implementation Phases

### Phase 1: Add DomainOp Variant
1. Add `UpdateEdgeStyle { id: String, style: EdgeStyle }` to DomainOp enum
2. Add `use crate::models::document::EdgeStyle` import
3. Update derive macros: Clone, Serialize, Deserialize

### Phase 2: Update kind() Method
1. Add match arm: `Self::UpdateEdgeStyle { .. } => OpKind::Edge`

### Phase 3: Verify Serialization
1. Test JSON roundtrip: DomainOp -> JSON -> DomainOp
2. Verify op_type tag is "update_edge_style"
