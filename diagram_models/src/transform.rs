//! Transform operations for diagram nodes.
//!
//! Provides pure logic for transforming (translating, scaling, rotating)
//! and aligning/distributing nodes.

use crate::document::{Node, NodeId, OrderedFloat};
use crate::geometry::{Coordinate, Radians, RectMetrics, ScaleFactor};
use im::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TransformError {
    #[error("Empty selection")]
    EmptySelection,
    #[error("Invalid transform")]
    InvalidTransform,
    #[error("Item not found: {0}")]
    ItemNotFound(NodeId),
    #[error("Node locked: {0}")]
    NodeLocked(NodeId),
}

/// Alignment axes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentAxis {
    Horizontal,
    Vertical,
}

/// Alignment modes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentMode {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone)]
pub struct ValidTransform {
    pub dx: Coordinate,
    pub dy: Coordinate,
    pub scale_x: ScaleFactor,
    pub scale_y: ScaleFactor,
    pub rotation: Radians,
}

impl ValidTransform {
    /// Creates a new `ValidTransform`.
    ///
    /// # Errors
    /// Returns `TransformError::InvalidTransform` if any parameters are non-finite or scales are zero.
    pub fn try_new(
        dx: f64,
        dy: f64,
        scale_x: f64,
        scale_y: f64,
        rotation: f64,
    ) -> Result<Self, TransformError> {
        if !dx.is_finite()
            || !dy.is_finite()
            || !scale_x.is_finite()
            || !scale_y.is_finite()
            || !rotation.is_finite()
            || scale_x == 0.0
            || scale_y == 0.0
        {
            return Err(TransformError::InvalidTransform);
        }
        Ok(Self {
            dx: Coordinate(dx),
            dy: Coordinate(dy),
            scale_x: ScaleFactor(scale_x),
            scale_y: ScaleFactor(scale_y),
            rotation: Radians(rotation),
        })
    }

    /// Creates a pure translation transform.
    ///
    /// # Errors
    /// Returns `TransformError::InvalidTransform` if dx or dy is not finite.
    pub fn translate(dx: f64, dy: f64) -> Result<Self, TransformError> {
        Self::try_new(dx, dy, 1.0, 1.0, 0.0)
    }

    /// Creates a pure scale transform around an anchor point.
    ///
    /// # Errors
    /// Returns `TransformError::InvalidTransform` if scale factors are zero or not finite.
    pub fn scale_around(
        scale_x: f64,
        scale_y: f64,
        anchor_x: f64,
        anchor_y: f64,
    ) -> Result<Self, TransformError> {
        let dx = anchor_x.mul_add(1.0 - scale_x, 0.0);
        let dy = anchor_y.mul_add(1.0 - scale_y, 0.0);
        Self::try_new(dx, dy, scale_x, scale_y, 0.0)
    }
}

/// Pure function: Applies a transform to a single node.
///
/// # Errors
/// Returns `TransformError::InvalidTransform` if the resulting coordinates are invalid.
pub fn apply_transform_to_node(
    node: &Node,
    transform: &ValidTransform,
) -> Result<Node, TransformError> {
    let mut updated = node.clone();
    updated.x = calculate_scaled_coord(node.x.0, transform.scale_x.0, transform.dx.0)?;
    updated.y = calculate_scaled_coord(node.y.0, transform.scale_y.0, transform.dy.0)?;
    updated.width = calculate_dimension(node.width.0, transform.scale_x.0)?;
    updated.height = calculate_dimension(node.height.0, transform.scale_y.0)?;

    if transform.rotation.0 != 0.0 {
        apply_rotation_to_metadata(&mut updated, transform.rotation);
    }

    Ok(updated)
}

fn calculate_scaled_coord(
    val: f64,
    scale: f64,
    delta: f64,
) -> Result<OrderedFloat, TransformError> {
    OrderedFloat::new(val.mul_add(scale, delta)).map_err(|_| TransformError::InvalidTransform)
}

fn calculate_dimension(val: f64, scale: f64) -> Result<OrderedFloat, TransformError> {
    OrderedFloat::new(val * scale).map_err(|_| TransformError::InvalidTransform)
}

fn apply_rotation_to_metadata(node: &mut Node, rotation: Radians) {
    let current_rot = node
        .metadata
        .get("rotation")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    node.metadata.insert(
        "rotation".to_string(),
        serde_json::json!(current_rot + rotation.0),
    );
}

/// Pure function: Calculates aligned positions for a group of nodes.
///
/// # Errors
/// Returns `TransformError::EmptySelection` if selection is too small.
/// Returns `TransformError::ItemNotFound` if a node is missing.
/// Returns `TransformError::InvalidTransform` if coordinate math fails.
pub fn calculate_alignment(
    nodes: &HashMap<NodeId, Node>,
    selection: &[NodeId],
    axis: AlignmentAxis,
    mode: AlignmentMode,
) -> Result<Vec<(NodeId, Node)>, TransformError> {
    if selection.len() < 2 {
        return Err(TransformError::EmptySelection);
    }

    let extents = collect_extents(nodes, selection)?;
    let (min_val, max_val) = calculate_range(&extents, axis);
    let target_pos = calculate_target_pos(min_val, max_val, mode);

    extents
        .into_iter()
        .map(|(id, node, metrics)| {
            let (_, size) = get_axis_metrics(metrics, axis);
            let new_pos = calculate_node_pos(target_pos, size, mode);
            let updated = apply_axis_pos(node, new_pos, axis)?;
            Ok((id.clone(), updated))
        })
        .collect()
}

