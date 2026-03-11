# Contract Specification

## Context
- **Feature**: Add serialization and projection unit tests for UpdateEdgeStyle DomainOp
- **Bead**: seshat-h9f
- **Domain terms**:
  - `DomainOp` - enum of all diagram operations
  - `EdgeStyle` - enum (Solid, Dashed, Dotted)
  - `DiagramProjection` - domain model
  - `apply_operation` - function applying DomainOp to projection
- **Assumptions**:
  - DomainOp::UpdateEdgeStyle variant exists (seshat-81d completed)
  - apply_update_edge_style function exists (seshat-aek completed)
- **Open questions**:
  - None - test scope is clear

## EARS Requirements
- **Ubiquitous**: THE SYSTEM SHALL have comprehensive test coverage for edge style operations
- **Event-driven**: WHEN tests execute, THE SYSTEM SHALL verify serialization and projection behavior
- **Unwanted**: IF tests pass with incorrect implementation, THE SYSTEM SHALL fail CI linting

## Preconditions
- [P1] **Test infrastructure ready**: Test module can import DomainOp, EdgeStyle, DiagramProjection
- [P2] **Fixture data available**: Test can construct valid DiagramProjection with edges

## Postconditions
- [Q1] **Serialization test passes**: DomainOp::UpdateEdgeStyle serializes to correct JSON
- [Q2] **Deserialization test passes**: JSON deserializes to DomainOp::UpdateEdgeStyle with correct fields
- [Q3] **Roundtrip test passes**: Serialize then deserialize yields equivalent DomainOp
- [Q4] **Projection test passes**: apply_operation with UpdateEdgeStyle updates edge style correctly
- [Q5] **Error test passes**: Apply to non-existent edge returns EdgeNotFound error

## Invariants
- [I1] **Test isolation**: Each test runs independently, no shared mutable state
- [I2] **Deterministic**: Same input always produces same output

## Error Taxonomy
- **Test failure**: Any assertion mismatch
- **ReplayError::EdgeNotFound** - Expected error for non-existent edge test

## Contract Signatures

### Test module location
```rust
// diagram_tool/src/models/projection/tests.rs
// or new file: diagram_tool/src/models/projection/update_edge_style_tests.rs
```

### Test functions to implement
```rust
#[test]
fn test_update_edge_style_serialization()

#[test]
fn test_update_edge_style_deserialization()

#[test]
fn test_update_edge_style_roundtrip()

#[test]
fn test_update_edge_style_projection()

#[test]
fn test_update_edge_style_edge_not_found()
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Imports available | Compile-time | Rust module system |
| P2: Valid fixtures | Compile-time | Inline test fixtures |

## Violation Examples

### Test Failures
- **VIOLATES Q1**: Wrong JSON output
  - Input: DomainOp::UpdateEdgeStyle { id: "e1", style: EdgeStyle::Solid }
  - Expected JSON: `{"op_type":"update_edge_style","id":"e1","style":"solid"}`
  - Actual: Different JSON format

- **VIOLATES Q4**: Style not applied
  - Input: apply_operation with UpdateEdgeStyle { id: "e1", style: EdgeStyle::Dashed }
  - Expected: projection.edges["e1"].style == EdgeStyle::Dashed
  - Actual: style unchanged

- **VIOLATES Q5**: No error for missing edge
  - Input: apply_operation with UpdateEdgeStyle { id: "e999", style: EdgeStyle::Dotted }
  - Expected: Err(ReplayError::EdgeNotFound)
  - Actual: Ok(projection) - silently succeeded

## Ownership Contracts

- **Test fixtures**: Owned by test functions, dropped after each test
- **Projection**: Cloned in each test for isolation

## Non-goals
- [ ] Integration tests with UI
- [ ] Performance/benchmarks
- [ ] Property-based testing

---

## Implementation Phases

### Phase 1: Serialization Tests
1. test_update_edge_style_serialization - verify JSON output format
2. test_update_edge_style_deserialization - verify parsing from JSON
3. test_update_edge_style_roundtrip - verify no data loss

### Phase 2: Projection Tests
1. test_update_edge_style_projection - apply op, verify style changed
2. test_update_edge_style_preserves_other_fields - verify no side effects
3. test_update_edge_style_edge_not_found - verify error handling

### Phase 3: Edge Case Tests
1. test_update_edge_style_all_variants - test all 3 EdgeStyle values
2. test_update_edge_style_idempotent - applying same style twice is no-op
