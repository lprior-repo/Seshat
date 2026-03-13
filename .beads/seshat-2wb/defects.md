# Black Hat Defect Report: seshat-2wb

## Review Date: 2026-03-12

## New Critical Defects Found

### DEFECT: Silent Default Instead of Parse Error (Lines 422-428)

**Severity**: CRITICAL
**Phase**: PHASE 3 - NASA-Level Functional Rust (The Big 6)
**Contract Clause**: P1 (Parse, Don't Validate)

**Description**: 
The `parse_update_node_style` function silently defaults invalid style values to `NodeStyle::Box` instead of returning an error. This violates the "Parse, Don't Validate" principle.

**Location**: `diagram_tool/src/models/envelope.rs:422-428`

**Current Code**:
```rust
let style = match style_str {
    "box" => NodeStyle::Box,
    "cloud" => NodeStyle::Cloud,
    "cylinder" => NodeStyle::Cylinder,
    "dashed" => NodeStyle::Dashed,
    _ => NodeStyle::Box,  // BUG: Silent default!
};
```

**Impact**:
- Invalid style strings like `"invalid_style"` or `"rectangle"` are accepted but produce incorrect behavior
- Creates a gap between compile-time type safety (enum) and runtime parsing
- Violates contract precondition P1 enforcement

**Fix Required**:
```rust
_ => return Err(ContractError::InvalidPayload(format!("unknown NodeStyle: {style_str}"))),
```

---

## Previous Defects (Already Fixed)

### P2: Missing test for empty ID validation

**Severity**: Medium
**Contract Clause**: P2 (Valid NodeId: The id field must be a non-empty string)
**Test Coverage**: Missing

**Description**: 
The UpdateNodeStyle operation was missing a test to verify that empty IDs are rejected at parse time. While the validation logic existed in `parse_update_node_style`, there was no test coverage.

**Evidence**:
- NodeResize had `given_node_resize_with_empty_id_when_parsing_then_returns_invalid_payload_error`
- UpdateLabel had `given_update_label_with_empty_id_returns_error`
- UpdateNodeStyle had no equivalent test

**Fix Applied**:
- Added test `given_update_node_style_with_empty_id_returns_error` in `diagram_tool/src/models/envelope.rs`
- Test verifies that `parse_domain_op` returns `ContractError::InvalidPayload` when id is empty string

---

### Incomplete exhaustive match test array

**Severity**: High
**Contract Clause**: I1 (DomainOp completeness)
**Test Coverage**: Incomplete

**Description**: 
The exhaustive match test `given_all_domain_op_variants_exhaustive_match_then_all_cases_handled` had the match arms for UpdateLabel and UpdateNodeStyle, but the variants array used to verify coverage was missing both of these variants.

**Evidence**:
- Match arms at lines 1671-1673 included: UpdateLabel, UpdateNodeStyle, UpdateEdgeStyle
- Variants array ended at UpdateEdgeStyle, missing UpdateLabel and UpdateNodeStyle

**Fix Applied**:
- Added `DomainOp::UpdateLabel { id: "n1".to_string(), label: "test".to_string() }` to variants array
- Added `DomainOp::UpdateNodeStyle { id: "n1".to_string(), style: NodeStyle::Box }` to variants array

---

## Files Modified

- `diagram_tool/src/models/envelope.rs` - Added tests and fixed variants array

## Verification

- Test `given_update_node_style_with_empty_id_returns_error` added and validates P2
- Test `given_all_domain_op_variants_exhaustive_match_then_all_cases_handled` now has complete coverage
