//! Transform operations for subgraph nodes
//!
//! Operations for scaling and transforming groups of nodes.

use crate::document::{Node, NodeId};
use crate::geometry::Point;
use crate::transform::{apply_transform_to_node, TransformError, ValidTransform};

use super::types::CanvasState;
use super::types::PositiveScale;
use thiserror::Error;

pub type Subgraph = CanvasState;

pub const MAX_COORDINATE: f64 = 1_000_000.0;
pub const MIN_DIMENSION: f64 = 1.0;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GroupTransformError {
    #[error("Selection cannot be empty")]
    EmptySelection,
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Node locked: {0}")]
    NodeLocked(NodeId),
    #[error("Scale out of bounds")]
    OutOfBounds,
}

impl From<TransformError> for GroupTransformError {
    fn from(err: TransformError) -> Self {
        match err {
            TransformError::EmptySelection => Self::EmptySelection,
            TransformError::ItemNotFound(id) => Self::NodeNotFound(id),
            TransformError::NodeLocked(id) => Self::NodeLocked(id),
            TransformError::InvalidTransform => Self::OutOfBounds,
        }
    }
}

/// Scales a group of selected nodes relative to an anchor point.
///
/// # Errors
/// Returns `GroupTransformError` if selection is empty, a node is not found,
/// a node is locked, or if the resulting scale exceeds bounds.
pub fn scale_group(
    subgraph: &mut Subgraph,
    selection: &[NodeId],
    scale_factor: PositiveScale,
    anchor: Point,
) -> Result<(), GroupTransformError> {
    if selection.is_empty() {
        return Err(GroupTransformError::EmptySelection);
    }

    let s = scale_factor.value();
    let transform = ValidTransform::scale_around(s, s, anchor.x, anchor.y)?;

    let new_nodes = selection.iter().try_fold(
        subgraph.nodes.clone(),
        |acc, id| -> Result<im::HashMap<NodeId, Node>, GroupTransformError> {
            let node = acc
                .get(id)
                .ok_or_else(|| GroupTransformError::NodeNotFound(id.clone()))?;
            if node.lock_state.is_locked() {
                return Err(GroupTransformError::NodeLocked(id.clone()));
            }
            let updated = apply_transform_to_node(node, &transform)?;

            // Validate against constants
            if updated.x.0.abs() > MAX_COORDINATE
                || updated.y.0.abs() > MAX_COORDINATE
                || updated.width.0 > MAX_COORDINATE
                || updated.height.0 > MAX_COORDINATE
            {
                return Err(GroupTransformError::OutOfBounds);
            }

            Ok(acc.update(id.clone(), updated))
        },
    )?;

    subgraph.nodes = new_nodes;
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::document::node::{LockState, NodeKind};
    use crate::document::types::OrderedFloat;
    use im::HashMap;

    fn test_node() -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "test".to_string(),
            font_size: None,
            font_weight: None,
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(10.0),
            height: OrderedFloat(10.0),
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn test_subgraph() -> Subgraph {
        Subgraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    fn validate_scaled_dimensions(
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    ) -> Result<(), GroupTransformError> {
        if !x.is_finite() || !y.is_finite() || !w.is_finite() || !h.is_finite() {
            return Err(GroupTransformError::OutOfBounds);
        }
        if x.abs() > MAX_COORDINATE
            || y.abs() > MAX_COORDINATE
            || w > MAX_COORDINATE
            || h > MAX_COORDINATE
        {
            return Err(GroupTransformError::OutOfBounds);
        }
        Ok(())
    }

    fn compute_scaled_dimensions(
        node: &Node,
        scale: f64,
        anchor: Point,
    ) -> Result<(f64, f64, f64, f64), GroupTransformError> {
        let rel_x = node.x.0 - anchor.x;
        let rel_y = node.y.0 - anchor.y;

        let new_x = rel_x.mul_add(scale, anchor.x);
        let new_y = rel_y.mul_add(scale, anchor.y);
        let new_w = (node.width.0 * scale).max(MIN_DIMENSION);
        let new_h = (node.height.0 * scale).max(MIN_DIMENSION);

        validate_scaled_dimensions(new_x, new_y, new_w, new_h)?;
        Ok((new_x, new_y, new_w, new_h))
    }

    #[test]
    fn test_scale_group_empty_selection() {
        let mut subgraph = test_subgraph();
        let scale = PositiveScale::try_new(OrderedFloat(2.0)).unwrap();
        let result = scale_group(&mut subgraph, &[], scale, Point::origin());
        assert_eq!(result, Err(GroupTransformError::EmptySelection));
    }

    #[test]
    fn test_scale_group_node_not_found() {
        let mut subgraph = test_subgraph();
        let scale = PositiveScale::try_new(OrderedFloat(2.0)).unwrap();
        let id = NodeId::new("missing".to_string());
        let result = scale_group(&mut subgraph, &[id.clone()], scale, Point::origin());
        assert_eq!(result, Err(GroupTransformError::NodeNotFound(id)));
    }

    #[test]
    fn test_scale_group_locked_node() {
        let mut subgraph = test_subgraph();
        let scale = PositiveScale::try_new(OrderedFloat(2.0)).unwrap();
        let id = NodeId::new("n1".to_string());
        let mut node = test_node();
        node.lock_state = LockState::Locked;
        subgraph.nodes = subgraph.nodes.update(id.clone(), node);

        let result = scale_group(&mut subgraph, &[id.clone()], scale, Point::origin());
        assert_eq!(result, Err(GroupTransformError::NodeLocked(id)));
    }

    #[test]
    fn test_scale_group_success() {
        let mut subgraph = test_subgraph();
        let scale = PositiveScale::try_new(OrderedFloat(2.0)).unwrap();
        let id1 = NodeId::new("n1".to_string());
        let mut node1 = test_node();
        node1.x = OrderedFloat(10.0);
        node1.y = OrderedFloat(10.0);
        node1.width = OrderedFloat(20.0);
        node1.height = OrderedFloat(20.0);

        subgraph.nodes = subgraph.nodes.update(id1.clone(), node1);

        let anchor = Point::origin();
        let result = scale_group(&mut subgraph, &[id1.clone()], scale, anchor);
        assert_eq!(result, Ok(()));

        let updated = subgraph.nodes.get(&id1).unwrap();
        assert_eq!(updated.x, OrderedFloat(20.0));
        assert_eq!(updated.y, OrderedFloat(20.0));
        assert_eq!(updated.width, OrderedFloat(40.0));
        assert_eq!(updated.height, OrderedFloat(40.0));
    }

    #[test]
    fn test_scale_group_out_of_bounds() {
        let mut subgraph = test_subgraph();
        let scale = PositiveScale::try_new(OrderedFloat(100.0)).unwrap(); // large scale
        let id1 = NodeId::new("n1".to_string());
        let mut node1 = test_node();
        node1.x = OrderedFloat(MAX_COORDINATE - 10.0); // Will push it over

        subgraph.nodes = subgraph.nodes.update(id1.clone(), node1);

        let anchor = Point::origin();
        let result = scale_group(&mut subgraph, &[id1], scale, anchor);
        assert_eq!(result, Err(GroupTransformError::OutOfBounds));
    }

    #[test]
    fn test_validate_scaled_dimensions() {
        assert_eq!(validate_scaled_dimensions(0.0, 0.0, 10.0, 10.0), Ok(()));
        assert_eq!(
            validate_scaled_dimensions(f64::NAN, 0.0, 10.0, 10.0),
            Err(GroupTransformError::OutOfBounds)
        );
        assert_eq!(
            validate_scaled_dimensions(0.0, f64::INFINITY, 10.0, 10.0),
            Err(GroupTransformError::OutOfBounds)
        );
        assert_eq!(
            validate_scaled_dimensions(MAX_COORDINATE + 1.0, 0.0, 10.0, 10.0),
            Err(GroupTransformError::OutOfBounds)
        );
        assert_eq!(
            validate_scaled_dimensions(0.0, 0.0, MAX_COORDINATE + 1.0, 10.0),
            Err(GroupTransformError::OutOfBounds)
        );
    }

    #[test]
    fn test_compute_scaled_dimensions_clamps_to_min() {
        let node = test_node(); // width=10, height=10
        let scale = 0.01; // Results in 0.1, should clamp to MIN_DIMENSION (1.0)
        let anchor = Point::origin();

        let (x, y, w, h) = compute_scaled_dimensions(&node, scale, anchor).unwrap();
        assert_eq!(w, MIN_DIMENSION);
        assert_eq!(h, MIN_DIMENSION);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }
}
