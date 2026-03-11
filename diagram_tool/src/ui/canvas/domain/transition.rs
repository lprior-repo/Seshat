use super::canvas_event::CanvasEvent;
use super::interaction_state::{apply_drag_delta, DragState, InteractionState};
use super::types::{CanvasError, CanvasVector};

const fn state_name(state: &InteractionState) -> &'static str {
    match state {
        InteractionState::Idle => "Idle",
        InteractionState::Hovering { .. } => "Hovering",
        InteractionState::Dragging { .. } => "Dragging",
        InteractionState::Selecting { .. } => "Selecting",
    }
}

const fn event_name(event: &CanvasEvent) -> &'static str {
    match event {
        CanvasEvent::MouseDownTarget { .. } => "MouseDownTarget",
        CanvasEvent::MouseDownBackground { .. } => "MouseDownBackground",
        CanvasEvent::MouseMove { .. } => "MouseMove",
        CanvasEvent::DragMove { .. } => "DragMove",
        CanvasEvent::MouseUp => "MouseUp",
        CanvasEvent::TouchDownTarget { .. } => "TouchDownTarget",
        CanvasEvent::TouchDownBackground { .. } => "TouchDownBackground",
        CanvasEvent::TouchMove { .. } => "TouchMove",
        CanvasEvent::TouchUp => "TouchUp",
    }
}

/// Transitions the interaction state machine
/// # Errors
/// Returns `CanvasError::InvalidTransition` if the event is not valid for the current state
#[allow(clippy::match_same_arms)]
pub fn transition(
    state: InteractionState,
    event: CanvasEvent,
) -> Result<InteractionState, CanvasError> {
    let state_str = state_name(&state).to_string();
    let event_str = event_name(&event).to_string();

    let invalid = || {
        Err(CanvasError::InvalidTransition {
            state: state_str.clone(),
            event: event_str.clone(),
        })
    };

    match (state, event) {
        (InteractionState::Idle, CanvasEvent::MouseMove { point }) => {
            Ok(InteractionState::Hovering { point })
        }
        (InteractionState::Idle, CanvasEvent::TouchMove { .. }) => Ok(InteractionState::Idle),
        (
            InteractionState::Idle,
            CanvasEvent::MouseDownTarget { point, .. } | CanvasEvent::TouchDownTarget { point, .. },
        ) => Ok(InteractionState::Dragging {
            drag: DragState {
                start: point,
                current: point,
                cumulative_offset: CanvasVector::new(0.0, 0.0)?,
            },
        }),
        (
            InteractionState::Idle,
            CanvasEvent::MouseDownBackground { point, mode }
            | CanvasEvent::TouchDownBackground { point, mode },
        ) => Ok(InteractionState::Selecting {
            start: point,
            current: point,
            mode,
        }),
        (InteractionState::Idle, CanvasEvent::MouseUp | CanvasEvent::TouchUp) => {
            Ok(InteractionState::Idle)
        }

        (InteractionState::Hovering { .. }, CanvasEvent::MouseMove { point }) => {
            Ok(InteractionState::Hovering { point })
        }
        (
            InteractionState::Hovering { .. },
            CanvasEvent::MouseDownTarget { point, .. } | CanvasEvent::TouchDownTarget { point, .. },
        ) => Ok(InteractionState::Dragging {
            drag: DragState {
                start: point,
                current: point,
                cumulative_offset: CanvasVector::new(0.0, 0.0)?,
            },
        }),
        (
            InteractionState::Hovering { .. },
            CanvasEvent::MouseDownBackground { point, mode }
            | CanvasEvent::TouchDownBackground { point, mode },
        ) => Ok(InteractionState::Selecting {
            start: point,
            current: point,
            mode,
        }),
        (InteractionState::Hovering { .. }, CanvasEvent::MouseUp | CanvasEvent::TouchUp) => {
            Ok(InteractionState::Idle)
        }

        (
            InteractionState::Dragging { mut drag },
            CanvasEvent::DragMove { delta } | CanvasEvent::TouchMove { delta, .. },
        ) => {
            apply_drag_delta(&mut drag, delta)?;
            Ok(InteractionState::Dragging { drag })
        }
        (InteractionState::Dragging { .. }, CanvasEvent::MouseUp | CanvasEvent::TouchUp) => {
            Ok(InteractionState::Idle)
        }
        (InteractionState::Dragging { drag }, CanvasEvent::MouseMove { point: _ }) => {
            // Treat MouseMove as just updating current pos without offset, or it might be an invalid transition
            // For now, dragging uses drag move
            Ok(InteractionState::Dragging { drag })
        }

        (
            InteractionState::Selecting { start, mode, .. },
            CanvasEvent::MouseMove { point } | CanvasEvent::TouchMove { point, .. },
        ) => Ok(InteractionState::Selecting {
            start,
            current: point,
            mode,
        }),
        (InteractionState::Selecting { .. }, CanvasEvent::MouseUp | CanvasEvent::TouchUp) => {
            Ok(InteractionState::Idle)
        }

        _ => invalid(),
    }
}
