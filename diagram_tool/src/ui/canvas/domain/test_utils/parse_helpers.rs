//! Test helpers for parse/event tests
#![allow(clippy::unwrap_used)]
use crate::ui::canvas::domain::{
    CanvasEvent, CanvasPoint, CanvasVector, InteractionState, RawEvent, SelectionMode,
};

/// Helper to create a valid CanvasPoint
pub fn pt() -> CanvasPoint {
    CanvasPoint::new(0.0, 0.0).unwrap()
}

/// Helper to create a valid CanvasVector
pub fn vec() -> CanvasVector {
    CanvasVector::new(0.0, 0.0).unwrap()
}

/// Helper to create a DragState for testing
pub fn drag_state(start: CanvasPoint) -> crate::ui::canvas::domain::DragState {
    crate::ui::canvas::domain::DragState {
        start,
        current: start,
        cumulative_offset: CanvasVector::new(0.0, 0.0).unwrap(),
    }
}

/// Helper to create a RawEvent with given coordinates
pub fn raw_event(x: f64, y: f64, event_type: &str) -> RawEvent {
    RawEvent {
        event_type: event_type.into(),
        x,
        y,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    }
}

/// Helper to create a RawEvent with coordinates and delta
pub fn raw_event_with_delta(
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    event_type: &str,
    is_additive: bool,
) -> RawEvent {
    RawEvent {
        event_type: event_type.into(),
        x,
        y,
        dx,
        dy,
        is_additive,
    }
}

/// Returns all CanvasEvent variants for exhaustive testing
pub fn all_events() -> Vec<CanvasEvent> {
    let p = pt();
    let v = vec();
    vec![
        CanvasEvent::MouseDownTarget {
            point: p,
            mode: SelectionMode::Replace,
        },
        CanvasEvent::MouseDownBackground {
            point: p,
            mode: SelectionMode::Replace,
        },
        CanvasEvent::MouseMove { point: p },
        CanvasEvent::DragMove { delta: v },
        CanvasEvent::MouseUp,
        CanvasEvent::TouchDownTarget {
            point: p,
            mode: SelectionMode::Replace,
        },
        CanvasEvent::TouchDownBackground {
            point: p,
            mode: SelectionMode::Replace,
        },
        CanvasEvent::TouchMove { point: p, delta: v },
        CanvasEvent::TouchUp,
    ]
}
