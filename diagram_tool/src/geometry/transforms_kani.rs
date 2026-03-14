//! Kani proofs for geometry transformations.
//!
//! This module contains formal verification proofs for the transformation functions.
//! These proofs are only compiled when running with Kani model checker.

use crate::geometry::primitives::{Point, Rectangle, AABB};
use crate::geometry::transforms::{
    clamp_to_min_size, resize_with_aspect_lock, rotate_around_center, scale_around_anchor,
    scale_then_rotate, scale_with_clamp,
};

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-4
}

fn approx_eq_pt(a: Point, b: Point) -> bool {
    approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
}

#[kani::proof]
fn verify_rotate_360() {
    let px: f64 = kani::any();
    let py: f64 = kani::any();
    let cx: f64 = kani::any();
    let cy: f64 = kani::any();

    kani::assume(px.is_finite() && py.is_finite() && cx.is_finite() && cy.is_finite());
    kani::assume(px.abs() < 1000.0 && py.abs() < 1000.0);
    kani::assume(cx.abs() < 1000.0 && cy.abs() < 1000.0);

    let p = Point::new(px, py);
    let c = Point::new(cx, cy);

    let p_rotated = rotate_around_center(p, c, std::f64::consts::TAU);
    assert!(approx_eq_pt(p, p_rotated));
}

#[kani::proof]
fn verify_scale_inverse_yields_identity() {
    let px: f64 = kani::any();
    let py: f64 = kani::any();
    let ax: f64 = kani::any();
    let ay: f64 = kani::any();
    let factor: f64 = kani::any();

    kani::assume(
        px.is_finite() && py.is_finite() && ax.is_finite() && ay.is_finite() && factor.is_finite(),
    );
    kani::assume(factor.abs() > 0.01 && factor.abs() < 100.0);
    kani::assume(px.abs() < 1000.0 && py.abs() < 1000.0);
    kani::assume(ax.abs() < 1000.0 && ay.abs() < 1000.0);

    let p = Point::new(px, py);
    let a = Point::new(ax, ay);

    let scaled = scale_around_anchor(p, a, factor);
    let restored = scale_around_anchor(scaled, a, 1.0 / factor);

    assert!(approx_eq_pt(p, restored));
}

#[kani::proof]
fn verify_scale_identity() {
    let px: f64 = kani::any();
    let py: f64 = kani::any();
    let ax: f64 = kani::any();
    let ay: f64 = kani::any();

    kani::assume(px.is_finite() && py.is_finite() && ax.is_finite() && ay.is_finite());

    let p = Point::new(px, py);
    let a = Point::new(ax, ay);

    let scaled = scale_around_anchor(p, a, 1.0);
    assert!(approx_eq_pt(p, scaled));
}

#[kani::proof]
fn verify_scale_then_rotate_composition() {
    let px: f64 = kani::any();
    let py: f64 = kani::any();
    let ax: f64 = kani::any();
    let ay: f64 = kani::any();
    let scale: f64 = kani::any();
    let angle: f64 = kani::any();

    kani::assume(
        px.is_finite()
            && py.is_finite()
            && ax.is_finite()
            && ay.is_finite()
            && scale.is_finite()
            && angle.is_finite(),
    );
    kani::assume(scale.abs() < 100.0);
    kani::assume(px.abs() < 1000.0 && py.abs() < 1000.0);
    kani::assume(ax.abs() < 1000.0 && ay.abs() < 1000.0);

    let p = Point::new(px, py);
    let a = Point::new(ax, ay);

    let step1 = scale_around_anchor(p, a, scale);
    let step2 = rotate_around_center(step1, a, angle);

    let combined = scale_then_rotate(p, a, scale, angle);

    assert!(approx_eq_pt(step2, combined));
}

#[kani::proof]
fn verify_clamp_to_min_size() {
    let width: f64 = kani::any();
    let height: f64 = kani::any();
    let min_size: f64 = kani::any();

    kani::assume(width.is_finite() && height.is_finite() && min_size.is_finite());

    let (cw, ch) = clamp_to_min_size(width, height, min_size);

    assert!(cw >= min_size);
    assert!(ch >= min_size);
}

#[kani::proof]
fn verify_resize_with_aspect_lock() {
    let ow: f64 = kani::any();
    let oh: f64 = kani::any();
    let nw: f64 = kani::any();

    kani::assume(ow.is_finite() && oh.is_finite() && nw.is_finite());
    kani::assume(ow > 0.001);
    kani::assume(nw.abs() > 0.001);
    kani::assume(ow < 1000.0 && oh.abs() < 1000.0 && nw.abs() < 1000.0);

    let nh = resize_with_aspect_lock(ow, oh, nw);

    let original_ratio = oh / ow;
    let new_ratio = nh / nw;
    assert!(approx_eq(original_ratio, new_ratio));
}

#[kani::proof]
fn verify_scale_with_clamp_bounds() {
    let w: f64 = kani::any();
    let h: f64 = kani::any();
    let sx: f64 = kani::any();
    let sy: f64 = kani::any();
    let min_size: f64 = kani::any();

    kani::assume(
        w.is_finite() && h.is_finite() && sx.is_finite() && sy.is_finite() && min_size.is_finite(),
    );

    let (nw, nh) = scale_with_clamp(w, h, sx, sy, min_size);

    assert!(nw >= min_size);
    assert!(nh >= min_size);
}
