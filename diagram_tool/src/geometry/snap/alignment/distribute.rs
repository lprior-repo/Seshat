use crate::geometry::primitives::Point;
use crate::geometry::snap::mod_types::{SnapError, SnapNode};

fn distribute_axis(
    nodes: &[SnapNode],
    coord_fn: impl Fn(&SnapNode) -> f64,
    point_maker: impl Fn(f64, &SnapNode) -> Point,
) -> Result<Vec<Point>, SnapError> {
    if nodes.len() < 3 {
        return Err(SnapError::InsufficientNodesForDistribution(nodes.len()));
    }
    let mut sorted: Vec<usize> = (0..nodes.len()).collect();
    sorted.sort_by(|&a, &b| {
        coord_fn(&nodes[a])
            .partial_cmp(&coord_fn(&nodes[b]))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let first_val = coord_fn(&nodes[sorted[0]]);
    let last_val = coord_fn(&nodes[sorted[sorted.len() - 1]]);
    let spacing = (last_val - first_val) / (sorted.len() - 1) as f64;

    let mut result: Vec<(usize, Point)> = sorted
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let val = (i as f64).mul_add(spacing, first_val);
            (idx, point_maker(val, &nodes[idx]))
        })
        .collect();

    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, p)| p).collect())
}

/// Distribute nodes horizontally evenly.
///
/// # Errors
/// Returns an error if less than 3 nodes are selected.
pub fn distribute_horizontally(nodes: &[SnapNode]) -> Result<Vec<Point>, SnapError> {
    distribute_axis(nodes, |n| n.x, |new_x, n| Point::new(new_x, n.y))
}

/// Distribute nodes vertically evenly.
///
/// # Errors
/// Returns an error if less than 3 nodes are selected.
pub fn distribute_vertically(nodes: &[SnapNode]) -> Result<Vec<Point>, SnapError> {
    distribute_axis(nodes, |n| n.y, |new_y, n| Point::new(n.x, new_y))
}
