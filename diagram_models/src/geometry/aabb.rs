use super::point::Point;

/// Represents an axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl AABB {
    #[must_use]
    pub const fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
        }
    }

    /// Create an AABB from x, y, width, height (axis-aligned rectangle)
    /// Works with negative width/height (handles inverted rectangles)
    #[must_use]
    pub fn from_xywh(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::new(x, y, x + width, y + height)
    }

    #[must_use]
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    #[must_use]
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    #[must_use]
    pub fn center(&self) -> Point {
        Point::new(
            self.min_x + self.width() / 2.0,
            self.min_y + self.height() / 2.0,
        )
    }

    /// Expand the AABB by a given amount on all sides
    #[must_use]
    pub fn expand(&self, amount: f64) -> Self {
        Self::new(
            self.min_x - amount,
            self.min_y - amount,
            self.max_x + amount,
            self.max_y + amount,
        )
    }

    #[must_use]
    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }

    /// Compute the union of two AABBs
    #[must_use]
    pub const fn union(&self, other: &Self) -> Self {
        Self::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }
}
