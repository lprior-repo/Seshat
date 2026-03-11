use crate::ui::canvas::domain::types::CanvasPoint;
use std::num::NonZeroU64;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid timing threshold: must be strictly greater than zero")]
    InvalidTimingThreshold,
    #[error("Negative hit padding: touch padding must be >= 0")]
    NegativeHitPadding,
    #[error("Untracked pointer: pointer ID {0} is not currently tracked")]
    UntrackedPointer(u64),
    #[error("Too many simultaneous pointers")]
    TooManyPointers,
    #[error("Duplicate pointer ID")]
    DuplicatePointerId,
    #[error("Postcondition violation")]
    PostconditionViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerType {
    Mouse,
    Touch,
    Pen,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    PanCamera { dx: f64, dy: f64 },
    MoveShape { dx: f64, dy: f64 },
    DoubleTap,
    SingleTap,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputState {
    pub active_pointers: im::HashMap<u64, PointerData>,
    pub last_tap: Option<TapHistory>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            active_pointers: im::HashMap::new(),
            last_tap: None,
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerData {
    pub id: u64,
    pub pointer_type: PointerType,
    pub start_pos: CanvasPoint,
    pub current_pos: CanvasPoint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TapHistory {
    pub pos: CanvasPoint,
    pub time_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NonNegativeF64(f64);

impl NonNegativeF64 {
    pub fn new(val: f64) -> Result<Self, Error> {
        if val < 0.0 || val.is_nan() {
            Err(Error::NegativeHitPadding)
        } else {
            Ok(Self(val))
        }
    }
    pub const fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputConfig {
    pub double_tap_timeout_ms: NonZeroU64,
    pub touch_padding: NonNegativeF64,
    pub base_radius: f64,
}

impl InputConfig {
    pub fn new(
        double_tap_timeout_ms: u64,
        touch_padding: f64,
        base_radius: f64,
    ) -> Result<Self, Error> {
        let timeout =
            NonZeroU64::new(double_tap_timeout_ms).ok_or(Error::InvalidTimingThreshold)?;
        let padding = NonNegativeF64::new(touch_padding)?;
        Ok(Self {
            double_tap_timeout_ms: timeout,
            touch_padding: padding,
            base_radius,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    Down {
        id: u64,
        pointer_type: PointerType,
        pos: CanvasPoint,
        time_ms: u64,
    },
    Move {
        id: u64,
        pos: CanvasPoint,
    },
    Up {
        id: u64,
        pos: CanvasPoint,
        time_ms: u64,
    },
}

pub struct Handle {
    pub center: CanvasPoint,
}

#[derive(Debug, PartialEq)]
pub struct TwoFingerGesture {
    pub id1: u64,
    pub id2: u64,
}

impl TwoFingerGesture {
    pub const fn new(id1: u64, id2: u64) -> Result<Self, Error> {
        if id1 == id2 {
            Err(Error::DuplicatePointerId)
        } else {
            Ok(Self { id1, id2 })
        }
    }
}

pub fn process_pointer_event(
    state: &InputState,
    event: &PointerEvent,
    config: &InputConfig,
) -> Result<(InputState, Vec<Action>), Error> {
    match event {
        PointerEvent::Down {
            id,
            pointer_type,
            pos,
            time_ms,
        } => {
            let too_many =
                state.active_pointers.len() >= 10 && !state.active_pointers.contains_key(id);
            too_many
                .then(|| Err(Error::TooManyPointers))
                .unwrap_or_else(|| {
                    let next_pointers = state.active_pointers.update(
                        *id,
                        PointerData {
                            id: *id,
                            pointer_type: *pointer_type,
                            start_pos: *pos,
                            current_pos: *pos,
                        },
                    );

                    let (actions, next_last_tap) = state.last_tap.as_ref().map_or_else(
                        || {
                            (
                                vec![],
                                Some(TapHistory {
                                    pos: *pos,
                                    time_ms: *time_ms,
                                }),
                            )
                        },
                        |tap| {
                            let dist =
                                ((tap.pos.x - pos.x).powi(2) + (tap.pos.y - pos.y).powi(2)).sqrt();
                            let time_diff = time_ms.saturating_sub(tap.time_ms);

                            if time_diff <= config.double_tap_timeout_ms.get() && dist < 20.0 {
                                (vec![Action::DoubleTap], None)
                            } else {
                                (
                                    vec![],
                                    Some(TapHistory {
                                        pos: *pos,
                                        time_ms: *time_ms,
                                    }),
                                )
                            }
                        },
                    );

                    Ok((
                        InputState {
                            active_pointers: next_pointers,
                            last_tap: next_last_tap,
                        },
                        actions,
                    ))
                })
        }
        PointerEvent::Move { id, pos } => state
            .active_pointers
            .get(id)
            .ok_or(Error::UntrackedPointer(*id))
            .and_then(|pointer| {
                let next_pointers = state.active_pointers.update(
                    *id,
                    PointerData {
                        current_pos: *pos,
                        ..pointer.clone()
                    },
                );

                let dx = pos.x - pointer.current_pos.x;
                let dy = pos.y - pointer.current_pos.y;

                let actions = if next_pointers.len() == 2 {
                    let ids: Vec<u64> = next_pointers.keys().copied().collect();
                    TwoFingerGesture::new(ids[0], ids[1])
                        .map(|_| vec![Action::PanCamera { dx, dy }])?
                } else if next_pointers.len() == 1 {
                    vec![Action::MoveShape { dx, dy }]
                } else {
                    vec![]
                };

                Ok((
                    InputState {
                        active_pointers: next_pointers,
                        last_tap: state.last_tap.clone(),
                    },
                    actions,
                ))
            }),
        PointerEvent::Up {
            id,
            pos: _,
            time_ms: _,
        } => state
            .active_pointers
            .get(id)
            .ok_or(Error::UntrackedPointer(*id))
            .map(|_| {
                let next_pointers = state.active_pointers.without(id);
                (
                    InputState {
                        active_pointers: next_pointers,
                        last_tap: state.last_tap.clone(),
                    },
                    vec![],
                )
            }),
    }
}

pub fn hit_test_handle(
    handle: &Handle,
    point: CanvasPoint,
    pointer_type: PointerType,
    config: &InputConfig,
) -> Result<bool, Error> {
    let dist = ((handle.center.x - point.x).powi(2) + (handle.center.y - point.y).powi(2)).sqrt();
    let padding = match pointer_type {
        PointerType::Touch => config.touch_padding.get(),
        PointerType::Mouse | PointerType::Pen => 0.0,
    };
    Ok(dist <= config.base_radius + padding)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> CanvasPoint {
        CanvasPoint::new(x, y).unwrap()
    }

    fn default_config() -> InputConfig {
        InputConfig::new(300, 10.0, 5.0).unwrap()
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_pan_action_for_two_finger_movement_inp_004() {
        let config = default_config();
        let state = InputState::new();

        let (state, _) = process_pointer_event(
            &state,
            &PointerEvent::Down {
                id: 1,
                pointer_type: PointerType::Touch,
                pos: point(0.0, 0.0),
                time_ms: 0,
            },
            &config,
        )
        .unwrap();
        let (state, _) = process_pointer_event(
            &state,
            &PointerEvent::Down {
                id: 2,
                pointer_type: PointerType::Touch,
                pos: point(10.0, 10.0),
                time_ms: 10,
            },
            &config,
        )
        .unwrap();

        let (_state, actions) = process_pointer_event(
            &state,
            &PointerEvent::Move {
                id: 1,
                pos: point(5.0, 5.0),
            },
            &config,
        )
        .unwrap();
        assert_eq!(actions, vec![Action::PanCamera { dx: 5.0, dy: 5.0 }]);

        // Q1 violation: processing two finger move MUST NOT emit MoveShape action
        assert!(!actions
            .iter()
            .any(|a| matches!(a, Action::MoveShape { .. })));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_high_precision_hit_test_for_stylus_pen_inp_005() {
        let config = default_config(); // radius 5, padding 10
        let handle = Handle {
            center: point(0.0, 0.0),
        };

        // dist 8 is > 5 (base), but < 15 (base + padding)
        let hit = hit_test_handle(&handle, point(8.0, 0.0), PointerType::Pen, &config).unwrap();
        assert!(!hit); // Stylus ignores touch padding
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_double_tap_action_when_tapped_twice_rapidly_inp_006() {
        let config = default_config();
        let state = InputState::new();

        let (state, a1) = process_pointer_event(
            &state,
            &PointerEvent::Down {
                id: 1,
                pointer_type: PointerType::Touch,
                pos: point(0.0, 0.0),
                time_ms: 0,
            },
            &config,
        )
        .unwrap();
        assert_eq!(a1, vec![]);

        let (_state, a2) = process_pointer_event(
            &state,
            &PointerEvent::Down {
                id: 1,
                pointer_type: PointerType::Touch,
                pos: point(1.0, 1.0),
                time_ms: 100,
            },
            &config,
        )
        .unwrap();
        assert_eq!(a2, vec![Action::DoubleTap]);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_hit_success_for_touch_within_expanded_radius_inp_007() {
        let config = default_config(); // radius 5, padding 10
        let handle = Handle {
            center: point(0.0, 0.0),
        };

        // dist 12 is > 5 (base), but < 15 (base + padding)
        let hit = hit_test_handle(&handle, point(12.0, 0.0), PointerType::Touch, &config).unwrap();
        assert!(hit); // Touch uses expanded radius
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_pointer_move_received_for_untracked_id() {
        let config = default_config();
        let state = InputState::new();

        let res = process_pointer_event(
            &state,
            &PointerEvent::Move {
                id: 99,
                pos: point(0.0, 0.0),
            },
            &config,
        );
        assert_eq!(res, Err(Error::UntrackedPointer(99)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_too_many_simultaneous_pointers_active() {
        let config = default_config();
        let mut state = InputState::new();

        for i in 0..10 {
            let (next_state, _) = process_pointer_event(
                &state,
                &PointerEvent::Down {
                    id: i,
                    pointer_type: PointerType::Touch,
                    pos: point(0.0, 0.0),
                    time_ms: 0,
                },
                &config,
            )
            .unwrap();
            state = next_state;
        }

        let res = process_pointer_event(
            &state,
            &PointerEvent::Down {
                id: 10,
                pointer_type: PointerType::Touch,
                pos: point(0.0, 0.0),
                time_ms: 0,
            },
            &config,
        );
        assert_eq!(res, Err(Error::TooManyPointers));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_handles_pointer_up_without_prior_down_gracefully() {
        let config = default_config();
        let state = InputState::new();

        let res = process_pointer_event(
            &state,
            &PointerEvent::Up {
                id: 1,
                pos: point(0.0, 0.0),
                time_ms: 0,
            },
            &config,
        );
        assert_eq!(res, Err(Error::UntrackedPointer(1)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_p1_violation_returns_compile_error_or_invalid_timing_threshold() {
        let res = InputConfig::new(0, 10.0, 5.0);
        assert_eq!(res, Err(Error::InvalidTimingThreshold));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_p2_violation_returns_negative_hit_padding_error() {
        let res = InputConfig::new(300, -5.0, 5.0);
        assert_eq!(res, Err(Error::NegativeHitPadding));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_p3_violation_returns_duplicate_pointer_id_error() {
        let res = TwoFingerGesture::new(1, 1);
        assert_eq!(res, Err(Error::DuplicatePointerId));
    }
}
