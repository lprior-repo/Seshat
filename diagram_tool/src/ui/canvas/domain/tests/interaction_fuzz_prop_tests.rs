use super::super::test_utils::interaction_dsl::CanvasTestDsl;
use super::super::test_utils::parse_helpers::{all_events, pt, vec};
use crate::ui::canvas::domain::{
    CanvasEvent, CanvasPoint, CanvasVector, InteractionState, SelectionMode,
};
use proptest::prelude::*;

proptest! {
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn fuzz_parse_event_never_panics(
        event_type in ".*",
        x in any::<f64>(),
        y in any::<f64>(),
        dx in any::<f64>(),
        dy in any::<f64>(),
        is_additive in any::<bool>(),
    ) {
        let raw = RawEvent { event_type, x, y, dx, dy, is_additive };
        let _ = parse_event(raw); // Should return Ok or Err, but never panic.
    }
}

fn arb_point() -> impl Strategy<Value = CanvasPoint> {
    (any::<f64>(), any::<f64>()).prop_filter_map("finite", |(x, y)| CanvasPoint::new(x, y).ok())
}

fn arb_vector() -> impl Strategy<Value = CanvasVector> {
    (any::<f64>(), any::<f64>())
        .prop_filter_map("finite", |(dx, dy)| CanvasVector::new(dx, dy).ok())
}

fn arb_event() -> impl Strategy<Value = CanvasEvent> {
    prop_oneof![
        arb_point().prop_map(|point| CanvasEvent::MouseDownTarget {
            point,
            mode: SelectionMode::Replace
        }),
        arb_point().prop_map(|point| CanvasEvent::MouseDownBackground {
            point,
            mode: SelectionMode::Replace
        }),
        arb_point().prop_map(|point| CanvasEvent::MouseMove { point }),
        arb_vector().prop_map(|delta| CanvasEvent::DragMove { delta }),
        Just(CanvasEvent::MouseUp),
    ]
}

proptest! {
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_valid_event_sequence_maintains_invariants(events in prop::collection::vec(arb_event(), 1..50)) {
        let mut state = InteractionState::Idle;
        for event in events {
            if let Ok(next_state) = transition(state.clone(), event) {
                state = next_state;
            }
            // If transition errors, we just stay in current state for the sake of folding.
        }
    }
}

// =============================================================================
// Exhaustive State Machine Tests (Kani)
// =============================================================================

#[cfg(kani)]
#[kani::proof]
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

#[cfg(kani)]
#[kani::proof]
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

#[cfg(kani)]
#[kani::proof]
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

#[cfg(kani)]
#[kani::proof]
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
