use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CanvasError {
    #[error("Unparseable event from raw inputs")]
    UnparseableEvent,
    #[error("Invalid transition: {state} + {event}")]
    InvalidTransition { state: String, event: String },
    #[error("Coordinate out of bounds")]
    CoordinateOutOfBounds,
    #[error("Invalid selection bounds")]
    InvalidSelectionBounds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasPoint {
    pub x: f64,
    pub y: f64,
}

impl CanvasPoint {
    /// Creates a new `CanvasPoint`
    /// # Errors
    /// Returns `CanvasError::CoordinateOutOfBounds` if x or y is infinite or NaN
    pub const fn new(x: f64, y: f64) -> Result<Self, CanvasError> {
        if !x.is_finite() || !y.is_finite() {
            Err(CanvasError::CoordinateOutOfBounds)
        } else {
            Ok(Self { x, y })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasVector {
    pub dx: f64,
    pub dy: f64,
}

impl CanvasVector {
    /// Creates a new `CanvasVector`
    /// # Errors
    /// Returns `CanvasError::CoordinateOutOfBounds` if dx or dy is infinite or NaN
    pub const fn new(dx: f64, dy: f64) -> Result<Self, CanvasError> {
        if !dx.is_finite() || !dy.is_finite() {
            Err(CanvasError::CoordinateOutOfBounds)
        } else {
            Ok(Self { dx, dy })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Replace,
    Additive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionBounds {
    pub start: CanvasPoint,
    pub end: CanvasPoint,
}

impl SelectionBounds {
    /// Creates a new `SelectionBounds`
    /// # Errors
    /// Returns `CanvasError::InvalidSelectionBounds` if width or height is less than or equal to 0
    pub fn new(start: CanvasPoint, end: CanvasPoint) -> Result<Self, CanvasError> {
        let width = (end.x - start.x).abs();
        let height = (end.y - start.y).abs();
        if width <= 0.0 || height <= 0.0 {
            Err(CanvasError::InvalidSelectionBounds)
        } else {
            Ok(Self { start, end })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawEvent {
    pub event_type: String,
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub is_additive: bool,
}
