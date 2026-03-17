use crate::math::safe_zoom;
use crate::{CanvasCoord, ScreenCoord};
use diagram_models::document::OrderedFloat;

use super::transforms::to_canvas_coords;

pub const ZOOM_MIN: f64 = 0.1;
pub const ZOOM_MAX: f64 = 4.0;

#[derive(Clone, Copy, Debug)]
pub struct WheelInput {
    pub camera_x: OrderedFloat,
    pub camera_y: OrderedFloat,
    pub zoom: OrderedFloat,
    pub client_x: f64,
    pub client_y: f64,
    pub dx: f64,
    pub dy: f64,
    pub zoom_gesture: bool,
    pub shift_pan: bool,
    pub discrete_wheel: bool,
}

#[must_use]
pub fn wheel_transform(input: WheelInput) -> (f64, f64, f64) {
    let current_zoom = safe_zoom(input.zoom.0).unwrap_or(1.0);
    let wheel_delta = if input.shift_pan {
        if input.dx.abs() > f64::EPSILON {
            input.dx
        } else {
            input.dy
        }
    } else if input.dy.abs() > f64::EPSILON {
        input.dy
    } else {
        input.dx
    };

    let next_zoom = if input.discrete_wheel {
        let factor = if wheel_delta > 0.0 { 0.95 } else { 1.05 };
        (current_zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX)
    } else {
        let intensity = if input.zoom_gesture { 0.005 } else { 0.002 };
        let factor = wheel_delta.mul_add(-intensity, 1.0);
        (current_zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX)
    };

    if (next_zoom - current_zoom).abs() <= f64::EPSILON {
        (input.camera_x.0, input.camera_y.0, current_zoom)
    } else {
        let factor = current_zoom / next_zoom;
        let wx_wy = to_canvas_coords(
            ScreenCoord(input.client_x, input.client_y),
            CanvasCoord(input.camera_x.0, input.camera_y.0),
            current_zoom,
        );
        let wx = wx_wy.x();
        let wy = wx_wy.y();
        (
            (wx - input.camera_x.0).mul_add(-factor, wx),
            (wy - input.camera_y.0).mul_add(-factor, wy),
            next_zoom,
        )
    }
}

#[must_use]
pub fn wheel_update(input: WheelInput) -> Option<(OrderedFloat, OrderedFloat, OrderedFloat)> {
    if !input.client_x.is_finite()
        || !input.client_y.is_finite()
        || !input.dx.is_finite()
        || !input.dy.is_finite()
        || !input.camera_x.0.is_finite()
        || !input.camera_y.0.is_finite()
    {
        return None;
    }

    let (next_x, next_y, next_zoom) = wheel_transform(input);

    if !next_x.is_finite() || !next_y.is_finite() || !next_zoom.is_finite() {
        return None;
    }

    if (next_x - input.camera_x.0).abs() <= f64::EPSILON
        && (next_y - input.camera_y.0).abs() <= f64::EPSILON
        && (next_zoom - input.zoom.0).abs() <= f64::EPSILON
    {
        None
    } else {
        Some((
            OrderedFloat(next_x),
            OrderedFloat(next_y),
            OrderedFloat(next_zoom),
        ))
    }
}
