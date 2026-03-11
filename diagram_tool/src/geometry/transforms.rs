use crate::geometry::primitives::{Point, Rectangle, AABB};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitTransform {
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[must_use]
pub fn scale_around_anchor(point: Point, anchor: Point, factor: f64) -> Point {
    Point::new(
        (point.x - anchor.x).mul_add(factor, anchor.x),
        (point.y - anchor.y).mul_add(factor, anchor.y),
    )
}

#[must_use]
pub fn rotate_around_center(point: Point, center: Point, angle_radians: f64) -> Point {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();
    let dx = point.x - center.x;
    let dy = point.y - center.y;

    Point::new(
        dx.mul_add(cos, -(dy * sin)) + center.x,
        dx.mul_add(sin, dy * cos) + center.y,
    )
}

#[must_use]
pub fn resize_with_aspect_lock(original_width: f64, original_height: f64, new_width: f64) -> f64 {
    if original_width <= 0.0 {
        return new_width;
    }
    let aspect_ratio = original_height / original_width;
    new_width * aspect_ratio
}

#[must_use]
pub fn scale_then_rotate(
    point: Point,
    anchor: Point,
    scale_factor: f64,
    angle_radians: f64,
) -> Point {
    let scaled = scale_around_anchor(point, anchor, scale_factor);
    rotate_around_center(scaled, anchor, angle_radians)
}

#[must_use]
pub fn fit_to_viewport(
    content: &AABB,
    viewport_width: f64,
    viewport_height: f64,
    padding: f64,
) -> FitTransform {
    let content_width = content.width();
    let content_height = content.height();

    if content_width <= 0.0 || content_height <= 0.0 {
        return FitTransform {
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
        };
    }

    let available_width = 2.0f64.mul_add(-padding, viewport_width);
    let available_height = 2.0f64.mul_add(-padding, viewport_height);

    let scale_x = available_width / content_width;
    let scale_y = available_height / content_height;
    let scale = scale_x.min(scale_y);

    let content_center = content.center();
    let offset_x = content_center.x.mul_add(-scale, viewport_width / 2.0);
    let offset_y = content_center.y.mul_add(-scale, viewport_height / 2.0);

    FitTransform {
        scale,
        offset_x,
        offset_y,
    }
}

#[must_use]
pub const fn clamp_to_min_size(width: f64, height: f64, min_size: f64) -> (f64, f64) {
    let clamped_width = width.max(min_size);
    let clamped_height = height.max(min_size);
    (clamped_width, clamped_height)
}

#[must_use]
pub fn scale_with_flip(width: f64, height: f64, scale_x: f64, scale_y: f64) -> (f64, f64) {
    let new_width = (width * scale_x).abs();
    let new_height = (height * scale_y).abs();
    (new_width, new_height)
}

#[must_use]
pub fn scale_with_clamp(
    width: f64,
    height: f64,
    scale_x: f64,
    scale_y: f64,
    min_size: f64,
) -> (f64, f64) {
    let new_width = if scale_x < 0.0 {
        min_size
    } else {
        (width * scale_x).max(min_size)
    };
    let new_height = if scale_y < 0.0 {
        min_size
    } else {
        (height * scale_y).max(min_size)
    };
    (new_width, new_height)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    NorthWest,
    NorthEast,
    SouthEast,
    SouthWest,
}

#[must_use]
pub fn get_corner_point(rect: &Rectangle, corner: Corner) -> Point {
    match corner {
        Corner::NorthWest => Point::new(rect.x, rect.y),
        Corner::NorthEast => Point::new(rect.x + rect.width, rect.y),
        Corner::SouthEast => Point::new(rect.x + rect.width, rect.y + rect.height),
        Corner::SouthWest => Point::new(rect.x, rect.y + rect.height),
    }
}

#[must_use]
pub fn scale_rect_around_corner(rect: &Rectangle, corner: Corner, factor: f64) -> Rectangle {
    let anchor = get_corner_point(rect, corner);
    let nw = scale_around_anchor(get_corner_point(rect, Corner::NorthWest), anchor, factor);
    let se = scale_around_anchor(get_corner_point(rect, Corner::SouthEast), anchor, factor);

    let new_width = (se.x - nw.x).abs();
    let new_height = (se.y - nw.y).abs();

    let (new_x, new_y) = match corner {
        Corner::NorthWest => (anchor.x, anchor.y),
        Corner::NorthEast => (anchor.x - new_width, anchor.y),
        Corner::SouthEast => (anchor.x - new_width, anchor.y - new_height),
        Corner::SouthWest => (anchor.x, anchor.y - new_height),
    };

    Rectangle::new(new_x, new_y, new_width, new_height)
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

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
            px.is_finite()
                && py.is_finite()
                && ax.is_finite()
                && ay.is_finite()
                && factor.is_finite(),
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
            w.is_finite()
                && h.is_finite()
                && sx.is_finite()
                && sy.is_finite()
                && min_size.is_finite(),
        );

        let (nw, nh) = scale_with_clamp(w, h, sx, sy, min_size);

        assert!(nw >= min_size);
        assert!(nh >= min_size);
    }
}
