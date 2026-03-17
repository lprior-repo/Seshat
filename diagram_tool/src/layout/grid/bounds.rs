//! Grid bounds and size definitions.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellSize(f64);

#[derive(Debug, thiserror::Error)]
pub enum GridError {
    #[error("cell_size must be positive and finite, got {0}")]
    InvalidCellSize(f64),
}

impl CellSize {
    pub fn new(val: f64) -> Result<Self, GridError> {
        if val.is_finite() && val > 0.0 {
            Ok(Self(val))
        } else {
            Err(GridError::InvalidCellSize(val))
        }
    }
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}
