# Implementation Summary: seshat-o8u

## Black Hat Defect Fix

### Issue
- **Contract specifies**: `ReplayError::NodeNotFound` for non-existent node error
- **Implementation used**: `ReplayError::InvariantViolation`
- **Contract clause**: Q5 - Apply to non-existent node returns NodeNotFound error

### Fix Applied
Changed error type in `apply_update_node_style` function:
- **File**: `diagram_tool/src/models/projection/ops/node_ops.rs`
- **Line**: 421 (was 421, now 417-421)
- **Before**: `ReplayError::InvariantViolation(format!("node not found: {id}"))`
- **After**: `ReplayError::NodeNotFound(format!("node not found: {id}"))`

Also added `UpdateNodeStyle` case to `project_operation` to properly handle the error:
- Added case in `project_operation` to route `DomainOp::UpdateNodeStyle` to `apply_update_node_style`
- Error mapping: `ReplayError::NodeNotFound` → `ProjectionError::NodeNotFound`

## Contract Adherence

### Files Changed
- `diagram_tool/src/models/envelope.rs` - Added serialization tests
- `diagram_tool/src/models/projection/tests.rs` - Added projection tests
- `diagram_tool/src/models/projection/ops/node_ops.rs` - **Fixed error type** (Black Hat defect)

### Contract Clauses Verified

| Clause | Implementation |
|--------|----------------|
| Q1: Serialization test passes | given_update_node_style_serialization_produces_correct_json - verifies JSON format |
| Q2: Deserialization test passes | given_update_node_style_parse_from_json - verifies parsing |
| Q3: Roundtrip test passes | given_update_node_style_serialization_roundtrip_preserves_style - verify no data loss |
| Q4: Projection test passes | given_update_node_style_when_applying_then_updates_style - verifies style change |
| Q5: Error test passes | given_update_node_style_nonexistent_node_returns_error - verifies error handling |
| P1: Test infrastructure ready | Tests import DomainOp, NodeStyle, DiagramProjection |
| P2: Fixture data available | Inline test fixtures create valid DiagramProjection with nodes |
| I1: Test isolation | Each test uses separate state via clone() |
| I2: Deterministic | Same input produces same output |

## Tests Added

### Serialization Tests (envelope.rs)
- `given_update_node_style_serialization_roundtrip_preserves_style` - Full roundtrip test
- `given_update_node_style_all_style_variants_serialize_correctly` - Tests all 4 variants
- `given_update_node_style_parse_from_json` - Parsing from JSON
- `given_update_node_style_invalid_style_returns_error` - Error handling for invalid style
- `given_update_node_style_missing_style_returns_error` - Error handling for missing field
- `given_update_node_style_kind_returns_node` - Verifies OpKind::Node
- `given_update_node_style_serialization_produces_correct_json` - Verifies JSON format

### Projection Tests (tests.rs)
- `given_update_node_style_when_applying_then_updates_style` - Happy path
- `given_update_node_style_then_preserves_other_fields` - Verifies no side effects
- `given_update_node_style_all_variants_work` - Tests all 4 style variants
- `given_update_node_style_nonexistent_node_returns_error` - Error handling
- `given_update_node_style_idempotent` - Idempotency verification

## Test Results
All 12 UpdateNodeStyle-specific tests pass:
```
running 12 tests
test models::envelope::tests::given_update_node_style_all_style_variants_serialize_correctly ... ok
test models::envelope::tests::given_update_node_style_invalid_style_returns_error ... ok
test models::envelope::tests::given_update_node_style_kind_returns_node ... ok
test models::envelope::tests::given_update_node_style_missing_style_returns_error ... ok
test models::envelope::tests::given_update_node_style_parse_from_json ... ok
test models::envelope::tests::given_update_node_style_serialization_produces_correct_json ... ok
test models::envelope::tests::given_update_node_style_serialization_roundtrip_preserves_style ... ok
test models::projection::tests::tests::given_update_node_style_all_variants_work ... ok
test models::projection::tests::tests::given_update_node_style_idempotent ... ok
test models::projection::tests::tests::given_update_node_style_nonexistent_node_returns_error ... ok
test models::projection::tests::tests::given_update_node_style_then_preserves_other_fields ... ok
test models::projection::tests::tests::given_update_node_style_when_applying_then_updates_style ... ok
```

## Verification
All 1548 lib tests pass.
