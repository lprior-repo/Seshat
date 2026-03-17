// Re-export validated GridSize from diagram_models
pub use diagram_models::document::{GridError, GridSize, NonFiniteKind};

/// Validates and creates a `GridSize` from a raw f64.
///
/// # Errors
/// - `GridError::OutOfRange` if value < 10.0 or value > 100.0
/// - `GridError::NotFinite` if value is NaN or Infinity
#[allow(dead_code)]
pub fn validated_grid_size(value: f64) -> Result<GridSize, GridError> {
    GridSize::new(value)
}
