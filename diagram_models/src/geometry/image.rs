use super::aabb::AABB;

/// Represents an image with position and dimensions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Image {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Image {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> AABB {
        AABB::new(self.x, self.y, self.x + self.width, self.y + self.height)
    }
}
