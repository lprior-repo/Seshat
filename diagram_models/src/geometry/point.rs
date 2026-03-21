/// Represents a 2D point in diagram coordinate space
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new Point from x, y coordinates
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Create Point at origin (0.0, 0.0)
    #[must_use]
    pub const fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Euclidean distance to another point
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx.mul_add(dx, dy * dy)).sqrt()
    }

    /// Midpoint between this and another point
    #[must_use]
    pub const fn midpoint(self, other: Self) -> Self {
        Self {
            x: f64::midpoint(self.x, other.x),
            y: f64::midpoint(self.y, other.y),
        }
    }

    /// Linear interpolation toward another point
    #[must_use]
    pub fn interpolate(self, other: Self, t: f64) -> Self {
        Self {
            x: self.x.mul_add(1.0 - t, other.x * t),
            y: self.y.mul_add(1.0 - t, other.y * t),
        }
    }
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Self::new(x, y)
    }
}

impl From<Point> for (f64, f64) {
    fn from(p: Point) -> Self {
        (p.x, p.y)
    }
}
