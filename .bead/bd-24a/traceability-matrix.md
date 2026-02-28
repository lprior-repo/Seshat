# Traceability Matrix: Grid Core Module (bd-24a)

## Overview

This matrix traces requirements through tests to implementation, ensuring complete coverage and no orphan code.

---

## Requirements → Tests → Implementation

### REQ-1: GridSize Newtype with Validation

| Requirement ID | Description | Test IDs | Implementation Location |
|----------------|-------------|----------|------------------------|
| REQ-1.1 | GridSize wraps f64 with private field | T-HAPPY-01, T-CONTRACT-01 | `ui/grid/mod.rs:GridSize` struct |
| REQ-1.2 | Validation range [10.0, 100.0] | T-ERROR-01, T-ERROR-02, T-ERROR-03, T-ERROR-04, T-PROP-01 | `GridSize::new()` |
| REQ-1.3 | Reject NaN values | T-ERROR-05, T-CONTRACT-02 | `GridSize::new()` |
| REQ-1.4 | Reject Infinity values | T-ERROR-06, T-ERROR-07 | `GridSize::new()` |
| REQ-1.5 | Default value is 20.0 | T-HAPPY-04, T-CONTRACT-05 | `GridSize::default()` |
| REQ-1.6 | Access inner value via .inner() | T-HAPPY-01, T-CONTRACT-03 | `GridSize::inner()` |

### REQ-2: Serialization/Deserialization

| Requirement ID | Description | Test IDs | Implementation Location |
|----------------|-------------|----------|------------------------|
| REQ-2.1 | Serialize as raw f64 | T-CONTRACT-04 | `#[serde(into = "f64")]` |
| REQ-2.2 | Deserialize from raw f64 | T-HAPPY-08 | `#[serde(try_from = "f64")]` |
| REQ-2.3 | Reject non-number JSON | T-ERROR-08, T-ERROR-10 | Serde error handling |
| REQ-2.4 | Reject out-of-range JSON | T-ERROR-09 | `TryFrom<f64>` impl |
| REQ-2.5 | Round-trip preserves value | T-PROP-06 | `Serialize` + `Deserialize` |

### REQ-3: snap_value Function

| Requirement ID | Description | Test IDs | Implementation Location |
|----------------|-------------|----------|------------------------|
| REQ-3.1 | Identity when snap disabled | T-HAPPY-05, T-CONTRACT-02, T-PROP-04 | `snap_value()` early return |
| REQ-3.2 | Round to grid multiple when enabled | T-HAPPY-06, T-CONTRACT-03, T-PROP-03 | `snap_value()` calculation |
| REQ-3.3 | Treat zero/negative grid as 1.0 | T-EDGE-01, T-EDGE-02 | `.max(1.0)` in function |
| REQ-3.4 | Propagate NaN | T-EDGE-03 | Natural f64 behavior |
| REQ-3.5 | Preserve Infinity | T-EDGE-04 | Natural f64 behavior |
| REQ-3.6 | Exact multiple unchanged | T-EDGE-06 | Math: `round(x/g)*g == x` |
| REQ-3.7 | Midpoint rounds up | T-EDGE-07 | `.round()` behavior |
| REQ-3.8 | Handle negative values | T-EDGE-08 | Math handles naturally |

### REQ-4: snap_point Function

| Requirement ID | Description | Test IDs | Implementation Location |
|----------------|-------------|----------|------------------------|
| REQ-4.1 | Snap each coordinate independently | T-HAPPY-07 | `snap_point()` tuple construction |
| REQ-4.2 | Consistent with snap_value | T-PROP-05 | Delegates to `snap_value` |

### REQ-5: EditorState Integration

| Requirement ID | Description | Test IDs | Implementation Location |
|----------------|-------------|----------|------------------------|
| REQ-5.1 | Replace OrderedFloat with GridSize | T-HAPPY-08 (integration) | `models/document.rs:EditorState` |
| REQ-5.2 | Backward compatible JSON | T-PROP-06 | Serde config |
| REQ-5.3 | Default grid_size unchanged | T-HAPPY-04 | `EditorState::default()` |

### REQ-6: Error Taxonomy

| Requirement ID | Description | Test IDs | Implementation Location |
|----------------|-------------|----------|------------------------|
| REQ-6.1 | GridError::OutOfRange variant | T-ERROR-01..04 | `ui/grid/mod.rs:GridError` |
| REQ-6.2 | GridError::NotFinite variant | T-ERROR-05..07 | `ui/grid/mod.rs:GridError` |
| REQ-6.3 | GridError::InvalidType variant | T-ERROR-08 | `ui/grid/mod.rs:GridError` |
| REQ-6.4 | Error messages are descriptive | All error tests | Error Display impl |

---

