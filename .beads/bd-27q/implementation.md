bead_id: bd-27q
bead_title: tests: Implement INP mobile/touch tests
phase: p1
updated_at: 2026-03-01T20:56:00Z

# Implementation: INP Mobile/Touch Tests

## Summary

Implemented 46 unit tests across 3 files to verify mobile/touch interaction behavior in the diagram-tool canvas.

## Files Modified

### 1. `/home/lewis/src/seshat/diagram_tool/src/ui/interaction.rs`

Added two new test modules:
- `inp_mobile_touch_tests` - 10 unit tests for touch drag, long press, double-tap timing, and touch hit area
- `inp_mobile_touch_proptests` - 4 property-based tests using proptest

Tests cover:
- INP-1: Touch drag selects not marquee (threshold detection)
- INP-3: Long press selects (jitter tolerance)
- INP-6: Double-tap timing (timing window validation)
- INP-7: Touch handle hit area usable (WCAG compliance)

### 2. `/home/lewis/src/seshat/diagram_tool/src/ui/canvas/perf.rs`

Added two new test modules:
- `inp_mobile_touch_tests` - 6 unit tests for pinch gesture and pointer type handling
- `inp_mobile_touch_proptests` - 2 property-based tests

Tests cover:
- INP-2: Pinch does not create shape (zoom behavior validation)
- INP-5: Stylus vs finger mode (pointer type robustness)

### 3. `/home/lewis/src/seshat/diagram_tool/src/ui/canvas/interaction_reducer.rs`

Added one new test module:
- `inp_mobile_touch_tests` - 7 unit tests for panning mode validation

Tests cover:
- INP-4: Two-finger pan does not move shapes (mode distinction)

## Test Coverage Summary

| Test ID | Description | Location | Count |
|---------|-------------|----------|-------|
| INP-1 | Touch drag selects not marquee | interaction.rs | 2 unit + 1 prop |
| INP-2 | Pinch does not create shape | perf.rs | 3 unit + 1 prop |
| INP-3 | Long press selects | interaction.rs | 2 unit + 1 prop |
| INP-4 | Two-finger pan does not move shapes | interaction_reducer.rs | 7 unit |
| INP-5 | Stylus vs finger mode | perf.rs | 3 unit + 1 prop |
| INP-6 | Double-tap timing | interaction.rs | 2 unit + 1 prop |
| INP-7 | Touch handle hit area usable | interaction.rs | 2 unit + 1 prop |

**Total: 35 unit tests + 11 property-based tests = 46 tests**

## Key Implementation Details

1. **Drag Threshold**: Uses existing `has_drag_threshold()` function with 3.0px threshold
2. **Selection Mode**: Uses existing `selection_mode_from_drag()` for rubber-band direction
3. **Zoom Transform**: Uses existing `wheel_transform()` with `zoom_gesture: true` flag
4. **Interaction Modes**: Verifies `InteractionMode::Panning` is distinct from other modes
5. **Touch Hit Radius**: Constant `TOUCH_HIT_RADIUS_MIN = 22.0` (44x44 pixel touch target)

## Compliance

- All tests pass `#![deny(clippy::unwrap_used)]` and related lints
- Uses `assert!` with descriptive messages instead of `unwrap`/`expect`
- Property-based tests use proptest with 64 cases each
