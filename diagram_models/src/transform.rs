//! Transform operations for diagram nodes.
//!
//! Provides pure logic for transforming (translating, scaling, rotating)
//! and aligning/distributing nodes.

use crate::document::{Node, NodeId, OrderedFloat};
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
    pub dx: f64,
    pub dy: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation: f64,
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
            dx,
            dy,
            scale_x,
            scale_y,
            rotation,
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
    updated.x = OrderedFloat::new(node.x.0.mul_add(transform.scale_x, transform.dx))
        .map_err(|_| TransformError::InvalidTransform)?;
    updated.y = OrderedFloat::new(node.y.0.mul_add(transform.scale_y, transform.dy))
        .map_err(|_| TransformError::InvalidTransform)?;
    updated.width = OrderedFloat::new(node.width.0 * transform.scale_x)
        .map_err(|_| TransformError::InvalidTransform)?;
    updated.height = OrderedFloat::new(node.height.0 * transform.scale_y)
        .map_err(|_| TransformError::InvalidTransform)?;

    if transform.rotation != 0.0 {
        apply_rotation_to_metadata(&mut updated, transform.rotation);
    }

    Ok(updated)
}

fn apply_rotation_to_metadata(node: &mut Node, rotation: f64) {
    let current_rot = node
        .metadata
        .get("rotation")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    node.metadata.insert(
        "rotation".to_string(),
        serde_json::json!(current_rot + rotation),
    );
}

/// Pure function: Calculates aligned positions for a group of nodes.
///
/// # Errors
/// Returns `TransformError::EmptySelection` if selection is too small.
/// Returns `TransformError::ItemNotFound` if a node is missing.
pub fn calculate_alignment(
    nodes: &HashMap<NodeId, Node>,
    selection: &[NodeId],
    axis: AlignmentAxis,
    mode: AlignmentMode,
) -> Result<Vec<(NodeId, Node)>, TransformError> {
    if selection.len() < 2 {
        return Err(TransformError::EmptySelection);
    }

    let extents = collect_extents(nodes, selection, axis)?;
    let (min_val, max_val) = calculate_range(&extents);
    let target_pos = calculate_target_pos(min_val, max_val, mode);

    extents
        .into_iter()
        .map(|(id, node, _, size)| {
            let new_pos = calculate_node_pos(target_pos, size, mode);
            let updated = apply_axis_pos(node, new_pos, axis)?;
            Ok((id.clone(), updated))
        })
        .collect()
}

fn collect_extents<'a>(
    nodes: &'a HashMap<NodeId, Node>,
    selection: &'a [NodeId],
    axis: AlignmentAxis,
) -> Result<Vec<(&'a NodeId, &'a Node, f64, f64)>, TransformError> {
    selection
        .iter()
        .map(|id| {
            nodes
                .get(id)
                .map(|n| {
                    let (pos, size) = get_axis_metrics(n, axis);
                    (id, n, pos, size)
                })
                .ok_or_else(|| TransformError::ItemNotFound(id.clone()))
        })
        .collect()
}

const fn get_axis_metrics(node: &Node, axis: AlignmentAxis) -> (f64, f64) {
    match axis {
        AlignmentAxis::Horizontal => (node.x.0, node.width.0),
        AlignmentAxis::Vertical => (node.y.0, node.height.0),
    }
}

fn calculate_range(extents: &[(&NodeId, &Node, f64, f64)]) -> (f64, f64) {
    extents
        .iter()
        .fold((f64::MAX, f64::MIN), |(min, max), (_, _, pos, size)| {
            (min.min(*pos), max.max(*pos + *size))
        })
}

fn calculate_target_pos(min: f64, max: f64, mode: AlignmentMode) -> f64 {
    match mode {
        AlignmentMode::Start => min,
        AlignmentMode::Center => min + (max - min) / 2.0,
        AlignmentMode::End => max,
    }
}

fn calculate_node_pos(target: f64, size: f64, mode: AlignmentMode) -> f64 {
    match mode {
        AlignmentMode::Start => target,
        AlignmentMode::Center => target - (size / 2.0),
        AlignmentMode::End => target - size,
    }
}

fn apply_axis_pos(node: &Node, pos: f64, axis: AlignmentAxis) -> Result<Node, TransformError> {
    let mut updated = node.clone();
    let val = OrderedFloat::new(pos).map_err(|_| TransformError::InvalidTransform)?;
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
pub fn calculate_distribution(
    nodes: &HashMap<NodeId, Node>,
    selection: &[NodeId],
    axis: AlignmentAxis,
) -> Result<Vec<(NodeId, Node)>, TransformError> {
    if selection.len() < 3 {
        return Err(TransformError::EmptySelection);
    }

    let mut sorted = collect_extents(nodes, selection, axis)?;
    sorted.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let spacing = calculate_spacing(&sorted);
    let first_pos = sorted[0].2;

    let (_, updates) = sorted.into_iter().fold(
        (first_pos, Vec::new()),
        |(curr, mut acc), (id, node, _, size)| {
            if let Ok(updated) = apply_axis_pos(node, curr, axis) {
                acc.push((id.clone(), updated));
            }
            (curr + size + spacing, acc)
        },
    );

    Ok(updates)
}

fn calculate_spacing(sorted: &[(&NodeId, &Node, f64, f64)]) -> f64 {
    let Some(first_elem) = sorted.first() else {
        return 0.0;
    };
    let Some(last_elem) = sorted.last() else {
        return 0.0;
    };
    let total_span = (last_elem.2 + last_elem.3) - first_elem.2;
    let sum_sizes: f64 = sorted.iter().map(|e| e.3).sum();
    #[allow(clippy::cast_precision_loss)]
    let count = sorted.len() as f64;
    if count <= 1.0 {
        return 0.0;
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
