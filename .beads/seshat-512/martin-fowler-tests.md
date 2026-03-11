# Martin Fowler Test Plan

## Happy Path Tests
- `test_returns_pan_action_for_two_finger_movement_inp_004`
- `test_returns_high_precision_hit_test_for_stylus_pen_inp_005`
- `test_returns_double_tap_action_when_tapped_twice_rapidly_inp_006`
- `test_returns_hit_success_for_touch_within_expanded_radius_inp_007`

## Error Path Tests
- `test_returns_error_when_pointer_move_received_for_untracked_id`
- `test_returns_error_when_too_many_simultaneous_pointers_active`

## Edge Case Tests
- `test_handles_pointer_up_without_prior_down_gracefully`
- `test_handles_rapid_alternating_pen_and_touch_events_correctly`
- `test_handles_two_taps_just_outside_double_tap_timing_threshold`

## Contract Verification Tests
- `test_precondition_two_finger_gesture_requires_distinct_ids`
- `test_postcondition_two_finger_pan_does_not_move_shapes`
- `test_postcondition_stylus_ignores_touch_padding`
- `test_invariant_touch_hit_radius_always_ge_mouse_radius`
- `test_invariant_max_active_pointers_enforced`

## Contract Violation Tests
- `test_p1_violation_returns_compile_error_or_invalid_timing_threshold`
  Given: `InputConfig` with `0` for `double_tap_timeout_ms`
  When: Configuration is instantiated
  Then: Returns `Err(Error::InvalidTimingThreshold)`
  
- `test_p2_violation_returns_negative_hit_padding_error`
  Given: `InputConfig` with `-5.0` for `touch_padding`
  When: Configuration is instantiated
  Then: Returns `Err(Error::NegativeHitPadding)`
  
- `test_p3_violation_returns_duplicate_pointer_id_error`
  Given: Attempt to create `TwoFingerGesture`
  When: Both provided pointer IDs are identical
  Then: Returns `Err(Error::DuplicatePointerId)`
  
- `test_p4_violation_returns_untracked_pointer_error`
  Given: An `InputState` with no active pointers
  When: `process_pointer_event` is called with `PointerMove(id: 99)`
  Then: Returns `Err(Error::UntrackedPointer)`

- `test_q1_violation_pan_moves_shape`
  Given: A mock state returning `MoveShape` for a two-finger interaction
  When: Validating the output actions
  Then: The test infrastructure should flag this as a contract violation (in this plan, we verify the implementation does NOT do this).

- `test_q2_violation_stylus_uses_touch_padding`
  Given: A hit test with `PointerType::Pen` slightly outside the base radius but inside the touch padded radius
  When: `hit_test_handle` is called
  Then: It MUST return `Ok(false)`, enforcing that the violation doesn't occur.

- `test_q3_violation_returns_single_taps_instead_of_double_tap`
  Given: Two sequential taps within 100ms
  When: `process_pointer_event` is called
  Then: The output actions MUST contain `DoubleTap` and NOT two `SingleTap` actions.

- `test_q4_violation_touch_fails_within_expanded_radius`
  Given: A `PointerType::Touch` hit test at `base_radius + (touch_padding / 2)`
  When: `hit_test_handle` is called
  Then: It MUST return `Ok(true)`, enforcing the touch padding is applied.

## Given-When-Then Scenarios

### Scenario 1: Two-Finger Pan Prevents Shape Movement (INP-004)
Given: A selected shape on the canvas and `InputState` tracking one active touch on the shape.
When: A second touch event `PointerDown` occurs, followed by `PointerMove` for both fingers.
Then: 
- The second `PointerDown` transitions the state to a `PanGesture` mode.
- The `PointerMove` events emit a `PanCamera` action.
- The `PointerMove` events DO NOT emit a `MoveShape` action.

### Scenario 2: Double-Tap Timing Consistency (INP-006)
Given: An empty `InputState` and a `double_tap_timeout_ms` of 300ms.
When: A `PointerDown` and `PointerUp` occur, followed by a second `PointerDown` on the same location 250ms later.
Then:
- The first tap may emit a `SingleTap` or queue it (depending on UI design).
- The second tap emits a `DoubleTap` action.
- The `InputState` tap history is cleared or reset to prevent triple-tap being read as two double-taps.

### Scenario 3: Stylus vs Touch Hit Testing (INP-005, INP-007)
Given: A shape resize handle with `base_radius = 5.0` and `touch_padding = 10.0`.
When: A `PointerDown` with `PointerType::Pen` occurs at distance `8.0` from the handle center.
Then:
- `hit_test_handle` returns `false` (stylus requires precision).
When: A `PointerDown` with `PointerType::Touch` occurs at distance `12.0` from the handle center.
Then:
- `hit_test_handle` returns `true` (touch uses expanded radius of 15.0).
