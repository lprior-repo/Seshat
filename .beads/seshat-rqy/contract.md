# Contract Specification: DomainOp UpdateLabel Enum

## Context
- **Feature**: Add `UpdateLabel` variant to `DomainOp` enum for text label updates
- **Domain terms**:
  - `DomainOp` - Operation type in event envelope
  - `UpdateLabel` - Operation to update node/edge label text
  - `OpKind::Node` - Classification for node operations
- **Assumptions**:
  - Existing DomainOp pattern is established
  - Labels support full Unicode including RTL
- **Open questions**: Should UpdateLabel work for both nodes and edges?

## EARS Requirements

| ID | Type | Requirement |
|----|------|-------------|
| EARS-1 | Ubiquitous | The system SHALL persist text label edits to the event log |
| EARS-2 | Event-Driven | When an UpdateLabel event is projected, the system SHALL replace the target's label |
| EARS-3 | Unwanted | If label is not valid UTF-8, the system SHALL NOT accept it |
| EARS-4 | Unwanted | If target ID is empty, the system SHALL NOT accept it |

## Preconditions

| ID | Description | Type Enforcement |
|----|-------------|------------------|
| P1 | Valid JSON input | Runtime - `ContractError::InvalidJson` |
| P2 | Contains "op" field with value "update_label" | Runtime - `ContractError::UnknownOpType` |
| P3 | Contains "id" field (non-empty string) | Runtime - `ContractError::MissingField` |
| P4 | Contains "label" field (valid UTF-8 string) | Runtime - `ContractError::InvalidPayload` |

## Postconditions

| ID | Description |
|----|-------------|
| Q1 | Returns `Ok(DomainOp::UpdateLabel { id, label })` with exact input values |
| Q2 | `kind()` returns `OpKind::Node` |
| Q3 | Serialization roundtrip preserves label exactly |
| Q4 | Exhaustive match in `kind()` method covers UpdateLabel |
| Q5 | `parse_domain_op` recognizes "update_label" op type |

## Invariants

| ID | Description |
|----|-------------|
| INV-1 | DomainOp enum remains exhaustive - new variant requires test update |
| INV-2 | All DomainOp variants must implement Clone, Debug, PartialEq, Serialize, Deserialize |
| INV-3 | OpKind classification is consistent: UpdateLabel -> Node |

## Error Taxonomy

- `ContractError::InvalidJson(String)` - JSON parse failure with details
- `ContractError::MissingField(&'static str)` - Required field absent
- `ContractError::UnknownOpType(String)` - Unrecognized operation type
- `ContractError::InvalidPayload(String)` - Invalid label values (non-UTF-8)

## Contract Signatures

```rust
// In models/envelope.rs
pub enum DomainOp {
    UpdateLabel {
        id: String,
        label: String,
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
| label valid UTF-8 | Runtime validation | String in Rust is always UTF-8 |

## Violation Examples (REQUIRED)

- **VIOLATES P2**: `parse_domain_op(r#"{"op": "update_lable", "id": "n1", "label": "New"}"#)` -- should produce `Err(ContractError::UnknownOpType("update_lable"))`
- **VIOLATES P3**: `parse_domain_op(r#"{"op": "update_label", "label": "New"}"#)` -- should produce `Err(ContractError::MissingField("id"))`
- **VIOLATES P3**: `parse_domain_op(r#"{"op": "update_label", "id": "", "label": "New"}"#)` -- should produce `Err(ContractError::InvalidPayload(...))` (empty ID)
- **VIOLATES Q1**: `DomainOp::UpdateLabel { id: "n1".to_string(), label: "New".to_string() }` -- label should be exactly preserved after serialization
- **VIOLATES Q2**: `DomainOp::UpdateLabel { .. }.kind()` -- should return `OpKind::Node`, not Edge or Composite

## Ownership Contracts

- `DomainOp::UpdateLabel` owns its `id` and `label` fields (owned String)
- No borrowing - all fields are owned types
- Clone is derived - intentional for event duplication in replay scenarios

## Non-goals

- [ ] Label length limits (UI concern, not domain)
- [ ] Rich text formatting (plain text only)
- [ ] Label validation against max length
