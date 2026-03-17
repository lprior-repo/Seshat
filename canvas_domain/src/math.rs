#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::{CanvasCoord, ScreenCoord};
pub use canvas_math::{safe_zoom, sanitize_zoom, within};

#[must_use]
pub fn screen_to_canvas(
    client: ScreenCoord,
    camera: CanvasCoord,
    zoom: f64,
) -> Option<CanvasCoord> {
    let valid_zoom = safe_zoom(zoom)?;
    Some(CanvasCoord(
        (client.x() / valid_zoom) + camera.x(),
        (client.y() / valid_zoom) + camera.y(),
    ))
}

#[must_use]
pub fn canvas_to_screen(world: CanvasCoord, camera: CanvasCoord, zoom: f64) -> Option<ScreenCoord> {
    let valid_zoom = safe_zoom(zoom)?;
    Some(ScreenCoord(
        (world.x() - camera.x()) * valid_zoom,
        (world.y() - camera.y()) * valid_zoom,
    ))
}
