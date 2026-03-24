#![allow(clippy::unwrap_used)]
use super::*;
use crate::ui::canvas::domain::types::{CanvasPoint, CanvasVector};

fn point(x: f64, y: f64) -> CanvasPoint {
    CanvasPoint::new(x, y).unwrap()
}

fn default_config() -> InputConfig {
    InputConfig::new(300, 10.0, 5.0).unwrap()
}

#[cfg(kani)]
#[kani::proof]
fn test_returns_pan_action_for_two_finger_movement_inp_004() {
    let config = default_config();
    let state = InputState::new();

    let (state, _) = process_pointer_event(
        &state,
        &PointerEvent::Down {
            id: PointerId(1),
            pointer_type: PointerType::Touch,
            pos: point(0.0, 0.0),
            time: TimeMs(0),
        },
        &config,
    )
    .unwrap();
    let (state, _) = process_pointer_event(
        &state,
        &PointerEvent::Down {
            id: PointerId(2),
            pointer_type: PointerType::Touch,
            pos: point(10.0, 10.0),
            time: TimeMs(10),
        },
        &config,
    )
    .unwrap();

    let (_state, actions) = process_pointer_event(
        &state,
        &PointerEvent::Move {
            id: PointerId(1),
            pos: point(5.0, 5.0),
        },
        &config,
    )
    .unwrap();
    assert_eq!(
        actions,
        vec![Action::PanCamera {
            vector: CanvasVector::new(5.0, 5.0).unwrap()
        }]
    );
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::MoveShape { .. })));
}

#[cfg(kani)]
#[kani::proof]
fn test_returns_high_precision_hit_test_for_stylus_pen_inp_005() {
    let config = default_config();
    let handle = Handle {
        center: point(0.0, 0.0),
    };
    let hit = hit_test_handle(&handle, point(8.0, 0.0), PointerType::Pen, &config).unwrap();
    assert!(!hit);
}

#[cfg(kani)]
#[kani::proof]
fn test_returns_double_tap_action_when_tapped_twice_rapidly_inp_006() {
    let config = default_config();
    let state = InputState::new();

    let (state, a1) = process_pointer_event(
        &state,
        &PointerEvent::Down {
            id: PointerId(1),
            pointer_type: PointerType::Touch,
            pos: point(0.0, 0.0),
            time: TimeMs(0),
        },
        &config,
    )
    .unwrap();
    assert_eq!(a1, vec![]);

    let (_state, a2) = process_pointer_event(
        &state,
        &PointerEvent::Down {
            id: PointerId(1),
            pointer_type: PointerType::Touch,
            pos: point(1.0, 1.0),
            time: TimeMs(100),
        },
        &config,
    )
    .unwrap();
    assert_eq!(a2, vec![Action::DoubleTap]);
}

#[cfg(kani)]
#[kani::proof]
fn test_returns_hit_success_for_touch_within_expanded_radius_inp_007() {
    let config = default_config();
    let handle = Handle {
        center: point(0.0, 0.0),
    };
    let hit = hit_test_handle(&handle, point(12.0, 0.0), PointerType::Touch, &config).unwrap();
    assert!(hit);
}

#[cfg(kani)]
#[kani::proof]
fn test_returns_error_when_pointer_move_received_for_untracked_id() {
    let config = default_config();
    let state = InputState::new();

    let res = process_pointer_event(
        &state,
        &PointerEvent::Move {
            id: PointerId(99),
            pos: point(0.0, 0.0),
        },
        &config,
    );
    assert_eq!(res, Err(Error::UntrackedPointer(PointerId(99))));
}

#[cfg(kani)]
#[kani::proof]
fn test_returns_error_when_too_many_simultaneous_pointers_active() {
    let config = default_config();
    let mut state = InputState::new();

    for i in 0..10 {
        let (next_state, _) = process_pointer_event(
            &state,
            &PointerEvent::Down {
                id: PointerId(i),
                pointer_type: PointerType::Touch,
                pos: point(0.0, 0.0),
                time: TimeMs(0),
            },
            &config,
        )
        .unwrap();
        state = next_state;
    }

    let res = process_pointer_event(
        &state,
        &PointerEvent::Down {
            id: PointerId(10),
            pointer_type: PointerType::Touch,
            pos: point(0.0, 0.0),
            time: TimeMs(0),
        },
        &config,
    );
    assert_eq!(res, Err(Error::TooManyPointers));
}

#[cfg(kani)]
#[kani::proof]
fn test_handles_pointer_up_without_prior_down_gracefully() {
    let config = default_config();
    let state = InputState::new();

    let res = process_pointer_event(
        &state,
        &PointerEvent::Up {
            id: PointerId(1),
            pos: point(0.0, 0.0),
            time: TimeMs(0),
        },
        &config,
    );
    assert_eq!(res, Err(Error::UntrackedPointer(PointerId(1))));
}

#[cfg(kani)]
#[kani::proof]
fn test_p1_violation_returns_compile_error_or_invalid_timing_threshold() {
    let res = InputConfig::new(0, 10.0, 5.0);
    assert_eq!(res, Err(Error::InvalidTimingThreshold));
}

#[cfg(kani)]
#[kani::proof]
fn test_p2_violation_returns_negative_hit_padding_error() {
    let res = InputConfig::new(300, -5.0, 5.0);
    assert_eq!(res, Err(Error::NegativeHitPadding));
}

#[cfg(kani)]
#[kani::proof]
fn test_p3_violation_returns_duplicate_pointer_id_error() {
    let res = TwoFingerGesture::new(PointerId(1), PointerId(1));
    assert_eq!(res, Err(Error::DuplicatePointerId));
}
