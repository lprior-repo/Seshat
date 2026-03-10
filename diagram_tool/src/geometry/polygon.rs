use crate::geometry::primitives::Point;

/// Placeholder for polygon types per extraction plan
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub points: Vec<Point>,
}

impl Polygon {
    #[must_use]
    pub fn new(points: Vec<Point>) -> Self {
        Self { points }
    }
}
