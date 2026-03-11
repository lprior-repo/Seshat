# Contract Specification: DomainOp NodeResize Enum

## Context
- **Feature**: Add `NodeResize` variant to `DomainOp` enum for node dimension changes
- **Domain terms**: 
  - `DomainOp` - Operation type in event envelope
  - `NodeResize` - Operation to update node width/height
  - `OpKind::Node` - Classification for node operations
- **Assumptions**: 
  - Existing DomainOp pattern is established
  - JSON serialization uses serde with `#[serde(tag = "op_type")]`
- **Open questions**: None

## EARS Requirements

| ID | Type | Requirement |
|----|------|-------------|
| EARS-1 | Ubiquitous | The system SHALL persist node dimension changes to the event log |
| EARS-2 | Event-Driven | When a NodeResize event is projected, the system SHALL update the target node's width and height |
| EARS-3 | Unwanted | If width or height is <= 0 or NaN/Infinity, the system SHALL NOT apply the projection |

## Preconditions

| ID | Description | Type Enforcement |
|----|-------------|------------------|
| P1 | Valid JSON input | Runtime - `ContractError::InvalidJson` |
| P2 | Contains "op" field with value "node_resize" | Runtime - `ContractError::UnknownOpType` |
| P3 | Contains "id" field (non-empty string) | Runtime - `ContractError::MissingField` |
| P4 | Contains "width" field (valid f64, > 0, finite) | Runtime - `ContractError::InvalidPayload` |
| P5 | Contains "height" field (valid f64, > 0, finite) | Runtime - `ContractError::InvalidPayload` |

## Postconditions

| ID | Description |
|----|-------------|
| Q1 | Returns `Ok(DomainOp::NodeResize { id, width, height })` with exact input values |
| Q2 | `kind()` returns `OpKind::Node` |
| Q3 | Serialization roundtrip preserves all field values exactly |
| Q4 | Exhaustive match in `kind()` method covers NodeResize |
| Q5 | `parse_domain_op` recognizes "node_resize" op type |

## Invariants

| ID | Description |
|----|-------------|
| INV-1 | DomainOp enum remains exhaustive - new variant requires test update |
| INV-2 | All DomainOp variants must implement Clone, Debug, PartialEq, Serialize, Deserialize |
| INV-3 | OpKind classification is consistent: NodeResize -> Node |

## Error Taxonomy

- `ContractError::InvalidJson(String)` - JSON parse failure with details
- `ContractError::MissingField(&'static str)` - Required field absent  
- `ContractError::UnknownOpType(String)` - Unrecognized operation type
- `ContractError::InvalidPayload(String)` - Invalid width/height values

## Contract Signatures

```rust
// In models/envelope.rs
pub enum DomainOp {
    NodeResize {
        id: String,
        width: f64,
        height: f64,
    },
}

impl DomainOp {
    pub const fn kind(&self) -> OpKind;
}

pub fn parse_domain_op(raw: &str) -> Result<DomainOp, ContractError>;
pub const fn domain_op_kind(op: &DomainOp) -> OpKind;
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|--------------|-------------------|----------------|
| id non-empty | Runtime constructor | Check `!id.is_empty()` in parser |
| width > 0, finite | Runtime validation | `width.is_finite() && width > 0.0` |
| height > 0, finite | Runtime validation | `height.is_finite() && height > 0.0` |

## Violation Examples (REQUIRED)

- **VIOLATES P2**: `parse_domain_op(r#"{"op": "node_rezise", "id": "n1", "width": 80.0, "height": 40.0}"#)` -- should produce `Err(ContractError::UnknownOpType("node_rezise"))`
- **VIOLATES P3**: `parse_domain_op(r#"{"op": "node_resize", "width": 80.0, "height": 40.0}"#)` -- should produce `Err(ContractError::MissingField("id"))`
- **VIOLATES P4**: `parse_domain_op(r#"{"op": "node_resize", "id": "n1", "width": -10.0, "height": 40.0}"#)` -- should produce `Err(ContractError::InvalidPayload(...))`
- **VIOLATES P4**: `parse_domain_op(r#"{"op": "node_resize", "id": "n1", "width": 0.0, "height": 40.0}"#)` -- should produce `Err(ContractError::InvalidPayload(...))`
- **VIOLATES P4**: `parse_domain_op(r#"{"op": "node_resize", "id": "n1", "width": NaN, "height": 40.0}"#)` -- should produce `Err(ContractError::InvalidPayload(...))`
- **VIOLATES P5**: Same as P4 but for height field
- **VIOLATES Q1**: `DomainOp::NodeResize { id: "n1".to_string(), width: 80.0, height: 40.0 }` -- width/height should be exactly preserved after serialization
- **VIOLATES Q2**: `DomainOp::NodeResize { .. }.kind()` -- should return `OpKind::Node`, not Edge or Composite

## Ownership Contracts

- `DomainOp::NodeResize` owns its `id`, `width`, `height` fields (owned String, Copy f64)
- No borrowing - all fields are owned types
- Clone is derived - intentional for event duplication in replay scenarios

## Non-goals

- [ ] Handling multi-node resize (use Group operation)
- [ ] Aspect ratio preservation (UI concern, not domain)
- [ ] Minimum/maximum dimension enforcement (UI validation)