## Test ID Reference

### Happy Path Tests (T-HAPPY-*)

| ID | Test Name | File |
|----|-----------|------|
| T-HAPPY-01 | `given_valid_value_when_creating_grid_size_then_returns_ok` | `ui/grid/mod.rs` |
| T-HAPPY-02 | `given_minimum_value_when_creating_grid_size_then_returns_ok` | `ui/grid/mod.rs` |
| T-HAPPY-03 | `given_maximum_value_when_creating_grid_size_then_returns_ok` | `ui/grid/mod.rs` |
| T-HAPPY-04 | `given_default_when_getting_default_grid_size_then_returns_20` | `ui/grid/mod.rs` |
| T-HAPPY-05 | `given_snap_disabled_when_snapping_value_then_returns_value_unchanged` | `ui/grid/mod.rs` |
| T-HAPPY-06 | `given_snap_enabled_when_snapping_value_then_returns_grid_multiple` | `ui/grid/mod.rs` |
| T-HAPPY-07 | `given_point_when_snapping_then_each_coordinate_snapped_independently` | `ui/grid/mod.rs` |
| T-HAPPY-08 | `given_valid_json_number_when_deserializing_grid_size_then_succeeds` | `models/document.rs` |

### Error Path Tests (T-ERROR-*)

| ID | Test Name | File |
|----|-----------|------|
| T-ERROR-01 | `given_value_below_minimum_when_creating_grid_size_then_returns_out_of_range_error` | `ui/grid/mod.rs` |
| T-ERROR-02 | `given_value_above_maximum_when_creating_grid_size_then_returns_out_of_range_error` | `ui/grid/mod.rs` |
| T-ERROR-03 | `given_negative_value_when_creating_grid_size_then_returns_out_of_range_error` | `ui/grid/mod.rs` |
| T-ERROR-04 | `given_zero_value_when_creating_grid_size_then_returns_out_of_range_error` | `ui/grid/mod.rs` |
| T-ERROR-05 | `given_nan_value_when_creating_grid_size_then_returns_not_finite_error` | `ui/grid/mod.rs` |
| T-ERROR-06 | `given_positive_infinity_when_creating_grid_size_then_returns_not_finite_error` | `ui/grid/mod.rs` |
| T-ERROR-07 | `given_negative_infinity_when_creating_grid_size_then_returns_not_finite_error` | `ui/grid/mod.rs` |
| T-ERROR-08 | `given_json_string_when_deserializing_grid_size_then_returns_invalid_type_error` | `ui/grid/mod.rs` |
| T-ERROR-09 | `given_out_of_range_json_number_when_deserializing_then_returns_out_of_range_error` | `ui/grid/mod.rs` |
| T-ERROR-10 | `given_json_null_when_deserializing_grid_size_then_returns_error` | `ui/grid/mod.rs` |

### Edge Case Tests (T-EDGE-*)

| ID | Test Name | File |
|----|-----------|------|
| T-EDGE-01 | `given_zero_grid_size_when_snapping_then_uses_one_as_fallback` | `ui/grid/mod.rs` |
| T-EDGE-02 | `given_negative_grid_size_when_snapping_then_uses_one_as_fallback` | `ui/grid/mod.rs` |
| T-EDGE-03 | `given_nan_value_when_snapping_then_returns_nan` | `ui/grid/mod.rs` |
| T-EDGE-04 | `given_infinity_value_when_snapping_then_returns_infinity` | `ui/grid/mod.rs` |
| T-EDGE-05 | `given_nan_grid_size_when_snapping_then_uses_one_as_fallback` | `ui/grid/mod.rs` |
| T-EDGE-06 | `given_exact_grid_multiple_when_snapping_then_returns_same_value` | `ui/grid/mod.rs` |
| T-EDGE-07 | `given_value_midway_between_grid_lines_when_snapping_then_rounds_to_nearest` | `ui/grid/mod.rs` |
| T-EDGE-08 | `given_negative_value_when_snapping_then_handles_correctly` | `ui/grid/mod.rs` |

### Contract Tests (T-CONTRACT-*)

| ID | Test Name | File |
|----|-----------|------|
| T-CONTRACT-01 | `test_precondition_p1_range_validation` | `ui/grid/mod.rs` |
| T-CONTRACT-02 | `test_precondition_p1_finite_validation` | `ui/grid/mod.rs` |
| T-CONTRACT-03 | `test_postcondition_q1_inner_value_preserved` | `ui/grid/mod.rs` |
| T-CONTRACT-04 | `test_postcondition_q4_serialization_format` | `ui/grid/mod.rs` |
| T-CONTRACT-05 | `test_postcondition_q5_default_value` | `ui/grid/mod.rs` |
| T-CONTRACT-06 | `test_invariant_i1_range_guaranteed` | `ui/grid/mod.rs` |
| T-CONTRACT-07 | `test_invariant_i2_finite_guaranteed` | `ui/grid/mod.rs` |

