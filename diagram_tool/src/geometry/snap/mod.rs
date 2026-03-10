pub mod alignment;
pub mod grid;
pub mod mod_types;

pub use alignment::*;
pub use grid::*;
pub use mod_types::*;

use crate::geometry::primitives::Point;

#[must_use]
pub fn should_snap(distance: f64, threshold: f64) -> bool {
    if !distance.is_finite() || !threshold.is_finite() || threshold < 0.0 {
        return false;
    }
    distance <= threshold
}

pub fn drag_with_snap(
    _start: Point,
    current: Point,
    grid_size: f64,
    snap_enabled: bool,
) -> (Point, Point) {
    if !snap_enabled || grid_size <= 0.0 {
        return (current, current);
    }

    let snapped = grid::snap_to_grid(current, grid_size);
    (snapped, snapped)
}

pub fn drag_multi_with_snap(
    nodes: &[SnapNode],
    drag_delta: Point,
    grid_size: f64,
    snap_enabled: bool,
) -> Vec<Point> {
    if nodes.is_empty() {
        return Vec::new();
    }

    if !snap_enabled || grid_size <= 0.0 {
        return nodes
            .iter()
            .map(|n| Point::new(n.x + drag_delta.x, n.y + drag_delta.y))
            .collect();
    }

    let primary = &nodes[0];
    let primary_new = Point::new(primary.x + drag_delta.x, primary.y + drag_delta.y);
    let primary_snapped = grid::snap_to_grid(primary_new, grid_size);

    let snap_offset = Point::new(
        primary_snapped.x - primary_new.x,
        primary_snapped.y - primary_new.y,
    );

    nodes
        .iter()
        .map(|n| {
            Point::new(
                n.x + drag_delta.x + snap_offset.x,
                n.y + drag_delta.y + snap_offset.y,
            )
        })
        .collect()
}

pub fn snap_multi_nodes(nodes: &[SnapNode], grid_size: f64) -> Vec<Point> {
    if nodes.is_empty() || grid_size <= 0.0 {
        return nodes.iter().map(|n| Point::new(n.x, n.y)).collect();
    }

    nodes
        .iter()
        .map(|n| grid::snap_to_grid(Point::new(n.x, n.y), grid_size))
        .collect()
}

pub fn snap_multi_to_primary(
    nodes: &[SnapNode],
    primary_index: usize,
    grid_size: f64,
) -> Vec<Point> {
    if nodes.is_empty() || grid_size <= 0.0 {
        return nodes.iter().map(|n| Point::new(n.x, n.y)).collect();
    }

    let primary = match nodes.get(primary_index) {
        Some(p) => p,
        None => return nodes.iter().map(|n| Point::new(n.x, n.y)).collect(),
    };

    let primary_snapped = grid::snap_to_grid(Point::new(primary.x, primary.y), grid_size);
    let snap_offset = Point::new(primary_snapped.x - primary.x, primary_snapped.y - primary.y);

    nodes
        .iter()
        .map(|n| Point::new(n.x + snap_offset.x, n.y + snap_offset.y))
        .collect()
}

#[must_use]
pub fn toggle_snap(state: bool) -> bool {
    !state
}

#[must_use]
pub const fn is_snap_enabled(state: SnapState) -> bool {
    state.enabled
}

pub fn toggle_during_drag(
    position: Point,
    snap_was_enabled: bool,
    grid_size: f64,
) -> (Point, bool) {
    if snap_was_enabled {
        (position, false)
    } else {
        let snapped = grid::snap_to_grid(position, grid_size);
        (snapped, true)
    }
}

#[cfg(test)]
mod tests;
