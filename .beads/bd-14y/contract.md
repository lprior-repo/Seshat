bead_id: bd-14y
bead_title: edge-case-bdd-tests-numeric-boundaries
phase: p0
updated_at: 2026-03-02T05:25:00Z

# Contract: BDD Tests for Numeric Boundaries Edge Cases

## Scope

Add comprehensive BDD-style tests for numeric boundary handling across the diagram_tool codebase. Tests must cover edge cases for revision numbers, timestamps, and floating-point special values (infinity, negative infinity, NaN).

## Required Test Cases

### 1. Max Revision Numbers

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_revision_near_u64_max_when_appending_event_then_handles_gracefully` | Store at revision near `u64::MAX` | Append event attempted | Returns appropriate error or handles correctly without overflow |
| `given_revision_at_u64_max_when_incrementing_then_wraps_or_errors` | Projection at `u64::MAX` | Replay attempts increment | No silent overflow, either wraps or returns error |
| `given_large_revision_number_when_serializing_then_preserves_value` | Projection with revision `1_000_000_000` | Serialize and deserialize | Revision preserved exactly |

### 2. Timestamp Boundaries (Min/Max/Negative)

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_timestamp_at_i64_max_when_creating_envelope_then_preserves_value` | Envelope with `i64::MAX` timestamp | Serialize and deserialize | Timestamp preserved exactly |
| `given_timestamp_at_i64_min_when_creating_envelope_then_preserves_value` | Envelope with `i64::MIN` timestamp | Serialize and deserialize | Timestamp preserved exactly |
| `given_negative_timestamp_when_replaying_then_handles_gracefully` | Event with timestamp `-1` | Replay is attempted | No panic, handles gracefully (may be valid for pre-epoch) |
| `given_zero_timestamp_when_creating_envelope_then_succeeds` | Envelope with timestamp `0` | Create and serialize | Succeeds without error |

### 3. Infinity Values in Coordinates

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_node_with_positive_infinity_x_when_applying_operation_then_handles_gracefully` | NodeAdd with `x: f64::INFINITY` | Operation is applied | No panic, either succeeds or returns validation error |
| `given_node_with_negative_infinity_y_when_applying_operation_then_handles_gracefully` | NodeAdd with `y: f64::NEG_INFINITY` | Operation is applied | No panic, either succeeds or returns validation error |
| `given_edge_with_infinity_coordinates_when_exporting_then_serializes_correctly` | Edge geometry with infinity values | Export to JSON | Serializes without error (JSON representation) |
| `given_geometry_calculation_with_infinity_input_then_no_panic` | Bounding box with infinity | Geometry methods called | No panic, returns sensible result or infinity |

### 4. NaN Values

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_node_with_nan_coordinates_when_applying_operation_then_handles_gracefully` | NodeAdd with `x: f64::NAN` | Operation is applied | No panic, either succeeds with NaN or returns error |
| `given_nan_in_dimension_when_validating_then_fails_or_propagates` | Node with `width: f64::NAN` | Validation runs | Either fails validation or NaN propagates without panic |
| `given_geometry_with_nan_when_calculating_bounds_then_no_panic` | Bounding box calculation with NaN | Bounds computed | No panic, returns NaN or empty bounds |

### 5. Extreme Floating-Point Values

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_very_large_coordinate_when_exporting_then_no_overflow` | Node at `x: 1e308` | Export to JSON | No overflow, serializes correctly |
| `given_very_small_positive_coordinate_when_exporting_then_no_underflow` | Node at `x: 1e-308` | Export to JSON | No underflow, serializes correctly |
| `given_subnormal_float_when_roundtripping_then_preserves_value` | Node with subnormal float | Export then import | Value preserved or gracefully handled |

### 6. Revision Overflow in Snapshot/Replay

| Test Case | Given | When | Then |
|-----------|-------|------|------|
| `given_events_with_sequential_revisions_near_max_when_replaying_then_no_overflow` | Events with revisions near `u64::MAX - 10` | Replay attempted | No silent overflow during replay |
| `given_snapshot_with_large_revision_when_loading_then_preserves_value` | Snapshot at revision `u64::MAX / 2` | Load projection | Revision preserved correctly |

## Implementation Requirements

1. **Location**: Tests should be added to the relevant modules:
   - Revision tests: `diagram_tool/src/store.rs` or `diagram_tool/src/models/projection.rs`
   - Timestamp tests: `diagram_tool/src/models/envelope.rs` or `diagram_tool/src/models/snapshot.rs`
   - Geometry/float tests: `diagram_tool/src/geometry/mod.rs` or `diagram_tool/src/models/projection.rs`

2. **Naming Convention**: All tests must follow `given_X_when_Y_then_Z` BDD naming pattern.

3. **Assertions**: Each test must have clear assertions that verify:
   - No panic occurs (use `std::panic::catch_unwind` if needed)
   - Correct error type is returned for error cases
   - Values are preserved exactly for roundtrip tests

4. **No Unwrap/Expect**: Tests must not use `.unwrap()` or `.expect()` - use `assert!` on Result::is_ok/is_err or pattern match.

5. **Edge Case Priority**: Prioritize tests that exercise:
   - Integer overflow/underflow boundaries
   - Floating-point special values (NaN, Infinity)
   - Serialization edge cases with extreme values

## Acceptance Criteria

- [ ] All 20+ test cases implemented
- [ ] All tests pass with `cargo test --package diagram_tool`
- [ ] No new clippy warnings introduced
- [ ] Test coverage increases for numeric handling paths
- [ ] Moon validation passes (`moon run :test`)

## Out of Scope

- Fuzz testing (covered by proptests in other modules)
- Performance testing with extreme values
- Hardware-specific floating-point behavior
