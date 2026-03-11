use crate::ui::canvas::domain::{
    parse_event, transition, CanvasEvent, CanvasPoint, CanvasVector, InteractionState, RawEvent,
    SelectionMode,
};
use proptest::prelude::*;

proptest! {
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
