# Contract Specification: DomainOp NodeResize Tests

## Context
- **Feature**: Serialization and projection unit tests for NodeResize DomainOp
- **Domain terms**:
  - `NodeResize` - Operation to update node width/height
  - `Serialization` - JSON encode/decode
  - `Projection` - Apply operation to document state
- **Assumptions**:
  - NodeResize variant exists in DomainOp enum (seshat-fir)
  - Projection logic exists (seshat-c0j)
- **Open questions**: None

## EARS Requirements

| ID | Type | Requirement |
|----|------|-------------|
| EARS-1 | Ubiquitous | Tests SHALL verify JSON serialization roundtrip |
| EARS-2 | Event-Driven | Tests SHALL verify projection applies dimensions correctly |
| EARS-3 | Unwanted | Tests SHALL verify error cases don't panic |

## Preconditions

| ID | Description | Type Enforcement |
|----|-------------|------------------|
| P1 | Valid NodeResize operation | Compile-time: constructed correctly |
| P2 | Document with existing node | Runtime: node in doc.nodes |
| P3 | Valid dimensions (> 0, finite) | Runtime |

## Postconditions

| ID | Description |
|----|-------------|
| Q1 | JSON roundtrip preserves all fields exactly |
| Q2 | Projection updates node dimensions correctly |
| Q3 | All error cases return appropriate errors |
| Q4 | Test names describe behavior unambiguously |

## Invariants

| ID | Description |
|----|-------------|
| INV-1 | All existing tests still pass after adding NodeResize |
| INV-2 | Test coverage includes happy path and error paths |
| INV-3 | Tests are deterministic and isolated |

## Error Taxonomy

- Same as parent contracts: `ContractError`, `ProjectionError`

## Contract Signatures

```rust
// Test module signatures
#[cfg(test)]
mod tests {
    fn given_valid_node_resize_json_when_parsing_then_returns_domain_op();
    fn given_valid_node_resize_when_encoding_then_roundtrip_works();
    fn given_node_resize_projection_when_applying_then_updates_dimensions();
    fn given_node_resize_with_invalid_dimensions_then_returns_error();
}
```

## Type Encoding

Same as parent contracts.

## Violation Examples (REQUIRED)

- **VIOLATES Q1**: Roundtrip test fails if width/height not preserved exactly
- **VIOLATES Q2**: Projection test fails if dimensions not applied
- **VIOLATES Q3**: Error test fails if wrong error type returned

## Ownership Contracts

- Tests use owned values for construction
- No mutable state between tests (isolation)

## Non-goals

- [ ] Integration tests with full system
- [ ] Performance/benchmark tests
- [ ] Fuzz testing
