use crate::ui::grid::{snap_point as grid_snap_point, snap_value as grid_snap_value, GridSize};

#[must_use]
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use crate::ui::grid::snap_value instead")]
pub fn snap_value(value: f64, snap_to_grid: bool, grid_size: f64) -> f64 {
    let clamped = grid_size.clamp(GridSize::MIN, GridSize::MAX);
    let grid = GridSize::new(clamped).unwrap_or_default();
    grid_snap_value(value, snap_to_grid, grid)
}

#[must_use]
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use crate::ui::grid::snap_point instead")]
pub fn snap_point(point: (f64, f64), snap_to_grid: bool, grid_size: f64) -> (f64, f64) {
    let clamped = grid_size.clamp(GridSize::MIN, GridSize::MAX);
    let grid = GridSize::new(clamped).unwrap_or_default();
    grid_snap_point(point, snap_to_grid, grid)
}
