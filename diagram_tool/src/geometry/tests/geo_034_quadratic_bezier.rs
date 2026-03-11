use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-034: Curved Connector Bounds ==============

/// Represents a quadratic Bezier curve
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadraticBezier {
    pub start: Point,
    pub control: Point,
    pub end: Point,
    pub stroke_width: f64,
}

impl QuadraticBezier {
    #[must_use]
    pub const fn new(start: Point, control: Point, end: Point) -> Self {
        Self {
            start,
            control,
            end,
            stroke_width: 1.0,
        }
    }

    #[must_use]
    pub const fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Evaluate the curve at parameter t (0..=1)
    #[must_use]
    pub fn evaluate(&self, t: f64) -> Point {
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        Point::new(
            mt2 * self.start.x + 2.0 * mt * t * self.control.x + t2 * self.end.x,
            mt2 * self.start.y + 2.0 * mt * t * self.control.y + t2 * self.end.y,
        )
    }

    /// Calculate approximate bounds by sampling the curve
    #[must_use]
    pub fn bounds(&self) -> AABB {
        let samples = 20;
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

        // Expand for stroke width
        let half_stroke = self.stroke_width / 2.0;
        AABB::new(
            min_x - half_stroke,
            min_y - half_stroke,
            max_x + half_stroke,
            max_y + half_stroke,
        )
    }

    /// Calculate tight bounds using derivative analysis
    #[must_use]
    pub fn tight_bounds(&self) -> AABB {
        // For quadratic Bezier, extrema occur at endpoints or where derivative is zero
        // B'(t) = 2(1-t)(C-P0) + 2t(P2-C)
        // Setting derivative to zero: t = (P0 - C) / (P0 - 2C + P2)

        let mut min_x = self.start.x.min(self.end.x);
        let mut max_x = self.start.x.max(self.end.x);
        let mut min_y = self.start.y.min(self.end.y);
        let mut max_y = self.start.y.max(self.end.y);

        // Check x extrema
        let denom_x = self.start.x - 2.0 * self.control.x + self.end.x;
        if denom_x.abs() > TOLERANCE {
            let t = (self.start.x - self.control.x) / denom_x;
            if (0.0..=1.0).contains(&t) {
                let p = self.evaluate(t);
                min_x = min_x.min(p.x);
                max_x = max_x.max(p.x);
            }
        }

        // Check y extrema
        let denom_y = self.start.y - 2.0 * self.control.y + self.end.y;
        if denom_y.abs() > TOLERANCE {
            let t = (self.start.y - self.control.y) / denom_y;
            if (0.0..=1.0).contains(&t) {
                let p = self.evaluate(t);
                min_y = min_y.min(p.y);
                max_y = max_y.max(p.y);
            }
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
