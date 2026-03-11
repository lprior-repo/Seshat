use crate::ui::canvas::domain::{
    apply_drag_delta, transition, CanvasError, CanvasEvent, CanvasPoint, CanvasVector,
    InteractionState, RawEvent, SelectionBounds,
};

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_precondition_parsed_boundaries() {
    let raw = RawEvent {
        event_type: "mouse_down_target".to_string(),
        x: 10.0,
        y: 10.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    let event = crate::ui::canvas::domain::parse_event(raw).unwrap();
    assert!(matches!(event, CanvasEvent::MouseDownTarget { .. }));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_precondition_semantic_coordinates() {
    let pt = CanvasPoint::new(10.0, 20.0).unwrap();
    assert_eq!(pt.x, 10.0);
    assert_eq!(pt.y, 20.0);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p1_violation_returns_unparseable_event() {
    let raw = RawEvent {
        event_type: "unknown_click".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };
    let result = crate::ui::canvas::domain::parse_event(raw);
    assert_eq!(result.unwrap_err(), CanvasError::UnparseableEvent);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p2_violation_returns_coordinate_out_of_bounds() {
    let result = CanvasPoint::new(f64::NAN, f64::INFINITY);
    assert_eq!(result.unwrap_err(), CanvasError::CoordinateOutOfBounds);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_p3_violation_returns_invalid_selection_bounds() {
    let p1 = CanvasPoint::new(10.0, 10.0).unwrap();
    let p2 = CanvasPoint::new(10.0, 10.0).unwrap(); // area is zero
    let result = SelectionBounds::new(p1, p2);
    assert_eq!(result.unwrap_err(), CanvasError::InvalidSelectionBounds);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q2_violation_returns_coordinate_out_of_bounds() {
    let mut drag = crate::ui::canvas::domain::DragState {
        start: CanvasPoint::new(0.0, 0.0).unwrap(),
        current: CanvasPoint::new(0.0, 0.0).unwrap(),
        cumulative_offset: CanvasVector::new(0.0, 0.0).unwrap(),
    };

    // Create vector using unsafe bypass or direct structural initialization if possible
    // Here we can't without unsafe, so we test apply_drag_delta rejecting invalid dx/dy directly.
    let res = apply_drag_delta(
        &mut drag,
        CanvasVector {
            dx: f64::NAN,
            dy: 0.0,
        },
    );
    assert_eq!(res.unwrap_err(), CanvasError::CoordinateOutOfBounds);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_q3_violation_returns_invalid_transition() {
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::DragMove {
            delta: CanvasVector::new(1.0, 1.0).unwrap(),
        },
    );
    assert_eq!(
        result.unwrap_err(),
        CanvasError::InvalidTransition {
            state: "Idle".to_string(),
            event: "DragMove".to_string(),
        }
    );
}
