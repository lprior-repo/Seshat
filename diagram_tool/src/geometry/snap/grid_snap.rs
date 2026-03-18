// Re-export validated GridSize from diagram_models
pub use diagram_models::document::{GridError, GridSize};

use super::mod_types::SnapMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GridSnapError {
    #[error("Coordinate is not finite")]
    NotFinite,
}

/// Wrapper function to match existing API.
/// Creates a new `GridSize`, returning error if out of range or not finite.
#[must_use]
pub fn try_grid_size(raw_step: f64) -> Result<GridSize, GridError> {
    GridSize::new(raw_step)
}

#[must_use]
pub fn snap_node_coordinate(
    raw_value: f64,
    mode: SnapMode,
    grid: GridSize,
) -> Result<f64, GridSnapError> {
    if !raw_value.is_finite() {
        return Err(GridSnapError::NotFinite);
    }
    match mode {
        SnapMode::Disabled => Ok(raw_value),
        SnapMode::Enabled => {
            let grid_val = grid.inner();
            let snapped = (raw_value / grid_val).round() * grid_val;
            Ok(snapped)
        }
    }
}

#[must_use]
pub fn snap_node_coordinates(
    raw_point: (f64, f64),
    mode: SnapMode,
    grid: GridSize,
) -> Result<(f64, f64), GridSnapError> {
    if !raw_point.0.is_finite() || !raw_point.1.is_finite() {
        return Err(GridSnapError::NotFinite);
    }
    let x = snap_node_coordinate(raw_point.0, mode, grid)?;
    let y = snap_node_coordinate(raw_point.1, mode, grid)?;
    Ok((x, y))
}

#[must_use]
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: GridSize) -> f64 {
    if !snap_to_grid {
        return value;
    }
    let step = grid_size.inner().max(1.0);
    (value / step).round() * step
}

#[must_use]
pub fn snap_point(point: (f64, f64), snap_to_grid: bool, grid_size: GridSize) -> (f64, f64) {
    (
        snap_value(point.0, snap_to_grid, grid_size),
        snap_value(point.1, snap_to_grid, grid_size),
    )
}

#[cfg(test)]
#[path = "grid_snap_tests.rs"]
mod grid_snap_tests;