### Property Tests (T-PROP-*)

| ID | Test Name | File |
|----|-----------|------|
| T-PROP-01 | `prop_grid_size_invariant_range` | `ui/grid/mod.rs` |
| T-PROP-02 | `prop_snap_idempotency` | `ui/grid/mod.rs` |
| T-PROP-03 | `prop_snap_grid_alignment` | `ui/grid/mod.rs` |
| T-PROP-04 | `prop_snap_disabled_identity` | `ui/grid/mod.rs` |
| T-PROP-05 | `prop_snap_point_consistent_with_snap_value` | `ui/grid/mod.rs` |
| T-PROP-06 | `prop_serialization_roundtrip` | `ui/grid/mod.rs` |

---

## Implementation Files

### New Files to Create

| File | Purpose | Contents |
|------|---------|----------|
| `ui/grid/mod.rs` | Grid module root | `GridSize`, `GridError`, `snap_value`, `snap_point`, `validated_grid_size` |

### Files to Modify

| File | Change | Affected Code |
|------|--------|---------------|
| `models/document.rs` | Replace `OrderedFloat` with `GridSize` in `EditorState.grid_size` | Line 271 |
| `ui/interaction.rs` | Move snap functions to grid module, add re-exports | Lines 97-112 |
| `ui/canvas.rs` | Update imports and use `.inner()` for grid_size | Lines 202, 381, 435, 436, 1041, 1044, 1143, 1144 |
| `ui/mod.rs` | Add `pub mod grid;` | Module declaration |

---

## Coverage Verification Checklist

- [x] Every requirement has at least one test
- [x] Every error variant has a test that triggers it
- [x] Every boundary condition has a test
- [x] Every invariant has a property test
- [x] Every serialization path has a roundtrip test
- [x] Every function has happy + error + edge tests

---

## Violation-to-Test Parity Check

| Violation Example (contract-spec.md) | Corresponding Test |
|-------------------------------------|-------------------|
| `GridSize::new(5.0)` -> OutOfRange | T-ERROR-01 |
| `GridSize::new(150.0)` -> OutOfRange | T-ERROR-02 |
| `GridSize::new(9.999999)` -> OutOfRange | T-ERROR-01 (boundary test) |
| `GridSize::new(-20.0)` -> OutOfRange | T-ERROR-03 |
| `GridSize::new(0.0)` -> OutOfRange | T-ERROR-04 |
| `GridSize::new(NAN)` -> NotFinite | T-ERROR-05 |
| `GridSize::new(INFINITY)` -> NotFinite | T-ERROR-06 |
| `GridSize::new(NEG_INFINITY)` -> NotFinite | T-ERROR-07 |
| `validated_grid_size(5.0)` -> OutOfRange | T-ERROR-01 |
| `validated_grid_size(NAN)` -> NotFinite | T-ERROR-05 |
| JSON `"twenty"` -> InvalidType | T-ERROR-08 |
| JSON `{"value": 20}` -> InvalidType | T-ERROR-08 |
| JSON `5.0` -> OutOfRange | T-ERROR-09 |

---

## Backward Compatibility Verification

| Compatibility Requirement | Test | Status |
|--------------------------|------|--------|
| Existing JSON `grid_size: 20.0` loads | T-HAPPY-08 | Covered |
| New JSON `grid_size: 20.0` format same | T-CONTRACT-04 | Covered |
| Default remains 20.0 | T-HAPPY-04, T-CONTRACT-05 | Covered |
| `snap_value(29.0, true, 20.0) == 20.0` | T-HAPPY-06 | Covered |
| `snap_value(x, true, 0.0)` uses 1.0 | T-EDGE-01 | Covered |

---

## Implementation Order

1. **Phase 1: Create grid module**
   - Create `ui/grid/mod.rs`
   - Implement `GridError` enum
   - Implement `GridSize` newtype with validation
   - Implement `validated_grid_size` function
   - Move `snap_value` and `snap_point` from `interaction.rs`
   - Add tests for new code

2. **Phase 2: Integrate with EditorState**
   - Modify `EditorState.grid_size` type from `OrderedFloat` to `GridSize`
   - Update default implementation
   - Add integration tests

3. **Phase 3: Update call sites**
   - Update `canvas.rs` to use `grid_size.inner()`
   - Update any other files using `grid_size.0`
   - Add re-exports in `interaction.rs` for transition period

4. **Phase 4: Cleanup**
   - Remove dead code from `interaction.rs` (old snap functions)
   - Update `ui/mod.rs` to export grid module
   - Run full test suite
