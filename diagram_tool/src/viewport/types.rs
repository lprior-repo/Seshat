use diagram_models::geometry::Point;

/// Maximum pan distance from origin in world units
pub const MAX_PAN_DISTANCE: f64 = 10000.0;

/// Maximum safe coordinate magnitude before overflow risk
/// Coordinates beyond this magnitude may cause float overflow in calculations
pub const MAX_SAFE_COORDINATE: f64 = 1e15;

/// Errors that can occur during viewport operations
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum ViewportError {
    #[error("Invalid padding: padding must be non-negative, got {0}")]
    InvalidPadding(f64),
    #[error("Invalid content bounds: width or height must be positive")]
    InvalidContentBounds,
    #[error("Content bounds overflow: coordinates too large for safe calculation")]
    CoordinateOverflow,
    #[error("Invalid viewport dimensions: width and height must be positive")]
    InvalidViewport,
}

/// A point in world coordinates (alias for Point)
pub type WorldPoint = Point;

/// A point in screen coordinates (alias for Point)
pub type ScreenPoint = Point;

/// Result of fit-to-content calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitTransform {
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}
