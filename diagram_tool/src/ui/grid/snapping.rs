use super::size::{GridError, GridSize};
use thiserror::Error;

// =============================================================================
// Contract Types: SnapMode and GridSnapError
// =============================================================================

/// Explicit snap state for contract-compliant API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapMode {
    /// Snapping is enabled
    Enabled,
    /// Snapping is disabled - free movement
    Disabled,
}

impl SnapMode {
    /// Converts a bool to `SnapMode` for backward compatibility with existing callers.
    #[must_use]
    pub const fn from_bool(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }

    /// Returns true if snapping is enabled.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

/// Errors for contract-compliant grid snapping API.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GridSnapError {
    /// Raw x coordinate is non-finite (NaN or Infinity)
    #[error("x coordinate must be finite, got non-finite value")]
    NonFiniteX,

    /// Raw y coordinate is non-finite (NaN or Infinity)
    #[error("y coordinate must be finite, got non-finite value")]
    NonFiniteY,

    /// Grid size is invalid
    #[error("invalid grid size: {0}")]
    InvalidGridSize(#[from] GridError),

    /// Contract violation detected
    #[error("contract violation in {clause}: {details}")]
    ContractViolation {
        /// The contract clause that was violated
        clause: &'static str,
        /// Details about the violation
        details: String,
    },
}

// =============================================================================
// Contract Functions: snap_node_coordinate, snap_node_coordinates, try_grid_size
// =============================================================================

/// Validates and creates a `GridSize` from raw f64, returning contract error on failure.
///
/// # Errors
/// Returns `GridSnapError::InvalidGridSize` if the grid size is invalid.
pub fn try_grid_size(raw_step: f64) -> Result<GridSize, GridSnapError> {
    GridSize::new(raw_step).map_err(GridSnapError::from)
}

/// Snaps a single coordinate value to the grid if snapping is enabled.
///
/// # Errors
/// Returns `GridSnapError::NonFiniteX` if the raw value is NaN or Infinity.
pub fn snap_node_coordinate(
    raw_value: f64,
    mode: SnapMode,
    grid: GridSize,
) -> Result<f64, GridSnapError> {
    if !raw_value.is_finite() {
        return Err(GridSnapError::NonFiniteX);
    }
    match mode {
        SnapMode::Disabled => Ok(raw_value),
        SnapMode::Enabled => Ok(snap_value(raw_value, true, grid)),
    }
}

/// Snaps a point (x, y) to the grid if snapping is enabled.
///
/// # Errors
/// Returns `GridSnapError::NonFiniteX` or `GridSnapError::NonFiniteY` if coordinates are non-finite.
pub fn snap_node_coordinates(
    raw_point: (f64, f64),
    mode: SnapMode,
    grid: GridSize,
) -> Result<(f64, f64), GridSnapError> {
    let (raw_x, raw_y) = raw_point;
    if !raw_x.is_finite() {
        return Err(GridSnapError::NonFiniteX);
    }
    if !raw_y.is_finite() {
        return Err(GridSnapError::NonFiniteY);
    }
    match mode {
        SnapMode::Disabled => Ok((raw_x, raw_y)),
        SnapMode::Enabled => Ok(snap_point((raw_x, raw_y), true, grid)),
    }
}

/// Snaps a single value to the grid if snapping is enabled.
///
/// # Guarantees
/// - If `snap_to_grid == false`, returns `value` unchanged
/// - If `grid_size` inner value is <= 0 or non-finite, treats `grid_size` as 1.0
/// - Result is always finite if input is finite
/// - NaN input returns NaN
#[must_use]
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: GridSize) -> f64 {
    if !snap_to_grid {
        return value;
    }

    let step = grid_size.inner().max(1.0);
    (value / step).round() * step
}

/// Snaps a point (x, y) to the grid if snapping is enabled.
///
/// # Guarantees
/// - Applies `snap_value` independently to each coordinate
/// - See [`snap_value`] for additional guarantees
#[must_use]
pub fn snap_point(point: (f64, f64), snap_to_grid: bool, grid_size: GridSize) -> (f64, f64) {
    (
        snap_value(point.0, snap_to_grid, grid_size),
        snap_value(point.1, snap_to_grid, grid_size),
    )
}
