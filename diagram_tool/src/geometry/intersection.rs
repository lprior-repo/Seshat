#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use crate::geometry::primitives::{Point, AABB};

const EPSILON: f64 = 1e-10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IntersectionError {
    #[error("Invalid endpoint: NaN or Infinity")]
    InvalidEndpoint,
    #[error("Degenerate line: start and end points are identical")]
    DegenerateLine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub start: Point,
    pub end: Point,
}

impl LineSegment {
    pub fn new(start: Point, end: Point) -> Result<Self, IntersectionError> {
        if !start.x.is_finite() || !start.y.is_finite() {
            return Err(IntersectionError::InvalidEndpoint);
        }
        if !end.x.is_finite() || !end.y.is_finite() {
            return Err(IntersectionError::InvalidEndpoint);
        }
        if (start.x - end.x).abs() < EPSILON && (start.y - end.y).abs() < EPSILON {
            return Err(IntersectionError::DegenerateLine);
        }
        Ok(Self { start, end })
    }

    #[must_use]
    pub const fn new_unchecked(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn direction(&self) -> Point {
        Point::new(self.end.x - self.start.x, self.end.y - self.start.y)
    }
}

#[must_use]
pub fn line_line_intersects(a: LineSegment, b: LineSegment) -> bool {
    let (ax1, ay1, ax2, ay2) = (a.start.x, a.start.y, a.end.x, a.end.y);
    let (bx1, by1, bx2, by2) = (b.start.x, b.start.y, b.end.x, b.end.y);

    let da = (ax2 - ax1, ay2 - ay1);
    let db = (bx2 - bx1, by2 - by1);
    let dp = (ax1 - bx1, ay1 - by1);

    let cross = da.0.mul_add(db.1, -(da.1 * db.0));

    if cross.abs() < EPSILON {
        let dot = da.0.mul_add(db.0, da.1 * db.1);
        if dot.abs() < EPSILON {
            let ab1 = (bx1 - ax1, by1 - ay1);
            let cross_b = da.0.mul_add(ab1.1, -(da.1 * ab1.0));
            if cross_b.abs() < EPSILON {
                let t0 = ab1.0.mul_add(da.0, ab1.1 * da.1);
                let t1 = t0 + db.0.mul_add(da.0, db.1 * da.1);
                let (t_min, t_max) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
                let denom = da.0.mul_add(da.0, da.1 * da.1);
                if t_max >= 0.0 && t_min <= denom {
                    return true;
                }
            }
        }
        return false;
    }

    let t = db.0.mul_add(dp.1, -(db.1 * dp.0)) / cross;
    let u = da.0.mul_add(dp.1, -(da.1 * dp.0)) / cross;

    (0.0 - EPSILON..=1.0 + EPSILON).contains(&t) && (0.0 - EPSILON..=1.0 + EPSILON).contains(&u)
}

#[must_use]
pub fn line_line_intersection(a: LineSegment, b: LineSegment) -> Option<Point> {
    let (ax1, ay1, ax2, ay2) = (a.start.x, a.start.y, a.end.x, a.end.y);
    let (bx1, by1, bx2, by2) = (b.start.x, b.start.y, b.end.x, b.end.y);

    let da = (ax2 - ax1, ay2 - ay1);
    let db = (bx2 - bx1, by2 - by1);
    let dp = (ax1 - bx1, ay1 - by1);

    let cross = da.0.mul_add(db.1, -(da.1 * db.0));

    if cross.abs() < EPSILON {
        if da.0.mul_add(db.0, da.1 * db.1).abs() < EPSILON {
            return None;
        }
        let ab1 = (bx1 - ax1, by1 - ay1);
        if da.0.mul_add(ab1.1, -(da.1 * ab1.0)).abs() < EPSILON {
            let t0 = ab1.0.mul_add(da.0, ab1.1 * da.1);
            let t1 = t0 + db.0.mul_add(da.0, db.1 * da.1);
            if t0 > t1 {
                return Some(Point::new(
                    ax1 + da.0 * t1 / da.0.mul_add(da.0, da.1 * da.1),
                    ay1 + da.1 * t1 / da.0.mul_add(da.0, da.1 * da.1),
                ));
            }
            return Some(Point::new(
                ax1 + da.0 * t0 / da.0.mul_add(da.0, da.1 * da.1),
                ay1 + da.1 * t0 / da.0.mul_add(da.0, da.1 * da.1),
            ));
        }
        return None;
    }

    let t = db.0.mul_add(dp.1, -(db.1 * dp.0)) / cross;
    let u = da.0.mul_add(dp.1, -(da.1 * dp.0)) / cross;

    if (0.0 - EPSILON..=1.0 + EPSILON).contains(&t) && (0.0 - EPSILON..=1.0 + EPSILON).contains(&u)
    {
        Some(Point::new(da.0.mul_add(t, ax1), da.1.mul_add(t, ay1)))
    } else {
        None
    }
}

#[must_use]
pub fn line_rect_intersects(line: LineSegment, rect: &AABB) -> bool {
    !line_rect_intersections(line, rect).is_empty()
}

#[must_use]
pub fn line_rect_intersections(line: LineSegment, rect: &AABB) -> Vec<Point> {
    let top = LineSegment::new_unchecked(
        Point::new(rect.min_x, rect.min_y),
        Point::new(rect.max_x, rect.min_y),
    );
    let bottom = LineSegment::new_unchecked(
        Point::new(rect.min_x, rect.max_y),
        Point::new(rect.max_x, rect.max_y),
    );
    let left = LineSegment::new_unchecked(
        Point::new(rect.min_x, rect.min_y),
        Point::new(rect.min_x, rect.max_y),
    );
    let right = LineSegment::new_unchecked(
        Point::new(rect.max_x, rect.min_y),
        Point::new(rect.max_x, rect.max_y),
    );

    let mut points = Vec::new();

    if let Some(p) = line_line_intersection(line, top) {
        points.push(p);
    }
    if let Some(p) = line_line_intersection(line, bottom) {
        if points.is_empty()
            || (p.x - points[0].x).abs() > EPSILON
            || (p.y - points[0].y).abs() > EPSILON
        {
            points.push(p);
        }
    }
    if let Some(p) = line_line_intersection(line, left) {
        let is_duplicate = points.iter().any(|existing| {
            (p.x - existing.x).abs() < EPSILON && (p.y - existing.y).abs() < EPSILON
        });
        if !is_duplicate {
            points.push(p);
        }
    }
    if let Some(p) = line_line_intersection(line, right) {
        let is_duplicate = points.iter().any(|existing| {
            (p.x - existing.x).abs() < EPSILON && (p.y - existing.y).abs() < EPSILON
        });
        if !is_duplicate {
            points.push(p);
        }
    }

    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_segment_new_rejects_nan() {
        let result = LineSegment::new(Point::new(f64::NAN, 0.0), Point::new(1.0, 1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_line_segment_new_rejects_infinity() {
        let result = LineSegment::new(Point::new(0.0, f64::INFINITY), Point::new(1.0, 1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_line_segment_new_rejects_zero_length() {
        let result = LineSegment::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_line_line_intersects_crossing() {
        let a = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 10.0));
        let b = LineSegment::new_unchecked(Point::new(0.0, 10.0), Point::new(10.0, 0.0));
        assert!(line_line_intersects(a, b));
    }

    #[test]
    fn test_line_line_intersects_parallel() {
        let a = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 0.0));
        let b = LineSegment::new_unchecked(Point::new(0.0, 5.0), Point::new(10.0, 5.0));
        assert!(!line_line_intersects(a, b));
    }

    #[test]
    fn test_line_line_intersection_crossing() -> Result<(), &'static str> {
        let a = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 10.0));
        let b = LineSegment::new_unchecked(Point::new(0.0, 10.0), Point::new(10.0, 0.0));
        let result = line_line_intersection(a, b);
        assert!(result.is_some());
        let p = result.ok_or("expected Some")?;
        assert!((p.x - 5.0).abs() < EPSILON);
        assert!((p.y - 5.0).abs() < EPSILON);
        Ok(())
    }

    #[test]
    fn test_line_line_intersection_parallel() {
        let a = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 0.0));
        let b = LineSegment::new_unchecked(Point::new(0.0, 5.0), Point::new(10.0, 5.0));
        assert!(line_line_intersection(a, b).is_none());
    }

    #[test]
    fn test_line_rect_intersects_crossing() {
        let line = LineSegment::new_unchecked(Point::new(0.0, 50.0), Point::new(100.0, 50.0));
        let rect = AABB::new(30.0, 30.0, 70.0, 70.0);
        assert!(line_rect_intersects(line, &rect));
    }

    #[test]
    fn test_line_rect_intersects_outside() {
        let line = LineSegment::new_unchecked(Point::new(0.0, 0.0), Point::new(10.0, 10.0));
        let rect = AABB::new(20.0, 20.0, 30.0, 30.0);
        assert!(!line_rect_intersects(line, &rect));
    }

    #[test]
    fn test_line_rect_intersections_two_points() {
        let line = LineSegment::new_unchecked(Point::new(0.0, 50.0), Point::new(100.0, 50.0));
        let rect = AABB::new(30.0, 30.0, 70.0, 70.0);
        let points = line_rect_intersections(line, &rect);
        assert_eq!(points.len(), 2);
    }
}
