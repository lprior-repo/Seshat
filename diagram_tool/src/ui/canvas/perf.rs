#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::OrderedFloat;

const ZOOM_MIN: f64 = 0.1;
const ZOOM_MAX: f64 = 4.0;
const VIEWPORT_EPSILON: f64 = 0.5;

#[derive(Clone, Copy)]
pub(super) struct WheelInput {
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
pub(super) fn to_canvas_coords(
    client_x: f64,
    client_y: f64,
    cam_x: f64,
    cam_y: f64,
    zoom: f64,
) -> (f64, f64) {
    ((client_x / zoom) + cam_x, (client_y / zoom) + cam_y)
}

#[must_use]
pub(super) fn to_screen_coords(
    world_x: f64,
    world_y: f64,
    cam_x: f64,
    cam_y: f64,
    zoom: f64,
) -> (f64, f64) {
    ((world_x - cam_x) * zoom, (world_y - cam_y) * zoom)
}

#[must_use]
pub(super) fn wheel_transform(input: WheelInput) -> (f64, f64, f64) {
    if input.shift_pan {
        let actual_dx = if input.dx.abs() > f64::EPSILON {
            input.dx
        } else {
            input.dy
        };
        (
            input.camera_x.0 + (actual_dx / input.zoom.0),
            input.camera_y.0,
            input.zoom.0,
        )
    } else if input.zoom_gesture {
        let zoom_factor = input.dy.mul_add(-0.01, 1.0);
        let new_zoom = (input.zoom.0 * zoom_factor).clamp(ZOOM_MIN, ZOOM_MAX);
        let factor = input.zoom.0 / new_zoom;
        let (wx, wy) = to_canvas_coords(
            input.client_x,
            input.client_y,
            input.camera_x.0,
            input.camera_y.0,
            input.zoom.0,
        );
        (
            (wx - input.camera_x.0).mul_add(-factor, wx),
            (wy - input.camera_y.0).mul_add(-factor, wy),
            new_zoom,
        )
    } else if input.discrete_wheel {
        let zoom_factor = if input.dy > 0.0 { 0.9 } else { 1.1 };
        let new_zoom = (input.zoom.0 * zoom_factor).clamp(ZOOM_MIN, ZOOM_MAX);
        let factor = input.zoom.0 / new_zoom;
        let (wx, wy) = to_canvas_coords(
            input.client_x,
            input.client_y,
            input.camera_x.0,
            input.camera_y.0,
            input.zoom.0,
        );
        (
            (wx - input.camera_x.0).mul_add(-factor, wx),
            (wy - input.camera_y.0).mul_add(-factor, wy),
            new_zoom,
        )
    } else {
        (
            input.camera_x.0 + (input.dx / input.zoom.0),
            input.camera_y.0 + (input.dy / input.zoom.0),
            input.zoom.0,
        )
    }
}

#[must_use]
pub(super) fn wheel_update(
    input: WheelInput,
) -> Option<(OrderedFloat, OrderedFloat, OrderedFloat)> {
    let (next_x, next_y, next_zoom) = wheel_transform(input);

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

#[must_use]
pub(super) const fn normalize_viewport(width: f64, height: f64) -> (f64, f64) {
    (width.max(1.0), height.max(1.0))
}

#[must_use]
pub(super) fn viewport_changed(current: (f64, f64), next: (f64, f64)) -> bool {
    (current.0 - next.0).abs() > VIEWPORT_EPSILON || (current.1 - next.1).abs() > VIEWPORT_EPSILON
}