fn collect_extents<'a>(
    nodes: &'a HashMap<NodeId, Node>,
    selection: &'a [NodeId],
) -> Result<Vec<(&'a NodeId, &'a Node, RectMetrics)>, TransformError> {
    selection
        .iter()
        .map(|id| {
            nodes
                .get(id)
                .map(|n| (id, n, RectMetrics::new(n.x.0, n.y.0, n.width.0, n.height.0)))
                .ok_or_else(|| TransformError::ItemNotFound(id.clone()))
        })
        .collect()
}

const fn get_axis_metrics(metrics: RectMetrics, axis: AlignmentAxis) -> (Coordinate, Coordinate) {
    match axis {
        AlignmentAxis::Horizontal => (metrics.x, metrics.width),
        AlignmentAxis::Vertical => (metrics.y, metrics.height),
    }
}

fn calculate_range(
    extents: &[(&NodeId, &Node, RectMetrics)],
    axis: AlignmentAxis,
) -> (Coordinate, Coordinate) {
    extents.iter().fold(
        (Coordinate::MAX, Coordinate::MIN),
        |(min, max), (_, _, m)| {
            let (pos, size) = get_axis_metrics(*m, axis);
            (min.min(pos), max.max(pos + size))
        },
    )
}

fn calculate_target_pos(min: Coordinate, max: Coordinate, mode: AlignmentMode) -> Coordinate {
    match mode {
        AlignmentMode::Start => min,
        AlignmentMode::Center => min + (max - min) / 2.0,
        AlignmentMode::End => max,
    }
}

fn calculate_node_pos(target: Coordinate, size: Coordinate, mode: AlignmentMode) -> Coordinate {
    match mode {
        AlignmentMode::Start => target,
        AlignmentMode::Center => target - (size / 2.0),
        AlignmentMode::End => target - size,
    }
}

fn apply_axis_pos(
    node: &Node,
    pos: Coordinate,
    axis: AlignmentAxis,
) -> Result<Node, TransformError> {
    let mut updated = node.clone();
    let val = OrderedFloat::new(pos.0).map_err(|_| TransformError::InvalidTransform)?;
    match axis {
        AlignmentAxis::Horizontal => updated.x = val,
        AlignmentAxis::Vertical => updated.y = val,
    }
    Ok(updated)
}

/// Pure function: Calculates distributed positions for a group of nodes.
///
/// # Errors
/// Returns `TransformError::EmptySelection` if selection is too small.
/// Returns `TransformError::ItemNotFound` if a node is missing.
/// Returns `TransformError::InvalidTransform` if coordinate math fails.
pub fn calculate_distribution(
    nodes: &HashMap<NodeId, Node>,
    selection: &[NodeId],
    axis: AlignmentAxis,
) -> Result<Vec<(NodeId, Node)>, TransformError> {
    if selection.len() < 3 {
        return Err(TransformError::EmptySelection);
    }

    let mut sorted = collect_extents(nodes, selection)?;
    sorted.sort_by(|a, b| {
        let (pos_a, _) = get_axis_metrics(a.2, axis);
        let (pos_b, _) = get_axis_metrics(b.2, axis);
        pos_a
            .partial_cmp(&pos_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let spacing = calculate_spacing(&sorted, axis);
    let first_pos = sorted
        .first()
        .map_or(Coordinate::ZERO, |e| get_axis_metrics(e.2, axis).0);

    Ok(apply_distribution_fold(&sorted, first_pos, spacing, axis))
}

fn apply_distribution_fold(
    sorted: &[(&NodeId, &Node, RectMetrics)],
    first_pos: Coordinate,
    spacing: Coordinate,
    axis: AlignmentAxis,
) -> Vec<(NodeId, Node)> {
    let (_, updates) = sorted.iter().fold(
        (first_pos, Vec::new()),
        |(curr, mut acc), (id, node, metrics)| {
            if let Ok(updated) = apply_axis_pos(node, curr, axis) {
                acc.push(((*id).clone(), updated));
            }
            let (_, size) = get_axis_metrics(*metrics, axis);
            (curr + size + spacing, acc)
        },
    );
    updates
}

fn calculate_spacing(sorted: &[(&NodeId, &Node, RectMetrics)], axis: AlignmentAxis) -> Coordinate {
    let Some(first_elem) = sorted.first() else {
        return Coordinate::ZERO;
    };
    let Some(last_elem) = sorted.last() else {
        return Coordinate::ZERO;
    };
    let (first_pos, _) = get_axis_metrics(first_elem.2, axis);
    let (last_pos, last_size) = get_axis_metrics(last_elem.2, axis);

    let total_span = (last_pos + last_size) - first_pos;
    let sum_sizes = Coordinate(
        sorted
            .iter()
            .map(|e| get_axis_metrics(e.2, axis).1 .0)
            .sum(),
    );
    #[allow(clippy::cast_precision_loss)]
    let count = sorted.len() as f64;
    if count <= 1.0 {
        return Coordinate::ZERO;
    }
    (total_span - sum_sizes) / (count - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transform() {
        let valid = ValidTransform::try_new(10.0, 20.0, 2.0, 2.0, 1.5);
        assert!(valid.is_ok());
    }
}
