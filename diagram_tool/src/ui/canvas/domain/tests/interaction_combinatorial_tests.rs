use crate::ui::canvas::domain::test_utils::interaction_dsl::CanvasTestDsl;
use crate::ui::canvas::domain::{
    CanvasEvent, CanvasPoint, CanvasVector, InteractionState, SelectionMode,
};

fn pt() -> CanvasPoint {
    CanvasPoint::new(0.0, 0.0).unwrap()
}
fn vec() -> CanvasVector {
    CanvasVector::new(0.0, 0.0).unwrap()
}

fn all_events() -> Vec<CanvasEvent> {
    vec![
        CanvasEvent::MouseDownTarget {
            point: pt(),
            mode: SelectionMode::Replace,
        },
        CanvasEvent::MouseDownBackground {
            point: pt(),
            mode: SelectionMode::Replace,
        },
        CanvasEvent::MouseMove { point: pt() },
        CanvasEvent::DragMove { delta: vec() },
        CanvasEvent::MouseUp,
        CanvasEvent::TouchDownTarget {
            point: pt(),
            mode: SelectionMode::Replace,
        },
        CanvasEvent::TouchDownBackground {
            point: pt(),
            mode: SelectionMode::Replace,
        },
        CanvasEvent::TouchMove {
            point: pt(),
            delta: vec(),
        },
        CanvasEvent::TouchUp,
    ]
}

#[test]
fn test_exhaustive_idle_transitions() {
    let state = InteractionState::Idle;
    for event in all_events() {
        let dsl = CanvasTestDsl::new()
            .given_state(state.clone())
            .when_parsed_event(event.clone());

        match event {
            CanvasEvent::MouseDownTarget { .. } | CanvasEvent::TouchDownTarget { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Dragging { .. }
                ));
            }
            CanvasEvent::MouseDownBackground { .. } | CanvasEvent::TouchDownBackground { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Selecting { .. }
                ));
            }
            CanvasEvent::MouseMove { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Hovering { .. }
                ));
            }
            CanvasEvent::MouseUp | CanvasEvent::TouchUp | CanvasEvent::TouchMove { .. } => {
                assert!(matches!(dsl.state.unwrap(), InteractionState::Idle));
            }
            _ => {
                assert!(dsl.last_result.unwrap().is_err());
            }
        }
    }
}

#[test]
fn test_exhaustive_hovering_transitions() {
    let state = InteractionState::Hovering { point: pt() };
    for event in all_events() {
        let dsl = CanvasTestDsl::new()
            .given_state(state.clone())
            .when_parsed_event(event.clone());

        match event {
            CanvasEvent::MouseDownTarget { .. } | CanvasEvent::TouchDownTarget { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Dragging { .. }
                ));
            }
            CanvasEvent::MouseDownBackground { .. } | CanvasEvent::TouchDownBackground { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Selecting { .. }
                ));
            }
            CanvasEvent::MouseMove { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Hovering { .. }
                ));
            }
            CanvasEvent::MouseUp | CanvasEvent::TouchUp => {
                assert!(matches!(dsl.state.unwrap(), InteractionState::Idle));
            }
            _ => {
                assert!(dsl.last_result.unwrap().is_err());
            }
        }
    }
}

#[test]
fn test_exhaustive_dragging_transitions() {
    let state = InteractionState::Dragging {
        drag: crate::ui::canvas::domain::DragState {
            start: pt(),
            current: pt(),
            cumulative_offset: vec(),
        },
    };
    for event in all_events() {
        let dsl = CanvasTestDsl::new()
            .given_state(state.clone())
            .when_parsed_event(event.clone());

        match event {
            CanvasEvent::DragMove { .. } | CanvasEvent::TouchMove { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Dragging { .. }
                ));
            }
            CanvasEvent::MouseUp | CanvasEvent::TouchUp => {
                assert!(matches!(dsl.state.unwrap(), InteractionState::Idle));
            }
            CanvasEvent::MouseMove { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Dragging { .. }
                ));
            }
            _ => {
                assert!(dsl.last_result.unwrap().is_err());
            }
        }
    }
}

#[test]
fn test_exhaustive_selecting_transitions() {
    let state = InteractionState::Selecting {
        start: pt(),
        current: pt(),
        mode: SelectionMode::Replace,
    };
    for event in all_events() {
        let dsl = CanvasTestDsl::new()
            .given_state(state.clone())
            .when_parsed_event(event.clone());

        match event {
            CanvasEvent::MouseMove { .. } | CanvasEvent::TouchMove { .. } => {
                assert!(matches!(
                    dsl.state.unwrap(),
                    InteractionState::Selecting { .. }
                ));
            }
            CanvasEvent::MouseUp | CanvasEvent::TouchUp => {
                assert!(matches!(dsl.state.unwrap(), InteractionState::Idle));
            }
            _ => {
                assert!(dsl.last_result.unwrap().is_err());
            }
        }
    }
}
