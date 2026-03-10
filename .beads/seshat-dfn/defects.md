# Code Review Defects - seshat-dfn

## Feature
Update JSON schema and UI/geometry (GEO-004) to allow customizing connection arrowhead types/directions, line sizes, and arrowhead sizes between connecting nodes, similar to Miro.

## Status: REJECTED

## Critical Defects

### 1. MISSING: `calculate_edge_bounds()` Function
**Contract**: Q3 requires "Geometric bounds calculation returns an Axis-Aligned Bounding Box (AABB) that fully encompasses the stroke width and all arrowhead geometries."

**Contract Signature**: `fn calculate_edge_bounds(&self) -> Result<AABB, Error>`

**Issue**: This function does not exist in the codebase. The contract explicitly requires this function but it was never implemented.

**Location**: Should be in `document.rs` on the `Edge` struct.

---

### 2. MISSING: P3 Validation (Arrowhead Fits on Line)
**Contract**: P3 states "Edge physical length must be strictly greater than the combined visual length of its rendered arrowheads."

**Error Type**: `Error::GeometryExceedsBounds` (line 364-365 in document.rs) exists but is never raised - there is no code that validates P3.

**Issue**: No validation that arrowhead size fits within the edge length. This is a contract requirement that was not implemented.

---

### 3. TYPE MISMATCH: PositiveFiniteFloat vs OrderedFloat
**Contract**: Lines 48-49 specify:
- P1 thickness: `PositiveFiniteFloat::new(val) -> Result<PositiveFiniteFloat, Error>`
- P2 arrowhead_size: `PositiveFiniteFloat::new(val) -> Result<PositiveFloat, Error>`

**Implementation**: Uses `OrderedFloat` which:
- ✅ Validates NaN and Infinity
- ❌ Does NOT validate positivity (allows negative values!)

**Location**: `document.rs:318-320` - Edge struct fields use `OrderedFloat`, not a positivity-enforcing newtype.

---

## Missing Tests

### Contract Violation Tests Not Implemented
The contract requires tests for these violation examples but none exist:

| Violation | Contract Line | Status |
|-----------|---------------|--------|
| VIOLATES P1 (Zero thickness) | Line 53 | ❌ MISSING |
| VIOLATES P1 (Negative thickness) | Line 54 | ❌ MISSING |
| VIOLATES P1 (NaN thickness) | Line 55 | ❌ MISSING |
| VIOLATES P1 (Infinity thickness) | Line 56 | ❌ MISSING |
| VIOLATES P2 (Zero arrowhead_size) | Line 57 | ❌ MISSING |
| VIOLATES P2 (Negative arrowhead_size) | Line 58 | ❌ MISSING |
| VIOLATES P2 (NaN arrowhead_size) | Line 59 | ❌ MISSING |
| VIOLATES P2 (Infinity arrowhead_size) | Line 60 | ❌ MISSING |
| VIOLATES P3 (arrowhead exceeds edge) | Line 61 | ❌ MISSING |
| VIOLATES Q2 (negative thickness in JSON) | Line 62 | ❌ MISSING |

---

## Partial Implementations

### Arrowhead Size Bucketing
**Location**: `canvas_view.rs:252-271`

The `edge_marker_ref_with_size` function buckets arrowhead sizes into 6 discrete values (8, 10, 12, 15, 18, 20). The contract does not specify this bucket strategy - is this intentional? The bucket boundaries (9.0, 11.0, 13.5, 16.5, 19.0) seem arbitrary.

---

## Summary

The implementation covers the UI/rendering aspects (dynamic markers, bidirectional arrowheads) but is missing:

1. **Core geometry calculation**: `calculate_edge_bounds()` function
2. **Validation**: P3 (arrowhead fits on line) is not enforced  
3. **Type safety**: Using `OrderedFloat` instead of positivity-enforcing newtypes
4. **Tests**: Zero contract violation tests exist

The code that exists is clean and follows functional patterns, but the contract is not fully satisfied.
