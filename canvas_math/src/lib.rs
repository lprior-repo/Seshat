#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

pub const MIN_ZOOM: f64 = 0.1;
pub const MAX_ZOOM: f64 = 4.0;
pub const ZOOM_IN_FACTOR: f64 = 1.25;
pub const ZOOM_OUT_FACTOR: f64 = 0.8;

#[must_use]
pub fn safe_zoom(zoom: f64) -> Option<f64> {
    (zoom.is_finite() && zoom > f64::EPSILON).then_some(zoom)
}

#[must_use]
pub fn within(subgraph: (f64, f64, f64, f64), node: (f64, f64, f64, f64)) -> bool {
    let (sx, sy, sw, sh) = subgraph;
    let (nx, ny, nw, nh) = node;
    nx >= sx && ny >= sy && nx + nw <= sx + sw && ny + nh <= sy + sh
}

#[must_use]
pub fn screen_to_canvas(
    client_x: f64,
    client_y: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
) -> Option<(f64, f64)> {
    let valid_zoom = safe_zoom(zoom)?;
    Some((
        (client_x / valid_zoom) + camera_x,
        (client_y / valid_zoom) + camera_y,
    ))
}

#[must_use]
pub fn canvas_to_screen(
    world_x: f64,
    world_y: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
) -> Option<(f64, f64)> {
    let valid_zoom = safe_zoom(zoom)?;
    Some((
        (world_x - camera_x) * valid_zoom,
        (world_y - camera_y) * valid_zoom,
    ))
}

#[must_use]
pub fn sanitize_zoom(zoom: f64, min: f64, max: f64) -> Option<f64> {
    safe_zoom(zoom).map(|valid| valid.clamp(min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_zoom_rejects_invalid_values() {
        assert_eq!(safe_zoom(0.0), None);
        assert_eq!(safe_zoom(-1.0), None);
        assert_eq!(safe_zoom(f64::NAN), None);
        assert_eq!(safe_zoom(f64::INFINITY), None);
        assert_eq!(safe_zoom(1.0), Some(1.0));
    }

    #[test]
    fn coordinate_transforms_round_trip() {
        let canvas = screen_to_canvas(400.0, 300.0, 100.0, 200.0, 2.0);
        assert_eq!(canvas, Some((300.0, 350.0)));

        let screen = canvas.and_then(|(x, y)| canvas_to_screen(x, y, 100.0, 200.0, 2.0));
        assert_eq!(screen, Some((400.0, 300.0)));
    }

    #[test]
    fn within_checks_containment() {
        assert!(within((0.0, 0.0, 10.0, 10.0), (2.0, 2.0, 3.0, 3.0)));
        assert!(!within((0.0, 0.0, 10.0, 10.0), (9.0, 9.0, 3.0, 3.0)));
    }
}

#[cfg(test)]
mod proptests;

#[cfg(kani)]
mod kani_proofs;
