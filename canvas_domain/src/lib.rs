#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod interaction_reducer;
pub mod math;
pub mod perf;
pub mod selection_geometry;

// Re-export Point from diagram_models
pub use diagram_models::geometry::Point;

/// Screen coordinates - tuple wrapper for ergonomic access
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenCoord(pub f64, pub f64);

impl ScreenCoord {
    #[must_use]
    pub const fn x(&self) -> f64 {
        self.0
    }
    #[must_use]
    pub const fn y(&self) -> f64 {
        self.1
    }
}

impl From<Point> for ScreenCoord {
    fn from(p: Point) -> Self {
        Self(p.x, p.y)
    }
}

impl From<ScreenCoord> for Point {
    fn from(sc: ScreenCoord) -> Self {
        Self::new(sc.0, sc.1)
    }
}

/// Canvas coordinates - tuple wrapper for ergonomic access
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasCoord(pub f64, pub f64);

impl CanvasCoord {
    #[must_use]
    pub const fn x(&self) -> f64 {
        self.0
    }
    #[must_use]
    pub const fn y(&self) -> f64 {
        self.1
    }
}

impl From<Point> for CanvasCoord {
    fn from(p: Point) -> Self {
        Self(p.x, p.y)
    }
}

impl From<CanvasCoord> for Point {
    fn from(cc: CanvasCoord) -> Self {
        Self::new(cc.0, cc.1)
    }
}

impl From<(f64, f64)> for ScreenCoord {
    fn from((x, y): (f64, f64)) -> Self {
        Self(x, y)
    }
}

impl From<(f64, f64)> for CanvasCoord {
    fn from((x, y): (f64, f64)) -> Self {
        Self(x, y)
    }
}

impl From<ScreenCoord> for (f64, f64) {
    fn from(sc: ScreenCoord) -> Self {
        (sc.0, sc.1)
    }
}

impl From<CanvasCoord> for (f64, f64) {
    fn from(cc: CanvasCoord) -> Self {
        (cc.0, cc.1)
    }
}

pub mod stubs;
