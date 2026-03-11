# Contract Specification: DomainOp UpdateLabel Tests

## Context
- **Feature**: Serialization and projection unit tests for UpdateLabel DomainOp
- **Domain terms**:
  - `UpdateLabel` - Operation to update node/edge label text
  - `Serialization` - JSON encode/decode
  - `Projection` - Apply operation to document state
- **Assumptions**:
  - UpdateLabel variant exists in DomainOp enum (seshat-rqy)
  - Projection logic exists (seshat-8tj)
- **Open questions**: None

## EARS Requirements

| ID | Type | Requirement |
|----|------|-------------|
| EARS-1 | Ubiquitous | Tests SHALL verify JSON serialization roundtrip |
| EARS-2 | Event-Driven | Tests SHALL verify projection applies label correctly |
| EARS-3 | Unwanted | Tests SHALL verify error cases don't panic |
| EARS-4 | Event-Driven | Tests SHALL verify Unicode/RTL label handling |

## Preconditions

| ID | Description | Type Enforcement |
|----|-------------|------------------|
| P1 | Valid UpdateLabel operation | Compile-time: constructed correctly |
| P2 | Document with existing node | Runtime: node in doc.nodes |
| P3 | Valid UTF-8 label | Compile-time: String guarantees |

## Postconditions

| ID | Description |
|----|-------------|
| Q1 | JSON roundtrip preserves label exactly (including Unicode) |
| Q2 | Projection updates node label correctly |
| Q3 | All error cases return appropriate errors |
| Q4 | Test names describe behavior unambiguously |
| Q5 | Empty label is valid (clears label) |

## Invariants

| ID | Description |
|----|-------------|
| INV-1 | All existing tests still pass after adding UpdateLabel |
| INV-2 | Test coverage includes happy path and error paths |
| INV-3 | Tests are deterministic and isolated |
| INV-4 | Unicode and RTL characters preserved in roundtrip |

## Error Taxonomy

- Same as parent contracts: `ContractError`, `ProjectionError`

## Contract Signatures

```rust
// Test module signatures
#[cfg(test)]
mod tests {
    fn given_valid_update_label_json_when_parsing_then_returns_domain_op();
    fn given_valid_update_label_when_encoding_then_roundtrip_works();
    fn given_update_label_projection_when_applying_then_updates_label();
    fn given_update_label_with_nonexistent_target_then_returns_error();
    fn given_update_label_with_unicode_then_preserves_characters();
    fn given_update_label_with_rtl_text_then_preserves_characters();
    fn given_update_label_with_empty_string_then_clears_label();
}
```

## Type Encoding

Same as parent contracts.

## Violation Examples (REQUIRED)

- **VIOLATES Q1**: Roundtrip test fails if label not preserved exactly
- **VIOLATES Q2**: Projection test fails if label not applied
- **VIOLATES Q3**: Error test fails if wrong error type returned
- **VIOLATES Q4**: Unicode/RTL test fails if characters not preserved

## Ownership Contracts

- Tests use owned values for construction
- No mutable state between tests (isolation)

## Non-goals

- [ ] Integration tests with full system
- [ ] Performance/benchmark tests
- [ ] Fuzz testing
- [ ] Edge label updates (separate feature)
