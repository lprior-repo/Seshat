#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMode {
    Contain,
    Intersect,
}

pub const DRAG_THRESHOLD_PX: f64 = 3.0;
pub const TOUCH_HIT_RADIUS_MIN: f64 = 22.0;
