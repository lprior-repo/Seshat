use super::aabb::AABB;
use super::rectangle::Rectangle;

/// Represents a shape with stroke
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StrokedShape<T> {
    pub shape: T,
    pub stroke_width: f64,
}

impl<T> StrokedShape<T> {
    #[must_use]
    pub const fn new(shape: T, stroke_width: f64) -> Self {
        Self {
            shape,
            stroke_width,
        }
    }
}

impl StrokedShape<Rectangle> {
    #[must_use]
    pub fn bounds_with_stroke(&self) -> AABB {
        let shape_aabb = self.shape.aabb();
        shape_aabb.expand(self.stroke_width / 2.0)
    }
}
