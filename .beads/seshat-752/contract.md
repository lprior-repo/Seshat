# Contract Specification - seshat-752

**bead_id:** seshat-752  
**bead_title:** domain-bound: Replace String with NodeId in DomainOp  
**phase:** Contract Synthesis  
**updated_at:** 2026-03-12T22:07:00Z

---

## Preconditions

### P1: Valid JSON Input
All DomainOp JSON inputs MUST be valid JSON before parsing.

### P2: Non-empty Entity IDs  
Entity ID fields MUST be non-empty strings before conversion to NewType.

### P3: Known OpType
The `op_type` field MUST match a known DomainOp variant.

---

## Postconditions

### Q1: NodeId Types (KEY)
After successful parsing, all entity identifier fields MUST be of type `NodeId` or `EdgeId`, NOT `String`.

**Evidence:** `UpdateLabel.target_id` becomes `NodeId` instead of `String`.

### Q2: Valid NewType Wrappers
All NodeId/EdgeId wrappers MUST encapsulate non-empty, valid strings.

### Q3: Roundtrip Serialization
DomainOp MUST preserve equality through JSON serialize → deserialize cycle.

---

## Invariants

### I1: No Naked Strings
No naked `String` type exists for entity identifiers in DomainOp.

### I2: Non-empty NodeId
NodeId MUST always wrap non-empty strings (enforced at construction). Whitespace-only strings are also rejected.

### III3: Errors Not Panics
Parsing failures return `Result<T, ContractError>`, never panic.

---

## Error Taxonomy

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ContractError {
    InvalidJson(String),
    MissingField(String),
    UnknownOpType(String),
    InvalidNodeId(String),  // Added for empty/invalid IDs
    InvalidPayload(String),
}
```

---

## Violation Examples

| Precondition | Invalid Input | Expected Error |
|--------------|---------------|-----------------|
| P1 | `{invalid json}` | `ContractError::InvalidJson` |
| P2 | `"target_id": ""` | `ContractError::InvalidNodeId` |
| P3 | `"op_type": "unknown"` | `ContractError::UnknownOpType` |

---

## Implementation Notes

### Type Change Required
```rust
// BEFORE (violates Q1, I1)
UpdateLabel {
    target_id: String,  // ❌ VIOLATION
    ...
}

// AFTER (satisfies Q1, I1)  
UpdateLabel {
    target_id: NodeId,   // ✅ COMPLIANT
    ...
}
```

### NodeId NewType
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractError> {
        let s = id.into();
        if s.is_empty() || s.trim().is_empty() {
            Err(ContractError::InvalidNodeId(s))
        } else {
            Ok(NodeId(s))
        }
    }
}
```
