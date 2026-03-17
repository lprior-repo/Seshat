#![forbid(unsafe_code)]

pub mod interaction_reducer;
pub mod math;
pub mod perf;
pub mod selection_geometry;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenCoord(pub f64, pub f64);

impl ScreenCoord {
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasCoord(pub f64, pub f64);

impl CanvasCoord {
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
}
pub mod stubs;
