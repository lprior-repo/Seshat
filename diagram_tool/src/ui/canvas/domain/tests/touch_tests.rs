#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
// Happy Path Tests
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_parses_touch_down_target_successfully() {
    let raw = RawEvent {
        event_type: "touch_down_target".to_string(),
        x: 10.0,
        y: 20.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    let event = parse_event(raw).unwrap();
    assert_eq!(
        event,
        CanvasEvent::TouchDownTarget {
            point: CanvasPoint::new(10.0, 20.0).unwrap(),
            mode: SelectionMode::Replace,
        }
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_parses_touch_down_background_successfully() {
    let raw = RawEvent {
        event_type: "touch_down_background".to_string(),
        x: 30.0,
        y: 40.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: true,
    };
    let event = parse_event(raw).unwrap();
    assert_eq!(
        event,
        CanvasEvent::TouchDownBackground {
            point: CanvasPoint::new(30.0, 40.0).unwrap(),
            mode: SelectionMode::Additive,
        }
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_parses_touch_move_successfully() {
    let raw = RawEvent {
        event_type: "touch_move".to_string(),
        x: 50.0,
        y: 60.0,
        dx: 5.0,
        dy: -5.0,
        is_additive: false,
    };
    let event = parse_event(raw).unwrap();
    assert_eq!(
        event,
        CanvasEvent::TouchMove {
            point: CanvasPoint::new(50.0, 60.0).unwrap(),
            delta: CanvasVector::new(5.0, -5.0).unwrap(),
        }
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_parses_touch_up_successfully() {
    let raw = RawEvent {
        event_type: "touch_up".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    let event = parse_event(raw).unwrap();
    assert_eq!(event, CanvasEvent::TouchUp);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_reduces_touch_down_target_from_idle_to_dragging() {
    let state = InteractionState::Idle;
    let event = CanvasEvent::TouchDownTarget {
        point: CanvasPoint::new(10.0, 20.0).unwrap(),
        mode: SelectionMode::Replace,
    };
    let new_state = reduce(state, event).unwrap();
    assert_eq!(
        new_state,
        InteractionState::Dragging {
            drag: DragState {
                start: CanvasPoint::new(10.0, 20.0).unwrap(),
                current: CanvasPoint::new(10.0, 20.0).unwrap(),
                cumulative_offset: CanvasVector::new(0.0, 0.0).unwrap(),
            }
        }
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_reduces_touch_down_background_from_idle_to_selecting() {
    let state = InteractionState::Idle;
    let event = CanvasEvent::TouchDownBackground {
        point: CanvasPoint::new(30.0, 40.0).unwrap(),
        mode: SelectionMode::Additive,
    };
    let new_state = reduce(state, event).unwrap();
    assert_eq!(
        new_state,
        InteractionState::Selecting {
            start: CanvasPoint::new(30.0, 40.0).unwrap(),
            current: CanvasPoint::new(30.0, 40.0).unwrap(),
            mode: SelectionMode::Additive,
        }
    );
}

// Error Path Tests
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_touch_coordinates_are_nan() {
    let raw = RawEvent {
        event_type: "touch_down_target".to_string(),
        x: f64::NAN,
        y: 20.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    let result = parse_event(raw);
    assert_eq!(result, Err(CanvasError::CoordinateOutOfBounds));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_when_touch_deltas_are_infinity() {
    let raw = RawEvent {
        event_type: "touch_move".to_string(),
        x: 50.0,
        y: 60.0,
        dx: f64::INFINITY,
        dy: 0.0,
        is_additive: false,
    };
    let result = parse_event(raw);
    assert_eq!(result, Err(CanvasError::CoordinateOutOfBounds));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_returns_error_for_unknown_touch_event_type() {
    let raw = RawEvent {
        event_type: "touch_hover".to_string(),
        x: 10.0,
        y: 10.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    let result = parse_event(raw);
    assert_eq!(result, Err(CanvasError::UnparseableEvent));
}

// Edge Case Tests
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_handles_zero_delta_touch_move_gracefully() {
    let state = InteractionState::Dragging {
        drag: DragState {
            start: CanvasPoint::new(10.0, 20.0).unwrap(),
            current: CanvasPoint::new(15.0, 25.0).unwrap(),
            cumulative_offset: CanvasVector::new(5.0, 5.0).unwrap(),
        },
    };
    let event = CanvasEvent::TouchMove {
        point: CanvasPoint::new(15.0, 25.0).unwrap(),
        delta: CanvasVector::new(0.0, 0.0).unwrap(),
    };
    let new_state = reduce(state.clone(), event).unwrap();
    assert_eq!(new_state, state);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_ignores_touch_move_when_idle() {
    let state = InteractionState::Idle;
    let event = CanvasEvent::TouchMove {
        point: CanvasPoint::new(10.0, 20.0).unwrap(),
        delta: CanvasVector::new(2.0, 3.0).unwrap(),
    };
    let new_state = reduce(state, event).unwrap();
    assert_eq!(new_state, InteractionState::Idle);
}

// Contract Verification Tests
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_precondition_finite_coordinates_for_touch() {
    let raw = RawEvent {
        event_type: "touch_down_background".to_string(),
        x: f64::INFINITY,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    assert_eq!(parse_event(raw), Err(CanvasError::CoordinateOutOfBounds));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_precondition_finite_deltas_for_touch() {
    let raw = RawEvent {
        event_type: "touch_move".to_string(),
        x: 0.0,
        y: 0.0,
        dx: f64::NAN,
        dy: 0.0,
        is_additive: false,
    };
    assert_eq!(parse_event(raw), Err(CanvasError::CoordinateOutOfBounds));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_postcondition_touch_move_ignored_when_idle() {
    let state = InteractionState::Idle;
    let event = CanvasEvent::TouchMove {
        point: CanvasPoint::new(10.0, 10.0).unwrap(),
        delta: CanvasVector::new(5.0, 5.0).unwrap(),
    };
    assert_eq!(reduce(state, event), Ok(InteractionState::Idle));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_invariant_touch_never_produces_nan_points() {
    let raw1 = RawEvent {
        event_type: "touch_down_target".to_string(),
        x: f64::NAN,
        y: 10.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    assert_eq!(parse_event(raw1), Err(CanvasError::CoordinateOutOfBounds));

    let raw2 = RawEvent {
        event_type: "touch_move".to_string(),
        x: 10.0,
        y: 10.0,
        dx: 5.0,
        dy: f64::NAN,
        is_additive: false,
    };
    assert_eq!(parse_event(raw2), Err(CanvasError::CoordinateOutOfBounds));
}

// Contract Violation Tests
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p1_violation_returns_coordinate_out_of_bounds() {
    let raw = RawEvent {
        event_type: "touch_down_target".to_string(),
        x: f64::NAN,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    assert_eq!(parse_event(raw), Err(CanvasError::CoordinateOutOfBounds));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p2_violation_returns_coordinate_out_of_bounds() {
    let raw = RawEvent {
        event_type: "touch_move".to_string(),
        x: 0.0,
        y: 0.0,
        dx: f64::INFINITY,
        dy: 0.0,
        is_additive: false,
    };
    assert_eq!(parse_event(raw), Err(CanvasError::CoordinateOutOfBounds));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p3_violation_returns_unparseable_event() {
    let raw = RawEvent {
        event_type: "touch_hover".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    assert_eq!(parse_event(raw), Err(CanvasError::UnparseableEvent));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q5_violation_prevents_hover_state() {
    let state = InteractionState::Idle;
    let event = CanvasEvent::TouchMove {
        point: CanvasPoint::new(0.0, 0.0).unwrap(),
        delta: CanvasVector::new(0.0, 0.0).unwrap(),
    };
    assert_eq!(reduce(state, event), Ok(InteractionState::Idle));
}
