pub mod alignment;
pub mod grid;
pub mod mod_types;

pub use alignment::*;
pub use grid::*;
pub use mod_types::*;

use crate::geometry::primitives::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleState {
    On,
    Off,
}

#[must_use]
pub fn should_snap(distance: f64, threshold: f64) -> bool {
    if !distance.is_finite() || !threshold.is_finite() || threshold < 0.0 {
        return false;
    }
    distance <= threshold
}

#[must_use]
pub fn drag_with_snap(
    _start: Point,
    current: Point,
    grid_size: f64,
    snap_mode: SnapMode,
) -> (Point, Point) {
    if snap_mode == SnapMode::Disabled || grid_size <= 0.0 {
        return (current, current);
    }

    let snapped = grid::snap_to_grid(current, grid_size);
    (snapped, snapped)
}

#[must_use]
pub fn drag_multi_with_snap(
    nodes: &[SnapNode],
    drag_delta: Point,
    grid_size: f64,
    snap_mode: SnapMode,
) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    if snap_mode == SnapMode::Disabled || grid_size <= 0.0 {
        return nodes
            .iter()
            .map(|n| Point::new(n.x + drag_delta.x, n.y + drag_delta.y))
            .collect();
    }

    let primary_snapped = grid::snap_to_grid(
        Point::new(nodes[0].x + drag_delta.x, nodes[0].y + drag_delta.y),
        grid_size,
    );
    let offset = Point::new(
        primary_snapped.x - (nodes[0].x + drag_delta.x),
        primary_snapped.y - (nodes[0].y + drag_delta.y),
    );

    nodes
        .iter()
        .map(|n| Point::new(n.x + drag_delta.x + offset.x, n.y + drag_delta.y + offset.y))
        .collect()
}

#[must_use]
pub fn snap_multi_nodes(nodes: &[SnapNode], grid_size: f64) -> Vec<Point> {
    if nodes.is_empty() || grid_size <= 0.0 {
        return nodes.iter().map(|n| Point::new(n.x, n.y)).collect();
    }

    nodes
        .iter()
        .map(|n| grid::snap_to_grid(Point::new(n.x, n.y), grid_size))
        .collect()
}

#[must_use]
pub fn snap_multi_to_primary(
    nodes: &[SnapNode],
    primary_index: usize,
    grid_size: f64,
) -> Vec<Point> {
    if nodes.is_empty() || grid_size <= 0.0 {
        return nodes.iter().map(|n| Point::new(n.x, n.y)).collect();
    }

    let Some(primary) = nodes.get(primary_index) else {
        return nodes.iter().map(|n| Point::new(n.x, n.y)).collect();
    };

    let primary_snapped = grid::snap_to_grid(Point::new(primary.x, primary.y), grid_size);
    let snap_offset = Point::new(primary_snapped.x - primary.x, primary_snapped.y - primary.y);

    nodes
        .iter()
        .map(|n| Point::new(n.x + snap_offset.x, n.y + snap_offset.y))
        .collect()
}

#[must_use]
pub const fn toggle_snap(state: ToggleState) -> ToggleState {
    match state {
        ToggleState::On => ToggleState::Off,
        ToggleState::Off => ToggleState::On,
    }
}

#[must_use]
pub const fn is_snap_enabled(state: SnapState) -> bool {
    state.is_enabled()
}

#[must_use]
pub fn toggle_during_drag(
    position: Point,
    snap_mode: SnapMode,
    grid_size: f64,
) -> (Point, SnapMode) {
    if snap_mode == SnapMode::Enabled {
        (position, SnapMode::Disabled)
    } else {
        let snapped = grid::snap_to_grid(position, grid_size);
        (snapped, SnapMode::Enabled)
    }
}

#[cfg(test)]
mod tests;
