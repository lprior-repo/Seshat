bead_id: bd-27q
bead_title: tests: Implement INP mobile/touch tests
phase: p0
updated_at: 2026-03-01T12:00:00Z

# Contract: INP Mobile/Touch Tests

## Scope

Implement 7 mobile/touch interaction tests for the INP (Input) system in the diagram-tool canvas.

## Test Requirements

### 1. Touch Drag Selects Not Marquee
- **Behavior**: A single-finger touch drag on canvas background should select nodes via rubber-band selection (not trigger marquee zoom).
- **Test Location**: `diagram_tool/src/ui/canvas/interaction.rs` (test module)
- **Acceptance**: Verify that `InteractionMode::RubberBand` is initiated on touch drag without modifier keys.

### 2. Pinch Does Not Create Shape
- **Behavior**: A two-finger pinch gesture should zoom the canvas, not create a new shape or subgraph.
- **Test Location**: `diagram_tool/src/ui/canvas/perf.rs` (test module)
- **Acceptance**: Verify that `wheel_transform` with `zoom_gesture: true` produces valid zoom values and does not trigger selection or drawing modes.

### 3. Long Press Selects
- **Behavior**: A long press (touch hold without movement) on a node should select it.
- **Test Location**: `diagram_tool/src/ui/interaction.rs` (test module)
- **Acceptance**: Verify that touch events held beyond the drag threshold timing do not trigger drag, but select the target node.

### 4. Two-Finger Pan Does Not Move Shapes
- **Behavior**: A two-finger pan gesture should pan the canvas, not move selected shapes.
- **Test Location**: `diagram_tool/src/ui/canvas/interaction_reducer.rs` (test module)
- **Acceptance**: Verify that `InteractionMode::Panning` takes precedence over `DraggingSelection` when two touch points are active.

### 5. Stylus vs Finger Mode
- **Behavior**: The system should distinguish between stylus and finger touch input modes if the platform provides this information.
- **Test Location**: `diagram_tool/src/ui/canvas/perf.rs` (test module)
- **Acceptance**: Verify that `WheelInput` and interaction handlers can process different pointer types without panicking.

### 6. Double-Tap Timing
- **Behavior**: Double-tap should be detected within the appropriate timing window (typically 300-500ms).
- **Test Location**: `diagram_tool/src/ui/interaction.rs` (test module)
- **Acceptance**: Verify that double-tap detection timing thresholds are consistent and finite.

### 7. Touch Handle Hit Area Usable
- **Behavior**: Selection handles should have touch-friendly hit areas (at least 44x44 points as per accessibility guidelines).
- **Test Location**: `diagram_tool/src/ui/canvas/canvas_view.rs` (test module)
- **Acceptance**: Verify that `find_edge_at` and selection handle hit testing use appropriate hit radii for touch.

## Implementation Constraints

1. **No UI Framework Changes**: Tests must be unit tests in Rust, not Playwright/E2E tests.
2. **Follow Existing Patterns**: Use the existing test patterns in `interaction.rs`, `perf.rs`, and `canvas_view.rs`.
3. **Property-Based Testing**: Where appropriate, use `proptest` for coverage of edge cases.
4. **Lint Compliance**: All tests must pass `#![deny(clippy::unwrap_used)]` and related lints.

## Acceptance Criteria

- [ ] 7 new test functions implemented across appropriate modules
- [ ] All tests pass: `moon run :test-rust`
- [ ] No clippy warnings: `moon run :clippy`
- [ ] Code formatted: `cargo fmt --check`

## Dependencies

- `diagram_tool/src/ui/interaction.rs` - Selection and drag logic
- `diagram_tool/src/ui/canvas/perf.rs` - Wheel/zoom transform logic
- `diagram_tool/src/ui/canvas/canvas_view.rs` - Hit testing
- `diagram_tool/src/ui/canvas/interaction_reducer.rs` - Interaction mode state machine

## Out of Scope

- E2E/Playwright touch tests
- Actual touch event integration (browser-level)
- Mobile viewport/responsive layout tests (covered in `mobile.rs`)
