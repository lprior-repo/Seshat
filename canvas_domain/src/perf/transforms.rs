use crate::math::{canvas_to_screen, screen_to_canvas};
use crate::{CanvasCoord, ScreenCoord};

pub const VIEWPORT_EPSILON: f64 = 0.5;

#[must_use]
pub fn to_canvas_coords(client: ScreenCoord, camera: CanvasCoord, zoom: f64) -> CanvasCoord {
    screen_to_canvas(client, camera, zoom).unwrap_or(CanvasCoord(
        client.x() + camera.x(),
        client.y() + camera.y(),
    ))
}

#[must_use]
pub fn to_screen_coords(world: CanvasCoord, camera: CanvasCoord, zoom: f64) -> ScreenCoord {
    canvas_to_screen(world, camera, zoom).unwrap_or(ScreenCoord(
        (world.x() - camera.x()) * zoom,
        (world.y() - camera.y()) * zoom,
    ))
}

#[must_use]
pub const fn normalize_viewport(width: f64, height: f64) -> (f64, f64) {
    (width.max(1.0), height.max(1.0))
}

#[must_use]
pub fn viewport_changed(current: (f64, f64), next: (f64, f64)) -> bool {
    (current.0 - next.0).abs() > VIEWPORT_EPSILON || (current.1 - next.1).abs() > VIEWPORT_EPSILON
}
