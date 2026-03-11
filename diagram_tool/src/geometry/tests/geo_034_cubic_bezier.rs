use super::super::*;
use super::*;
use crate::geometry::tests::geo_034_quadratic_bezier::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    pub start: Point,
    pub control1: Point,
    pub control2: Point,
    pub end: Point,
    pub stroke_width: f64,
}

impl CubicBezier {
    #[must_use]
    pub const fn new(start: Point, control1: Point, control2: Point, end: Point) -> Self {
        Self {
            start,
            control1,
            control2,
            end,
            stroke_width: 1.0,
        }
    }

    /// Evaluate the curve at parameter t (0..=1)
    #[must_use]
    pub fn evaluate(&self, t: f64) -> Point {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        Point::new(
            mt3 * self.start.x
                + 3.0 * mt2 * t * self.control1.x
                + 3.0 * mt * t2 * self.control2.x
                + t3 * self.end.x,
            mt3 * self.start.y
                + 3.0 * mt2 * t * self.control1.y
                + 3.0 * mt * t2 * self.control2.y
                + t3 * self.end.y,
        )
    }

    /// Calculate approximate bounds by sampling
    #[must_use]
    pub fn bounds(&self) -> AABB {
        let samples = 30;
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for i in 0..=samples {
            let t = f64::from(i) / f64::from(samples);
            let p = self.evaluate(t);
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let half_stroke = self.stroke_width / 2.0;
        AABB::new(
            min_x - half_stroke,
            min_y - half_stroke,
            max_x + half_stroke,
            max_y + half_stroke,
        )
    }
}

#[test]
fn test_quadratic_bezier_bounds_simple() {
    // Given: a simple quadratic Bezier (arc)
    let curve = QuadraticBezier::new(
        Point::new(0.0, 0.0),
        Point::new(50.0, 100.0), // Control point creates upward arc
        Point::new(100.0, 0.0),
    );

    // When: calculating bounds
    let bounds = curve.bounds();

    // Then: bounds contain the curve including the control point influence
    assert!(bounds.min_x <= 0.0);
    assert!(bounds.max_x >= 100.0);
    assert!(bounds.max_y >= 50.0); // Curve goes above the line between endpoints
}

#[test]
fn test_quadratic_bezier_bounds_straight_line() {
    // Given: a quadratic Bezier that's essentially a straight line
    let curve = QuadraticBezier::new(
        Point::new(0.0, 0.0),
        Point::new(50.0, 0.0), // Control point on the line
        Point::new(100.0, 0.0),
    );

    // When: calculating bounds
    let bounds = curve.bounds();

    // Then: bounds are essentially the line segment
    assert!((bounds.min_x - 0.0).abs() < 1.0);
    assert!((bounds.max_x - 100.0).abs() < 1.0);
}

#[test]
fn test_quadratic_bezier_bounds_with_stroke() {
    // Given: a curve with thick stroke
    let curve = QuadraticBezier::new(
        Point::new(0.0, 0.0),
        Point::new(50.0, 50.0),
        Point::new(100.0, 0.0),
    )
    .with_stroke_width(10.0);

    // When: calculating bounds
    let bounds = curve.bounds();

    // Then: bounds include stroke width
    assert!(bounds.min_y < 0.0); // Expanded for stroke
}

#[test]
fn test_quadratic_bezier_tight_bounds() {
    // Given: a curve
    let curve = QuadraticBezier::new(
        Point::new(0.0, 0.0),
        Point::new(50.0, 100.0),
        Point::new(100.0, 0.0),
    );

    // When: calculating tight bounds
    let tight = curve.tight_bounds();
    let sampled = curve.bounds();

    // Then: tight bounds should be close to sampled bounds
    // Both should contain the curve's actual extent
    assert!(tight.max_y > 0.0);
    // Tight bounds should be at most as large as sampled
    assert!(tight.max_y <= sampled.max_y + 1.0);
}

#[test]
fn test_cubic_bezier_bounds_simple() {
    // Given: a simple cubic Bezier (S-curve)
    let curve = CubicBezier::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 100.0),   // First control goes up
        Point::new(100.0, -50.0), // Second control goes down
        Point::new(100.0, 50.0),
    );

    // When: calculating bounds
    let bounds = curve.bounds();

    // Then: bounds contain the curve
    assert!(bounds.min_x <= 0.0);
    assert!(bounds.max_x >= 100.0);
    // S-curve should extend beyond endpoints vertically
    assert!(bounds.max_y > 50.0);
}

#[test]
fn test_cubic_bezier_bounds_complex() {
    // Given: a complex cubic Bezier with multiple extrema
    let curve = CubicBezier::new(
        Point::new(0.0, 50.0),
        Point::new(25.0, 0.0),
        Point::new(75.0, 100.0),
        Point::new(100.0, 50.0),
    );

    // When: calculating bounds
    let bounds = curve.bounds();

    // Then: bounds contain all curve points
    assert!(bounds.min_x <= 0.0);
    assert!(bounds.max_x >= 100.0);
    // Verify by sampling
    for i in 0..=10 {
        let t = f64::from(i) / 10.0;
        let p = curve.evaluate(t);
        assert!(p.x >= bounds.min_x - TOLERANCE);
        assert!(p.x <= bounds.max_x + TOLERANCE);
        assert!(p.y >= bounds.min_y - TOLERANCE);
        assert!(p.y <= bounds.max_y + TOLERANCE);
    }
}
