use super::types::{CanvasError, CanvasPoint, CanvasVector, RawEvent, SelectionMode};

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasEvent {
    MouseDownTarget {
        point: CanvasPoint,
        mode: SelectionMode,
    },
    MouseDownBackground {
        point: CanvasPoint,
        mode: SelectionMode,
    },
    MouseMove {
        point: CanvasPoint,
    },
    DragMove {
        delta: CanvasVector,
    },
    MouseUp,
}

/// Parses a raw event into a typed canvas event
/// # Errors
/// Returns `CanvasError::UnparseableEvent` if the raw event type is unknown
#[allow(clippy::needless_pass_by_value)]
pub fn parse_event(raw: RawEvent) -> Result<CanvasEvent, CanvasError> {
    let point = CanvasPoint::new(raw.x, raw.y)?;
    let mode = if raw.is_additive {
        SelectionMode::Additive
    } else {
        SelectionMode::Replace
    };

    match raw.event_type.as_str() {
        "mouse_down_target" => Ok(CanvasEvent::MouseDownTarget { point, mode }),
        "mouse_down_background" => Ok(CanvasEvent::MouseDownBackground { point, mode }),
        "mouse_move" => Ok(CanvasEvent::MouseMove { point }),
        "drag_move" => {
            let delta = CanvasVector::new(raw.dx, raw.dy)?;
            Ok(CanvasEvent::DragMove { delta })
        }
        "mouse_up" => Ok(CanvasEvent::MouseUp),
        _ => Err(CanvasError::UnparseableEvent),
    }
}
