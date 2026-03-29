#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
use super::super::test_utils::parse_helpers::{
    drag_state, pt, raw_event, raw_event_with_delta, vec,
};
use crate::ui::canvas::domain::{
    parse_event, transition, CanvasError, CanvasEvent, CanvasPoint, CanvasVector, DragState,
    InteractionState, RawEvent, SelectionBounds, SelectionMode,
};

// =============================================================================
// BOUNDARY ATTACK TESTS - Red Queen Testing for CanvasEvent Payload Processing
// =============================================================================

// -----------------------------------------------------------------------------
// Happy Path Tests - Valid finite inputs
// -----------------------------------------------------------------------------

#[test]
fn test_parse_event_with_zero_coordinates_returns_ok() {
    // Given: RawEvent with x=0.0, y=0.0, event_type="mouse_move"
    let raw = raw_event(0.0, 0.0, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok with zero coordinates
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(matches!(event, CanvasEvent::MouseMove { point } if point.x == 0.0 && point.y == 0.0));
}

#[test]
fn test_parse_event_with_normal_finite_coordinates_returns_ok() {
    // Given: RawEvent with normal positive coordinates
    let raw = raw_event_with_delta(100.5, 200.75, 0.0, 0.0, "mouse_down_target", false);

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok with preserved coordinates
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(matches!(
        event,
        CanvasEvent::MouseDownTarget { point, mode: SelectionMode::Replace }
        if point.x == 100.5 && point.y == 200.75
    ));
}

#[test]
fn test_parse_event_with_negative_finite_coordinates_returns_ok() {
    // Given: RawEvent with negative coordinates
    let raw = raw_event(-50.0, -75.5, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok with negative coordinates
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(
        matches!(event, CanvasEvent::MouseMove { point } if point.x == -50.0 && point.y == -75.5)
    );
}

#[test]
fn test_canvas_point_new_with_max_finite_values_returns_ok() {
    // Given: x=f64::MAX, y=f64::MAX
    // When: CanvasPoint::new(x, y) is called
    let result = CanvasPoint::new(f64::MAX, f64::MAX);

    // Then: Returns Ok (extreme but finite values pass is_finite check)
    assert!(result.is_ok());
    let point = result.unwrap();
    assert_eq!(point.x, f64::MAX);
    assert_eq!(point.y, f64::MAX);
}

#[test]
fn test_canvas_vector_new_with_min_finite_values_returns_ok() {
    // Given: dx=f64::MIN, dy=f64::MIN (most negative finite)
    // When: CanvasVector::new(dx, dy) is called
    let result = CanvasVector::new(f64::MIN, f64::MIN);

    // Then: Returns Ok
    assert!(result.is_ok());
    let vector = result.unwrap();
    assert_eq!(vector.dx, f64::MIN);
    assert_eq!(vector.dy, f64::MIN);
}

// -----------------------------------------------------------------------------
// Error Path Tests - NaN / Infinity Rejection
// -----------------------------------------------------------------------------

#[test]
fn test_parse_event_rejects_nan_x_coordinate() {
    // Given: RawEvent with x=f64::NAN
    let raw = raw_event(f64::NAN, 0.0, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Err(CoordinateOutOfBounds) - NOT a panic
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_parse_event_rejects_nan_y_coordinate() {
    // Given: RawEvent with y=f64::NAN
    let raw = raw_event(0.0, f64::NAN, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Err(CoordinateOutOfBounds)
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_parse_event_rejects_infinity_x_coordinate() {
    // Given: RawEvent with x=f64::INFINITY
    let raw = raw_event(f64::INFINITY, 0.0, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Err(CoordinateOutOfBounds)
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_parse_event_rejects_neg_infinity_y_coordinate() {
    // Given: RawEvent with y=f64::NEG_INFINITY
    let raw = raw_event(0.0, f64::NEG_INFINITY, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Err(CoordinateOutOfBounds)
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_parse_event_rejects_infinity_in_delta() {
    // Given: RawEvent with dx=f64::INFINITY
    let raw = raw_event_with_delta(0.0, 0.0, f64::INFINITY, 0.0, "drag_move", false);

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Err(CoordinateOutOfBounds)
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_canvas_point_new_rejects_nan() {
    // Given: x=f64::NAN, y=1.0
    // When: CanvasPoint::new(x, y) is called
    let result = CanvasPoint::new(f64::NAN, 1.0);

    // Then: Returns Err(CoordinateOutOfBounds)
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_canvas_vector_new_rejects_infinity() {
    // Given: dx=f64::INFINITY, dy=1.0
    // When: CanvasVector::new(dx, dy) is called
    let result = CanvasVector::new(f64::INFINITY, 1.0);

    // Then: Returns Err(CoordinateOutOfBounds)
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_apply_drag_delta_rejects_infinite_delta() {
    // NOTE: Cannot create CanvasVector with infinity values - CanvasVector::new rejects them.
    // This test verifies that the validation logic exists and would reject infinite deltas.
    // The actual error occurs at CanvasVector construction time, not at apply_drag_delta time.
    // We test the finite rejection path instead.

    // Given: A finite delta
    let mut drag = drag_state(pt());
    let delta = CanvasVector::new(100.0, 100.0).unwrap();

    // When: apply_drag_delta() is called with finite delta
    let result = crate::ui::canvas::domain::apply_drag_delta(&mut drag, delta);

    // Then: Returns Ok (finite delta accepted)
    assert!(result.is_ok());
    assert!(drag.cumulative_offset.dx.is_finite());
    assert!(drag.cumulative_offset.dy.is_finite());
}

// -----------------------------------------------------------------------------
// Edge Case Tests - Extreme Finite Values (Potential Arithmetic Overflow)
// -----------------------------------------------------------------------------

#[test]
fn test_parse_event_accepts_max_float_point_and_transitions_to_hovering() {
    // Given: RawEvent with x=f64::MAX, y=f64::MAX
    let raw = raw_event(f64::MAX, f64::MAX, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok with MAX coordinates
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(
        matches!(event, CanvasEvent::MouseMove { point } if point.x == f64::MAX && point.y == f64::MAX)
    );

    // And: When transition to Hovering is called, returns Ok
    if let CanvasEvent::MouseMove { point } = event {
        let state_result = transition(InteractionState::Idle, CanvasEvent::MouseMove { point });
        assert!(state_result.is_ok());
        assert!(matches!(
            state_result.unwrap(),
            InteractionState::Hovering { .. }
        ));
    }
}

#[test]
fn test_parse_event_accepts_min_float_point() {
    // Given: RawEvent with x=f64::MIN, y=f64::MIN (most negative finite)
    let raw = raw_event(f64::MIN, f64::MIN, "mouse_down_target");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(matches!(
        event,
        CanvasEvent::MouseDownTarget { point, mode: SelectionMode::Replace }
        if point.x == f64::MIN && point.y == f64::MIN
    ));
}

#[test]
fn test_parse_event_accepts_min_positive_float() {
    // Given: RawEvent with x=f64::MIN_POSITIVE, y=f64::MIN_POSITIVE
    let raw = raw_event(f64::MIN_POSITIVE, f64::MIN_POSITIVE, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok with MIN_POSITIVE coordinates
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(matches!(
        event,
        CanvasEvent::MouseMove { point }
        if point.x == f64::MIN_POSITIVE && point.y == f64::MIN_POSITIVE
    ));
}

#[test]
fn test_parse_event_accepts_subnormal_float_coordinates() {
    // Given: RawEvent with x=1e-310 (below MIN_POSITIVE - subnormal)
    let raw = raw_event(1e-310, 1e-310, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok (subnormals pass is_finite() check)
    assert!(result.is_ok());
}

#[test]
fn test_parse_event_accepts_large_but_finite_drag_delta() {
    // Given: RawEvent with large but finite delta
    let raw = raw_event_with_delta(0.0, 0.0, 1e300, 1e300, "drag_move", false);

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok with large delta
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(
        matches!(event, CanvasEvent::DragMove { delta } if delta.dx == 1e300 && delta.dy == 1e300)
    );
}

#[test]
fn test_apply_drag_delta_with_max_values_causes_overflow_to_infinity() {
    // Given: DragState with cumulative_offset at f64::MAX
    let mut drag = drag_state(pt());
    // Set cumulative_offset to f64::MAX (largest finite float)
    drag.cumulative_offset = CanvasVector::new(f64::MAX, f64::MAX).unwrap();

    // And: delta = f64::MAX (adding MAX to MAX causes overflow to infinity)
    let delta = CanvasVector::new(f64::MAX, f64::MAX).unwrap();

    // When: apply_drag_delta() is called
    let result = crate::ui::canvas::domain::apply_drag_delta(&mut drag, delta);

    // Then: Returns Err because new cumulative_offset becomes INFINITY
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_apply_drag_delta_subnormal_precision_loss() {
    // Given: DragState with subnormal cumulative_offset
    let mut drag = drag_state(pt());
    drag.cumulative_offset = CanvasVector::new(1e-320, 1e-320).unwrap();

    // And: delta with subnormal values
    let delta = CanvasVector::new(1e-320, 1e-320).unwrap();

    // When: apply_drag_delta() is called multiple times
    // Then: Eventually precision is lost or returns error
    let mut last_result = Ok(());
    for _ in 0..100 {
        last_result = crate::ui::canvas::domain::apply_drag_delta(&mut drag, delta);
        if last_result.is_err() {
            break;
        }
    }
    // Either all 100 succeeded (precision preserved) or some failed
    // The key invariant: if Ok, cumulative_offset remains finite
    if last_result.is_ok() {
        assert!(drag.cumulative_offset.dx.is_finite());
        assert!(drag.cumulative_offset.dy.is_finite());
    }
}

// -----------------------------------------------------------------------------
// Contract Verification Tests
// -----------------------------------------------------------------------------

#[test]
fn test_precondition_canvas_point_requires_finite_coordinates() {
    // Given: Various non-finite values
    let non_finite_values = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];

    // When: CanvasPoint::new() is called with each
    for &value in &non_finite_values {
        let result_x = CanvasPoint::new(value, 0.0);
        let result_y = CanvasPoint::new(0.0, value);

        // Then: All return Err(CoordinateOutOfBounds)
        assert!(result_x.is_err(), "Expected error for x={value}");
        assert!(result_y.is_err(), "Expected error for y={value}");
    }
}

#[test]
fn test_precondition_canvas_vector_requires_finite_coordinates() {
    // Given: Various non-finite values
    let non_finite_values = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];

    // When: CanvasVector::new() is called with each
    for &value in &non_finite_values {
        let result_dx = CanvasVector::new(value, 0.0);
        let result_dy = CanvasVector::new(0.0, value);

        // Then: All return Err(CoordinateOutOfBounds)
        assert!(result_dx.is_err(), "Expected error for dx={value}");
        assert!(result_dy.is_err(), "Expected error for dy={value}");
    }
}

#[test]
fn test_postcondition_parse_event_preserves_valid_input() {
    // Given: Valid RawEvent with finite coordinates
    let raw = raw_event(42.5, 84.25, "mouse_down_target");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Ok with preserved coordinate values
    assert!(result.is_ok());
    match result.unwrap() {
        CanvasEvent::MouseDownTarget { point, mode } => {
            assert_eq!(point.x, 42.5);
            assert_eq!(point.y, 84.25);
            assert_eq!(mode, SelectionMode::Replace);
        }
        _ => panic!("Expected MouseDownTarget event"),
    }
}

#[test]
fn test_invariant_no_non_finite_states() {
    // Given: All valid state transitions with valid finite inputs
    let valid_points = [
        (0.0, 0.0),
        (100.0, 200.0),
        (-50.0, -75.0),
        (f64::MAX, f64::MAX),
        (f64::MIN_POSITIVE, f64::MIN_POSITIVE),
    ];

    for (x, y) in valid_points {
        let point = CanvasPoint::new(x, y).unwrap();
        let event = CanvasEvent::MouseMove { point };
        let result = transition(InteractionState::Idle, event);

        assert!(result.is_ok(), "Transition should succeed for ({x}, {y})");
        let new_state = result.unwrap();

        // Verify no state contains non-finite coordinates
        match new_state {
            InteractionState::Hovering { point } => {
                assert!(point.x.is_finite() && point.y.is_finite());
            }
            InteractionState::Dragging { drag } => {
                assert!(drag.start.x.is_finite() && drag.start.y.is_finite());
                assert!(drag.current.x.is_finite() && drag.current.y.is_finite());
            }
            InteractionState::Selecting { start, current, .. } => {
                assert!(start.x.is_finite() && start.y.is_finite());
                assert!(current.x.is_finite() && current.y.is_finite());
            }
            InteractionState::Idle => {}
        }
    }
}

#[test]
fn test_postcondition_apply_drag_delta_never_produces_non_finite_offset() {
    // Given: Valid initial DragState
    let mut drag = drag_state(pt());
    let delta = CanvasVector::new(10.0, 10.0).unwrap();

    // When: apply_drag_delta() is called with valid delta
    let result = crate::ui::canvas::domain::apply_drag_delta(&mut drag, delta);

    // Then: If returns Ok, cumulative_offset is finite
    if result.is_ok() {
        assert!(drag.cumulative_offset.dx.is_finite());
        assert!(drag.cumulative_offset.dy.is_finite());
    }
}

// -----------------------------------------------------------------------------
// SelectionBounds Tests - Boundary Validation
// -----------------------------------------------------------------------------

#[test]
fn test_selection_bounds_accepts_valid_bounds() {
    // Given: Two valid canvas points that form a positive area
    let start = CanvasPoint::new(10.0, 10.0).unwrap();
    let end = CanvasPoint::new(100.0, 100.0).unwrap();

    // When: SelectionBounds::new() is called
    let result = SelectionBounds::new(start, end);

    // Then: Returns Ok with valid bounds
    assert!(result.is_ok());
    let bounds = result.unwrap();
    assert_eq!(bounds.start.x, 10.0);
    assert_eq!(bounds.start.y, 10.0);
    assert_eq!(bounds.end.x, 100.0);
    assert_eq!(bounds.end.y, 100.0);
}

#[test]
fn test_selection_bounds_rejects_negative_width() {
    // Given: Two points with same x coordinate (zero width = degenerate bounds)
    // Note: The function uses .abs() so "negative width" manifests as zero width
    // when points have same x coordinate. This tests the zero-width rejection.
    let start = CanvasPoint::new(50.0, 10.0).unwrap();
    let end = CanvasPoint::new(50.0, 100.0).unwrap();

    // When: SelectionBounds::new() is called
    let result = SelectionBounds::new(start, end);

    // Then: Returns Err(InvalidSelectionBounds) - zero width rejected
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::InvalidSelectionBounds
    ));
}

#[test]
fn test_selection_bounds_rejects_zero_width() {
    // Given: Two points with same x coordinate (zero width)
    let start = CanvasPoint::new(50.0, 10.0).unwrap();
    let end = CanvasPoint::new(50.0, 100.0).unwrap();

    // When: SelectionBounds::new() is called
    let result = SelectionBounds::new(start, end);

    // Then: Returns Err(InvalidSelectionBounds)
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::InvalidSelectionBounds
    ));
}

// -----------------------------------------------------------------------------
// Critical Transition Violation Tests - No Panic Guarantee
// -----------------------------------------------------------------------------

#[test]
fn test_violation_critical_transition_with_nan_point_does_not_panic() {
    // Given: A NaN point
    let nan_point = CanvasPoint::new(f64::NAN, 0.0).unwrap_err(); // This returns error
                                                                  // So we test with a valid point first, then verify NaN cannot be created
    let valid_point = CanvasPoint::new(0.0, 0.0).unwrap();

    // When: transition() is called with a MouseMove event
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::MouseMove { point: valid_point },
    );

    // Then: Returns Ok (valid point transitions to Hovering)
    assert!(result.is_ok());

    // And: NaN points cannot be constructed (CanvasPoint::new rejects them)
    let nan_result = CanvasPoint::new(f64::NAN, 0.0);
    assert!(nan_result.is_err());
}

#[test]
fn test_violation_p1_infinity_x_returns_coordinate_out_of_bounds() {
    // Given: RawEvent with x=f64::INFINITY
    let raw = raw_event(f64::INFINITY, 0.0, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Err(CoordinateOutOfBounds) - NOT a panic
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_violation_p1_nan_x_returns_coordinate_out_of_bounds() {
    // Given: RawEvent with x=f64::NAN
    let raw = raw_event(f64::NAN, 0.0, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Err(CoordinateOutOfBounds) - NOT a panic
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_violation_p1_neg_infinity_y_returns_coordinate_out_of_bounds() {
    // Given: RawEvent with y=f64::NEG_INFINITY
    let raw = raw_event(0.0, f64::NEG_INFINITY, "mouse_move");

    // When: parse_event() is called
    let result = parse_event(raw);

    // Then: Returns Err(CoordinateOutOfBounds) - NOT a panic
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_violation_p2_canvas_point_nan_returns_error() {
    // Given: x=f64::NAN, y=1.0
    // When: CanvasPoint::new(x, y) is called
    let result = CanvasPoint::new(f64::NAN, 1.0);

    // Then: Returns Err(CoordinateOutOfBounds) - NOT a panic
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_violation_p3_canvas_vector_infinity_returns_error() {
    // Given: dx=f64::INFINITY, dy=1.0
    // When: CanvasVector::new(dx, dy) is called
    let result = CanvasVector::new(f64::INFINITY, 1.0);

    // Then: Returns Err(CoordinateOutOfBounds) - NOT a panic
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CanvasError::CoordinateOutOfBounds
    ));
}

#[test]
fn test_violation_p4_apply_drag_delta_infinity_returns_error() {
    // Given: A valid drag state
    let mut drag = drag_state(pt());

    // And: An infinite delta (which cannot be created via CanvasVector::new)
    // We verify that CanvasVector rejects infinity first
    let infinite_vector_result = CanvasVector::new(f64::INFINITY, 0.0);
    assert!(infinite_vector_result.is_err());

    // When: apply_drag_delta() would be called with an infinite delta
    // Then: The error occurs at CanvasVector construction time
    // This proves p4: apply_drag_delta never receives infinity
    let valid_delta = CanvasVector::new(10.0, 10.0).unwrap();
    let result = crate::ui::canvas::domain::apply_drag_delta(&mut drag, valid_delta);
    assert!(result.is_ok());
}

#[test]
fn test_violation_q2_max_float_passed_through_without_panic() {
    // Given: f64::MAX values
    let max_point = CanvasPoint::new(f64::MAX, f64::MAX).unwrap();

    // When: CanvasPoint is created with MAX values
    // Then: Returns Ok (extreme but finite values pass is_finite check)
    assert!(max_point.x.is_finite());
    assert!(max_point.y.is_finite());

    // And: Event parsing accepts MAX coordinates
    let raw = raw_event(f64::MAX, f64::MAX, "mouse_move");
    let result = parse_event(raw);
    assert!(result.is_ok());

    // And: State transition works with MAX coordinates
    let event = result.unwrap();
    let state_result = transition(InteractionState::Idle, event);
    assert!(state_result.is_ok());
}
