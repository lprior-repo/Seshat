//! Kani proofs for geometry operations.
//!
//! This module contains formal verification proofs for the operations functions.
//! These proofs are only compiled when running with Kani model checker.

use crate::geometry::operations::{
    compute_subgraph_bounds, hit_test_rect, safe_bounds, screen_to_world, selection_center,
    snap_horizontal, world_to_screen, zoom_at_pointer,
};
use crate::geometry::primitives::{Point, Rectangle};

#[kani::proof]
fn verify_safe_bounds() {
    let min_x: f64 = kani::any();
    let min_y: f64 = kani::any();
    let max_x: f64 = kani::any();
    let max_y: f64 = kani::any();

    kani::assume(min_x.is_finite());
    kani::assume(min_y.is_finite());
    kani::assume(max_x.is_finite());
    kani::assume(max_y.is_finite());

    let result = safe_bounds(min_x, min_y, max_x, max_y);
    assert!(result.is_ok());
    let aabb = result.unwrap();

    // Invariant: min <= max
    assert!(aabb.min_x <= aabb.max_x);
    assert!(aabb.min_y <= aabb.max_y);

    // Invariant: coordinates are preserved
    assert!(aabb.min_x == min_x.min(max_x));
    assert!(aabb.max_x == min_x.max(max_x));
    assert!(aabb.min_y == min_y.min(max_y));
    assert!(aabb.max_y == min_y.max(max_y));
}

#[kani::proof]
fn verify_zoom_at_pointer() {
    let cx: f64 = kani::any();
    let cy: f64 = kani::any();
    let px: f64 = kani::any();
    let py: f64 = kani::any();
    let factor: f64 = kani::any();

    kani::assume(cx.is_finite());
    kani::assume(cy.is_finite());
    kani::assume(px.is_finite());
    kani::assume(py.is_finite());
    kani::assume(factor.is_finite());

    let view_center = Point::new(cx, cy);
    let pointer = Point::new(px, py);

    let new_center = zoom_at_pointer(view_center, pointer, factor);

    // Invariant: if factor is 1, center doesn't change
    if factor == 1.0 {
        assert!(new_center.x == cx);
        assert!(new_center.y == cy);
    }

    // Invariant: if pointer is at center, center doesn't change
    if cx == px && cy == py {
        assert!(new_center.x == cx);
        assert!(new_center.y == cy);
    }
}

#[kani::proof]
fn verify_snap_horizontal() {
    let line_y: f64 = kani::any();
    let target1: f64 = kani::any();
    let target2: f64 = kani::any();
    let tolerance: f64 = kani::any();

    kani::assume(line_y.is_finite());
    kani::assume(target1.is_finite());
    kani::assume(target2.is_finite());
    kani::assume(tolerance.is_finite());
    kani::assume(tolerance >= 0.0);

    let targets = [target1, target2];
    let result = snap_horizontal(line_y, &targets, tolerance);

    if let Some(snapped) = result {
        // Must have snapped to one of our targets
        assert!(snapped == target1 || snapped == target2);
        // Must be within tolerance
        assert!((line_y - snapped).abs() <= tolerance);
    }
}

#[kani::proof]
fn verify_compute_subgraph_bounds() {
    let x1: f64 = kani::any();
    let y1: f64 = kani::any();
    let w1: f64 = kani::any();
    let h1: f64 = kani::any();

    let x2: f64 = kani::any();
    let y2: f64 = kani::any();
    let w2: f64 = kani::any();
    let h2: f64 = kani::any();

    kani::assume(x1.is_finite() && y1.is_finite() && w1.is_finite() && h1.is_finite());
    kani::assume(x2.is_finite() && y2.is_finite() && w2.is_finite() && h2.is_finite());
    kani::assume(w1 >= 0.0 && h1 >= 0.0);
    kani::assume(w2 >= 0.0 && h2 >= 0.0);

    let children = [(x1, y1, w1, h1), (x2, y2, w2, h2)];
    let result = compute_subgraph_bounds(children);

    assert!(result.is_some());
    if let Some((bx, by, bw, bh)) = result {
        // Bounds must encompass child 1
        assert!(bx <= x1);
        assert!(by <= y1);
        assert!(bx + bw >= x1 + w1);
        assert!(by + bh >= y1 + h1);

        // Bounds must encompass child 2
        assert!(bx <= x2);
        assert!(by <= y2);
        assert!(bx + bw >= x2 + w2);
        assert!(by + bh >= y2 + h2);

        // Dimensions must be non-negative
        assert!(bw >= 0.0);
        assert!(bh >= 0.0);
    }
}

#[kani::proof]
fn verify_world_screen_conversion() {
    let world_x: f64 = kani::any();
    let world_y: f64 = kani::any();
    let cam_x: f64 = kani::any();
    let cam_y: f64 = kani::any();
    let zoom: f64 = kani::any();

    kani::assume(world_x.is_finite());
    kani::assume(world_y.is_finite());
    kani::assume(cam_x.is_finite());
    kani::assume(cam_y.is_finite());
    kani::assume(zoom.is_finite() && zoom > 0.0);

    let world = Point::new(world_x, world_y);
    let camera = Point::new(cam_x, cam_y);

    let screen = world_to_screen(world, camera, zoom);

    // Verify invariant: if zoom is 1 and camera is at origin, world == screen
    if zoom == 1.0 && cam_x == 0.0 && cam_y == 0.0 {
        assert!(screen.x == world_x);
        assert!(screen.y == world_y);
    }
}

#[kani::proof]
fn verify_selection_center() {
    let x1: f64 = kani::any();
    let y1: f64 = kani::any();
    let x2: f64 = kani::any();
    let y2: f64 = kani::any();

    kani::assume(x1.is_finite() && y1.is_finite());
    kani::assume(x2.is_finite() && y2.is_finite());

    let points = [Point::new(x1, y1), Point::new(x2, y2)];
    let center = selection_center(&points);

    assert!(center.x.is_finite());
    assert!(center.y.is_finite());
    assert!(center.x == (x1 + x2) / 2.0);
    assert!(center.y == (y1 + y2) / 2.0);
}

#[kani::proof]
fn verify_hit_test_rect() {
    let px: f64 = kani::any();
    let py: f64 = kani::any();
    let rx: f64 = kani::any();
    let ry: f64 = kani::any();
    let rw: f64 = kani::any();
    let rh: f64 = kani::any();
    let margin: f64 = kani::any();

    kani::assume(px.is_finite() && py.is_finite());
    kani::assume(rx.is_finite() && ry.is_finite());
    kani::assume(rw.is_finite() && rh.is_finite());
    kani::assume(rw >= 0.0 && rh >= 0.0);
    kani::assume(margin.is_finite() && margin >= 0.0);

    let point = Point::new(px, py);
    let rect = Rectangle::new(rx, ry, rw, rh);

    let is_hit = hit_test_rect(point, &rect, margin);

    // Invariant: if point is exactly the center, it should hit
    let cx = rx + rw / 2.0;
    let cy = ry + rh / 2.0;
    if px == cx && py == cy {
        assert!(is_hit);
    }
}
