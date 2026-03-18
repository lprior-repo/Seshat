use super::aabb::AABB;
use super::point::Point;

/// Represents a rectangle with position, dimensions, and optional rotation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64, // rotation in radians
}

impl Rectangle {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            rotation: 0.0,
        }
    }

    #[must_use]
    pub const fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    #[must_use]
    pub fn aabb(&self) -> AABB {
        if self.rotation == 0.0 {
            AABB::new(self.x, self.y, self.x + self.width, self.y + self.height)
        } else {
            let corners = self.corners();
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for corner in corners {
                min_x = min_x.min(corner.x);
                min_y = min_y.min(corner.y);
                max_x = max_x.max(corner.x);
                max_y = max_y.max(corner.y);
            }

            AABB::new(min_x, min_y, max_x, max_y)
        }
    }

    #[must_use]
    pub fn corners(&self) -> [Point; 4] {
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;

        let hw = self.width / 2.0;
        let hh = self.height / 2.0;

        let local_corners = [
            Point::new(-hw, -hh),
            Point::new(hw, -hh),
            Point::new(hw, hh),
            Point::new(-hw, hh),
        ];

        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        local_corners.map(|p| {
            Point::new(
                p.x.mul_add(cos, -(p.y * sin)) + cx,
                p.x.mul_add(sin, p.y * cos) + cy,
            )
        })
    }
}
