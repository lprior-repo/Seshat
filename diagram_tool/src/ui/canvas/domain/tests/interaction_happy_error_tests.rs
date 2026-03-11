use crate::ui::canvas::domain::test_utils::interaction_dsl::CanvasTestDsl;
use crate::ui::canvas::domain::{
    CanvasError, CanvasPoint, CanvasVector, InteractionState, RawEvent, SelectionMode,
};

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_raw_event_when_parsed_then_returns_canvas_event() {
    let raw = RawEvent {
        event_type: "mouse_down_target".to_string(),
        x: 10.0,
        y: 20.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };

    let dsl = CanvasTestDsl::new().when_raw_event(raw);
    dsl.then_expect_state(InteractionState::Dragging {
        drag: crate::ui::canvas::domain::DragState {
            start: CanvasPoint::new(10.0, 20.0).unwrap(),
            current: CanvasPoint::new(10.0, 20.0).unwrap(),
            cumulative_offset: CanvasVector::new(0.0, 0.0).unwrap(),
        },
    });
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_idle_state_when_mouse_down_then_transitions_to_selecting() {
    let raw = RawEvent {
        event_type: "mouse_down_background".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };

    CanvasTestDsl::new()
        .given_state(InteractionState::Idle)
        .when_raw_event(raw)
        .then_expect_state(InteractionState::Selecting {
            start: CanvasPoint::new(0.0, 0.0).unwrap(),
            current: CanvasPoint::new(0.0, 0.0).unwrap(),
            mode: SelectionMode::Replace,
        });
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_idle_state_when_mouse_move_then_transitions_to_hovering() {
    let raw = RawEvent {
        event_type: "mouse_move".to_string(),
        x: 5.0,
        y: 5.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };

    CanvasTestDsl::new()
        .given_state(InteractionState::Idle)
        .when_raw_event(raw)
        .then_expect_state(InteractionState::Hovering {
            point: CanvasPoint::new(5.0, 5.0).unwrap(),
        });
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_selecting_state_when_mouse_up_then_transitions_to_idle() {
    let raw = RawEvent {
        event_type: "mouse_up".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };

    CanvasTestDsl::new()
        .given_state(InteractionState::Selecting {
            start: CanvasPoint::new(0.0, 0.0).unwrap(),
            current: CanvasPoint::new(10.0, 10.0).unwrap(),
            mode: SelectionMode::Replace,
        })
        .when_raw_event(raw)
        .then_expect_state(InteractionState::Idle);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_drag_state_when_delta_applied_then_updates_cumulative_offset() {
    let raw = RawEvent {
        event_type: "drag_move".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 5.0,
        dy: 3.0,
        is_additive: false,
    };

    CanvasTestDsl::new()
        .given_state(InteractionState::Dragging {
            drag: crate::ui::canvas::domain::DragState {
                start: CanvasPoint::new(10.0, 10.0).unwrap(),
                current: CanvasPoint::new(10.0, 10.0).unwrap(),
                cumulative_offset: CanvasVector::new(0.0, 0.0).unwrap(),
            },
        })
        .when_raw_event(raw)
        .then_expect_state(InteractionState::Dragging {
            drag: crate::ui::canvas::domain::DragState {
                start: CanvasPoint::new(10.0, 10.0).unwrap(),
                current: CanvasPoint::new(15.0, 13.0).unwrap(),
                cumulative_offset: CanvasVector::new(5.0, 3.0).unwrap(),
            },
        });
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_unknown_raw_event_when_parsed_then_returns_unparseable_error() {
    let raw = RawEvent {
        event_type: "unknown_click".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };

    CanvasTestDsl::new()
        .when_raw_event(raw)
        .then_expect_error(CanvasError::UnparseableEvent);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_idle_state_when_drag_move_event_then_returns_invalid_transition_error() {
    let raw = RawEvent {
        event_type: "drag_move".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 1.0,
        dy: 1.0,
        is_additive: false,
    };

    CanvasTestDsl::new()
        .given_state(InteractionState::Idle)
        .when_raw_event(raw)
        .then_expect_error(CanvasError::InvalidTransition {
            state: "Idle".to_string(),
            event: "DragMove".to_string(),
        });
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_negative_area_when_creating_selection_bounds_then_returns_invalid_bounds_error() {
    let result = crate::ui::canvas::domain::SelectionBounds::new(
        CanvasPoint::new(10.0, 10.0).unwrap(),
        CanvasPoint::new(10.0, 10.0).unwrap(),
    );
    assert_eq!(result.unwrap_err(), CanvasError::InvalidSelectionBounds);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_zero_delta_when_dragging_then_state_unchanged() {
    let raw = RawEvent {
        event_type: "drag_move".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };

    let start_state = InteractionState::Dragging {
        drag: crate::ui::canvas::domain::DragState {
            start: CanvasPoint::new(10.0, 10.0).unwrap(),
            current: CanvasPoint::new(15.0, 15.0).unwrap(),
            cumulative_offset: CanvasVector::new(5.0, 5.0).unwrap(),
        },
    };

    CanvasTestDsl::new()
        .given_state(start_state.clone())
        .when_raw_event(raw)
        .then_expect_state(start_state);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_hovering_state_when_mouse_move_event_then_returns_same_hovering_state() {
    let raw = RawEvent {
        event_type: "mouse_move".to_string(),
        x: 10.0,
        y: 20.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };

    CanvasTestDsl::new()
        .given_state(InteractionState::Hovering {
            point: CanvasPoint::new(0.0, 0.0).unwrap(),
        })
        .when_raw_event(raw)
        .then_expect_state(InteractionState::Hovering {
            point: CanvasPoint::new(10.0, 20.0).unwrap(),
        });
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_dragging_state_when_mouse_down_event_then_returns_invalid_transition_error() {
    let raw = RawEvent {
        event_type: "mouse_down_target".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    };

    CanvasTestDsl::new()
        .given_state(InteractionState::Dragging {
            drag: crate::ui::canvas::domain::DragState {
                start: CanvasPoint::new(0.0, 0.0).unwrap(),
                current: CanvasPoint::new(0.0, 0.0).unwrap(),
                cumulative_offset: CanvasVector::new(0.0, 0.0).unwrap(),
            },
        })
        .when_raw_event(raw)
        .then_expect_error(CanvasError::InvalidTransition {
            state: "Dragging".to_string(),
            event: "MouseDownTarget".to_string(),
        });
}
