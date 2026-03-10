use super::types::{CanvasError, CanvasPoint, CanvasVector, SelectionMode};

#[derive(Debug, Clone, PartialEq)]
pub struct DragState {
    pub start: CanvasPoint,
    pub current: CanvasPoint,
    pub cumulative_offset: CanvasVector,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionState {
    Idle,
    Hovering {
        point: CanvasPoint,
    },
    Dragging {
        drag: DragState,
    },
    Selecting {
        start: CanvasPoint,
        current: CanvasPoint,
        mode: SelectionMode,
    },
}

/// Applies a drag delta to the given drag state
/// # Errors
/// Returns `CanvasError::CoordinateOutOfBounds` if the delta is infinite or NaN
#[allow(clippy::similar_names)]
pub fn apply_drag_delta(drag: &mut DragState, delta: CanvasVector) -> Result<(), CanvasError> {
    if !delta.dx.is_finite() || !delta.dy.is_finite() {
        return Err(CanvasError::CoordinateOutOfBounds);
    }

    let new_dx = drag.cumulative_offset.dx + delta.dx;
    let new_dy = drag.cumulative_offset.dy + delta.dy;

    drag.cumulative_offset = CanvasVector::new(new_dx, new_dy)?;
    drag.current = CanvasPoint::new(drag.current.x + delta.dx, drag.current.y + delta.dy)?;

    Ok(())
}
