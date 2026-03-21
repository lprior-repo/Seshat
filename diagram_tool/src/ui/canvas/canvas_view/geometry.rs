#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(clippy::imprecise_flops)]
#![allow(clippy::suboptimal_flops)]

// Re-export canonical Point from diagram_models
pub use crate::geometry::Point;

// Segment is still needed locally as it depends on Point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub start: Point,
    pub end: Point,
}

impl Segment {
    #[must_use]
    pub const fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    #[allow(clippy::similar_names)]
    #[must_use]
    pub fn distance_to_point(self, p: Point) -> f64 {
        let dx_pt = p.x - self.start.x;
        let dy_pt = p.y - self.start.y;
        let dx_seg = self.end.x - self.start.x;
        let dy_seg = self.end.y - self.start.y;
        let dot = dx_pt.mul_add(dx_seg, dy_pt * dy_seg);
        let len_sq = dx_seg.mul_add(dx_seg, dy_seg * dy_seg);
        let mut param = if len_sq == 0.0 { -1.0 } else { dot / len_sq };
        param = param.clamp(0.0, 1.0);
        let closest = Point::new(
            self.start.x + (param * dx_seg),
            self.start.y + (param * dy_seg),
        );
        p.distance_to(closest)
    }
}

#[must_use]
pub fn dist_to_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    Segment::new(Point::new(x1, y1), Point::new(x2, y2)).distance_to_point(Point::new(px, py))
}

#[must_use]
pub fn rect_ray_intersection(cx: f64, cy: f64, w: f64, h: f64, tx: f64, ty: f64) -> (f64, f64) {
    let dx = tx - cx;
    let dy = ty - cy;
    if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
        return (cx, cy);
    }

    let scale_x = (w / 2.0) / dx.abs();
    let scale_y = (h / 2.0) / dy.abs();
    let scale = scale_x.min(scale_y);

    let padded_scale = scale.mul_add(-5.0 / (dx * dx + dy * dy).sqrt(), scale);
    let final_scale = padded_scale.max(0.0);

    (cx + dx * final_scale, cy + dy * final_scale)
}

#[must_use]
pub fn quadratic_control(sx: f64, sy: f64, tx: f64, ty: f64) -> (f64, f64) {
    let start = Point::new(sx, sy);
    let target = Point::new(tx, ty);
    let dx = target.x - start.x;
    let dy = target.y - start.y;
    let mid = start.midpoint(target);
    (dy.mul_add(-0.25, mid.x), dx.mul_add(0.25, mid.y))
}

#[must_use]
pub fn interpolate_polyline_point(points: &[(f64, f64)], t: f64) -> (f64, f64) {
    if points.len() < 2 {
        return points.first().copied().unwrap_or((0.0, 0.0));
    }

    let segments = points
        .windows(2)
        .map(|window| {
            let start = Point::from(window[0]);
            let end = Point::from(window[1]);
            (start, end, start.distance_to(end))
        })
        .collect::<Vec<_>>();

    let total_len = segments.iter().fold(0.0, |acc, (_, _, len)| acc + len);
    if total_len <= f64::EPSILON {
        return points[0];
    }

    let target = total_len * t;
    let mut traversed = 0.0;
    for (start, end, len) in segments {
        if len <= f64::EPSILON {
            continue;
        }
        if traversed + len >= target {
            let local_t = ((target - traversed) / len).clamp(0.0, 1.0);
            return start.interpolate(end, local_t).into();
        }
        traversed += len;
    }

    points.last().copied().unwrap_or(points[0])
}

#[must_use]
pub fn quadratic_bezier_point(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let p0 = Point::from(p0);
    let p1 = Point::from(p1);
    let p2 = Point::from(p2);
    let one_minus_t = 1.0 - t;
    let one_minus_t_2 = one_minus_t * one_minus_t;
    let t_2 = t * t;
    let blend = 2.0 * one_minus_t * t;
    let x = p0.x.mul_add(one_minus_t_2, blend.mul_add(p1.x, t_2 * p2.x));
    let y = p0.y.mul_add(one_minus_t_2, blend.mul_add(p1.y, t_2 * p2.y));
    (x, y)
}

#[cfg(test)]
#[path = "geometry_proptests.rs"]
mod proptests;
