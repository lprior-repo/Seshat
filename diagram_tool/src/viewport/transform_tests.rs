use super::*;
use crate::geometry::AABB;
use proptest::prelude::*;

const TOLERANCE: f64 = 1e-9;

#[cfg(kani)]
#[kani::proof]
fn test_screen_to_world_origin() {
    let result = screen_to_world(0.0, 0.0, 0.0, 0.0, 1.0);
    assert!((result.x - 0.0).abs() < TOLERANCE);
    assert!((result.y - 0.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_screen_to_world_with_camera() {
    // Given: camera at (100, 200), zoom 2.0
    // When: convert screen (400, 300)
    // Then: world = (100 + 400/2, 200 + 300/2) = (300, 350)
    let result = screen_to_world(400.0, 300.0, 100.0, 200.0, 2.0);
    assert!((result.x - 300.0).abs() < TOLERANCE);
    assert!((result.y - 350.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_world_to_screen_origin() {
    let result = world_to_screen(0.0, 0.0, 0.0, 0.0, 1.0);
    assert!((result.x - 0.0).abs() < TOLERANCE);
    assert!((result.y - 0.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_world_to_screen_with_camera() {
    // Given: camera at (100, 200), zoom 2.0
    // When: convert world (300, 350)
    // Then: screen = ((300-100)*2, (350-200)*2) = (400, 300)
    let result = world_to_screen(300.0, 350.0, 100.0, 200.0, 2.0);
    assert!((result.x - 400.0).abs() < TOLERANCE);
    assert!((result.y - 300.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_roundtrip_transform() {
    let screen_x = 400.0;
    let screen_y = 300.0;
    let camera_x = 100.0;
    let camera_y = 200.0;
    let zoom = 2.0;

    let world = screen_to_world(screen_x, screen_y, camera_x, camera_y, zoom);
    let screen_back = world_to_screen(world.x, world.y, camera_x, camera_y, zoom);

    assert!((screen_back.x - screen_x).abs() < TOLERANCE);
    assert!((screen_back.y - screen_y).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_screen_to_world_invalid_zoom() {
    // Invalid zoom should use default of 1.0
    let result = screen_to_world(100.0, 100.0, 0.0, 0.0, 0.0);
    assert!((result.x - 100.0).abs() < TOLERANCE);

    let result = screen_to_world(100.0, 100.0, 0.0, 0.0, -1.0);
    assert!((result.x - 100.0).abs() < TOLERANCE);

    let result = screen_to_world(100.0, 100.0, 0.0, 0.0, f64::NAN);
    assert!((result.x - 100.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_fit_scale_basic() {
    let content = AABB::new(0.0, 0.0, 100.0, 100.0);
    let scale = fit_scale(&content, 200.0, 200.0, 0.0);
    assert!((scale - 2.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_fit_scale_with_padding() {
    let content = AABB::new(0.0, 0.0, 100.0, 100.0);
    let scale = fit_scale(&content, 120.0, 120.0, 10.0);
    // Available: 100x100, Content: 100x100, Scale: 1.0
    assert!((scale - 1.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_fit_scale_preserves_aspect() {
    let content = AABB::new(0.0, 0.0, 200.0, 100.0); // 2:1 aspect
    let scale = fit_scale(&content, 100.0, 100.0, 0.0);
    // Should fit width: 100/200 = 0.5
    // Should fit height: 100/100 = 1.0
    // Use minimum to fit both: 0.5
    assert!((scale - 0.5).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_screen_to_world_uses_epsilon_threshold() {
    let result = screen_to_world(100.0, 100.0, 0.0, 0.0, f64::EPSILON / 2.0);
    assert!((result.x - 100.0).abs() < TOLERANCE);

    let result = screen_to_world(100.0, 100.0, 0.0, 0.0, f64::EPSILON * 2.0);
    assert!((result.x - (100.0 / (f64::EPSILON * 2.0))).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_world_to_screen_uses_epsilon_threshold() {
    let result = world_to_screen(100.0, 100.0, 0.0, 0.0, f64::EPSILON / 2.0);
    assert!((result.x - 100.0).abs() < TOLERANCE);

    let result = world_to_screen(100.0, 100.0, 0.0, 0.0, f64::EPSILON * 2.0);
    assert!((result.x - (100.0 * f64::EPSILON * 2.0)).abs() < TOLERANCE);
}

proptest! {
    #[test]
    fn test_fuzz_screen_to_world(
        sx in prop::num::f64::ANY,
        sy in prop::num::f64::ANY,
        cx in prop::num::f64::ANY,
        cy in prop::num::f64::ANY,
        zoom in prop::num::f64::ANY
    ) {
        let _ = screen_to_world(sx, sy, cx, cy, zoom);
    }

    #[test]
    fn test_fuzz_world_to_screen(
        wx in prop::num::f64::ANY,
        wy in prop::num::f64::ANY,
        cx in prop::num::f64::ANY,
        cy in prop::num::f64::ANY,
        zoom in prop::num::f64::ANY
    ) {
        let _ = world_to_screen(wx, wy, cx, cy, zoom);
    }

    #[test]
    fn test_fuzz_fit_scale(
        min_x in prop::num::f64::ANY,
        min_y in prop::num::f64::ANY,
        max_x in prop::num::f64::ANY,
        max_y in prop::num::f64::ANY,
        vw in prop::num::f64::ANY,
        vh in prop::num::f64::ANY,
        padding in prop::num::f64::ANY
    ) {
        let (min_x, max_x) = if min_x.is_nan() || max_x.is_nan() { (0.0, 0.0) } else { (min_x.min(max_x), min_x.max(max_x)) };
        let (min_y, max_y) = if min_y.is_nan() || max_y.is_nan() { (0.0, 0.0) } else { (min_y.min(max_y), min_y.max(max_y)) };
        let aabb = AABB::new(min_x, min_y, max_x, max_y);
        let _ = fit_scale(&aabb, vw, vh, padding);
    }

    #[test]
    fn test_fuzz_center_camera_for_content(
        min_x in prop::num::f64::ANY,
        min_y in prop::num::f64::ANY,
        max_x in prop::num::f64::ANY,
        max_y in prop::num::f64::ANY,
        scale in prop::num::f64::ANY,
        vw in prop::num::f64::ANY,
        vh in prop::num::f64::ANY
    ) {
        let (min_x, max_x) = if min_x.is_nan() || max_x.is_nan() { (0.0, 0.0) } else { (min_x.min(max_x), min_x.max(max_x)) };
        let (min_y, max_y) = if min_y.is_nan() || max_y.is_nan() { (0.0, 0.0) } else { (min_y.min(max_y), min_y.max(max_y)) };
        let aabb = AABB::new(min_x, min_y, max_x, max_y);
        let _ = center_camera_for_content(&aabb, scale, vw, vh);
    }
}
