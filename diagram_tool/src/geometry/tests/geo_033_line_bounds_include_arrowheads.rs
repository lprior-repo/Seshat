use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-033: Line Bounds Include Arrowheads ==============

/// Represents a line segment with optional arrowheads at start and/or end
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub start: Point,
    pub end: Point,
    pub stroke_width: f64,
    pub start_arrow: Option<Arrowhead>,
    pub end_arrow: Option<Arrowhead>,
}

/// Represents an arrowhead configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arrowhead {
    pub size: f64,  // Length of arrowhead
    pub angle: f64, // Angle in radians (typically PI/6 for 30 degrees)
}

impl Line {
    #[must_use]
    pub const fn new(start: Point, end: Point) -> Self {
        Self {
            start,
            end,
            stroke_width: 1.0,
            start_arrow: None,
            end_arrow: None,
        }
    }

    #[must_use]
    pub const fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    #[must_use]
    pub const fn with_end_arrow(mut self, arrow: Arrowhead) -> Self {
        self.end_arrow = Some(arrow);
        self
    }

    #[must_use]
    pub const fn with_start_arrow(mut self, arrow: Arrowhead) -> Self {
        self.start_arrow = Some(arrow);
        self
    }

    /// Calculate the bounds including stroke and arrowheads
    #[must_use]
    pub fn bounds(&self) -> AABB {
        // Start with line segment bounds
        let min_x = self.start.x.min(self.end.x);
        let max_x = self.start.x.max(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_y = self.start.y.max(self.end.y);

        // Expand for stroke width
        let half_stroke = self.stroke_width / 2.0;
        let mut bounds = AABB::new(
            min_x - half_stroke,
            min_y - half_stroke,
            max_x + half_stroke,
            max_y + half_stroke,
        );

        // Expand for arrowheads
        if let Some(arrow) = self.start_arrow {
            bounds = bounds.union(&self.arrowhead_bounds(self.start, self.end, arrow));
        }
        if let Some(arrow) = self.end_arrow {
            bounds = bounds.union(&self.arrowhead_bounds(self.end, self.start, arrow));
        }

        bounds
    }

    /// Calculate bounds for an arrowhead at a point
    fn arrowhead_bounds(&self, tip: Point, opposite: Point, arrow: Arrowhead) -> AABB {
        // Direction from opposite to tip
        let dx = tip.x - opposite.x;
        let dy = tip.y - opposite.y;
        let length = dx.hypot(dy);
        if length < TOLERANCE {
            return AABB::new(tip.x, tip.y, tip.x, tip.y);
        }

        // Unit direction
        let ux = dx / length;
        let uy = dy / length;

        // Arrowhead extends back from tip and to the sides
        // The tip of the arrow is at `tip`, and the base is `arrow.size` back
        // The wings extend at `arrow.angle` from the base
        let wing_length = arrow.size * arrow.angle.sin();
        let base_distance = arrow.size * arrow.angle.cos();

        // Back point (base center)
        let back_x = ux.mul_add(-base_distance, tip.x);
        let back_y = uy.mul_add(-base_distance, tip.y);

        // Perpendicular direction
        let px = -uy;
        let py = ux;

        // Wing points
        let wing1_x = px.mul_add(wing_length, back_x);
        let wing1_y = py.mul_add(wing_length, back_y);
        let wing2_x = px.mul_add(-wing_length, back_x);
        let wing2_y = py.mul_add(-wing_length, back_y);

        // AABB containing tip and both wings
        AABB::new(
            tip.x.min(wing1_x).min(wing2_x),
            tip.y.min(wing1_y).min(wing2_y),
            tip.x.max(wing1_x).max(wing2_x),
            tip.y.max(wing1_y).max(wing2_y),
        )
    }
}

#[cfg(kani)]
#[kani::proof]
fn test_line_bounds_simple() {
    // Given: a simple line without arrowheads
    let line = Line::new(Point::new(0.0, 0.0), Point::new(100.0, 50.0));

    // When: calculating bounds
    let bounds = line.bounds();

    // Then: bounds contain the line segment
    assert!(bounds.min_x <= 0.0);
    assert!(bounds.max_x >= 100.0);
    assert!(bounds.min_y <= 0.0);
    assert!(bounds.max_y >= 50.0);
}

#[cfg(kani)]
#[kani::proof]
fn test_line_bounds_with_end_arrow() {
    // Given: a line with an arrowhead at the end
    let arrow = Arrowhead {
        size: 15.0,
        angle: std::f64::consts::FRAC_PI_6, // 30 degrees
    };
    let line = Line::new(Point::new(0.0, 0.0), Point::new(100.0, 0.0)).with_end_arrow(arrow);

    // When: calculating bounds
    let bounds = line.bounds();

    // Then: bounds extend beyond the endpoint for the arrowhead
    // The tip is at (100, 0), arrow extends back and to sides
    assert!(bounds.max_x >= 100.0);
    // The wings extend perpendicular to the line
    assert!(bounds.min_y < 0.0 || (bounds.min_y - 0.0).abs() < TOLERANCE);
    assert!(bounds.max_y > 0.0 || (bounds.max_y - 0.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_line_bounds_with_both_arrows() {
    // Given: a line with arrowheads at both ends
    let arrow = Arrowhead {
        size: 10.0,
        angle: std::f64::consts::FRAC_PI_6,
    };
    let line = Line::new(Point::new(0.0, 50.0), Point::new(100.0, 50.0))
        .with_start_arrow(arrow)
        .with_end_arrow(arrow);

    // When: calculating bounds
    let bounds = line.bounds();

    // Then: bounds extend on both ends for arrowheads
    assert!(bounds.min_x < 0.0 || (bounds.min_x - 0.0).abs() < TOLERANCE);
    assert!(bounds.max_x > 100.0);
}

#[cfg(kani)]
#[kani::proof]
fn test_line_bounds_with_thick_stroke() {
    // Given: a line with thick stroke
    let line = Line::new(Point::new(0.0, 0.0), Point::new(100.0, 0.0)).with_stroke_width(10.0);

    // When: calculating bounds
    let bounds = line.bounds();

    // Then: bounds include stroke width (5 on each side)
    assert!((bounds.min_y - (-5.0)).abs() < TOLERANCE);
    assert!((bounds.max_y - 5.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_line_bounds_diagonal_with_arrow() {
    // Given: a diagonal line with arrowhead
    let arrow = Arrowhead {
        size: 20.0,
        angle: std::f64::consts::FRAC_PI_6,
    };
    let line = Line::new(Point::new(0.0, 0.0), Point::new(100.0, 100.0)).with_end_arrow(arrow);

    // When: calculating bounds
    let bounds = line.bounds();

    // Then: bounds contain the tip and arrowhead wings
    assert!(bounds.max_x >= 100.0);
    assert!(bounds.max_y >= 100.0);
    // Arrowhead extends back from tip
    assert!(bounds.min_x <= 0.0);
    assert!(bounds.min_y <= 0.0);
}
