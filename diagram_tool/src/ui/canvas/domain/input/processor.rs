use crate::ui::canvas::domain::input::config::InputConfig;
use crate::ui::canvas::domain::input::types::{
    Action, Error, Handle, InputState, PointerData, PointerEvent, PointerId, PointerType,
    TapHistory, TwoFingerGesture,
};
use crate::ui::canvas::domain::types::{CanvasPoint, CanvasVector};

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
            time,
        } => {
            let too_many =
                state.active_pointers.len() >= 10 && !state.active_pointers.contains_key(id);
            if too_many {
                Err(Error::TooManyPointers)
            } else {
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
                                time: *time,
                            }),
                        )
                    },
                    |tap| {
                        let dist = (tap.pos.x - pos.x).hypot(tap.pos.y - pos.y);
                        let time_diff = time.0.saturating_sub(tap.time.0);

                        if time_diff <= config.double_tap_timeout_ms.get() && dist < 20.0 {
                            (vec![Action::DoubleTap], None)
                        } else {
                            (
                                vec![],
                                Some(TapHistory {
                                    pos: *pos,
                                    time: *time,
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
            }
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
                let vector = CanvasVector::new(dx, dy).unwrap_or(CanvasVector { dx: 0.0, dy: 0.0 });

                let actions = if next_pointers.len() == 2 {
                    let ids: Vec<PointerId> = next_pointers.keys().copied().collect();
                    TwoFingerGesture::new(ids[0], ids[1])
                        .map(|_| vec![Action::PanCamera { vector }])?
                } else if next_pointers.len() == 1 {
                    vec![Action::MoveShape { vector }]
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
        PointerEvent::Up { id, .. } => state
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
    let dist = (handle.center.x - point.x).hypot(handle.center.y - point.y);
    let padding = match pointer_type {
        PointerType::Touch => config.touch_padding.get(),
        PointerType::Mouse | PointerType::Pen => 0.0,
    };
    Ok(dist <= config.base_radius + padding)
}
