# Martin Fowler Test Plan: Touch Input Basics (INP-001 to INP-003)

## Happy Path Tests
- `test_parses_touch_down_target_successfully`
- `test_parses_touch_down_background_successfully`
- `test_parses_touch_move_successfully`
- `test_parses_touch_up_successfully`
- `test_reduces_touch_down_target_from_idle_to_dragging`
- `test_reduces_touch_down_background_from_idle_to_selecting`

## Error Path Tests
- `test_returns_error_when_touch_coordinates_are_nan`
- `test_returns_error_when_touch_deltas_are_infinity`
- `test_returns_error_for_unknown_touch_event_type`

## Edge Case Tests
- `test_handles_zero_delta_touch_move_gracefully`
- `test_ignores_touch_move_when_idle`

## Contract Verification Tests
- `test_precondition_finite_coordinates_for_touch`
- `test_precondition_finite_deltas_for_touch`
- `test_postcondition_touch_move_ignored_when_idle`
- `test_invariant_touch_never_produces_nan_points`

## Contract Violation Tests
- `test_p1_violation_returns_coordinate_out_of_bounds`
  Given: `RawEvent` with `event_type: "touch_down_target"` and `x: f64::NAN`
  When: `parse_event` is called
  Then: returns `Err(CanvasError::CoordinateOutOfBounds)`

- `test_p2_violation_returns_coordinate_out_of_bounds`
  Given: `RawEvent` with `event_type: "touch_move"` and `dx: f64::INFINITY`
  When: `parse_event` is called
  Then: returns `Err(CanvasError::CoordinateOutOfBounds)`

- `test_p3_violation_returns_unparseable_event`
  Given: `RawEvent` with `event_type: "touch_hover"`
  When: `parse_event` is called
  Then: returns `Err(CanvasError::UnparseableEvent)`

- `test_q5_violation_prevents_hover_state`
  Given: `InteractionState::Idle`
  When: `reduce` is called with `CanvasEvent::TouchMove`
  Then: returns `Ok(InteractionState::Idle)`, preventing invalid hover state entry

## Given-When-Then Scenarios
### Scenario 1: Basic Touch Tap on Background (INP-001)
Given: InteractionState is Idle
When: A TouchDownBackground event occurs
Then:
- State transitions to Selecting
- The selection start point matches the touch coordinates

### Scenario 2: Stray Touch Move Without Touch Down (INP-002 Edge Case)
Given: InteractionState is Idle
When: A TouchMove event occurs
Then:
- State remains Idle
- The event is effectively ignored because touch screens do not have hover cursors

### Scenario 3: Touch Drag on Target (INP-002)
Given: InteractionState is Idle
When: A TouchDownTarget event occurs, followed by TouchMove
Then:
- State transitions from Idle to Dragging
- The DragState updates its current point and cumulative offset based on TouchMove deltas

### Scenario 4: Touch Release (INP-003)
Given: InteractionState is Dragging
When: A TouchUp event occurs
Then:
- State transitions to Idle
- The canvas interaction completes successfully