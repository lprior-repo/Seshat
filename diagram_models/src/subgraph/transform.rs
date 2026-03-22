//! Transform operations for subgraph nodes
//!
//! Operations for scaling and transforming groups of nodes.

use crate::document::{Node, NodeId, OrderedFloat};
use crate::geometry::Point;

use super::types::CanvasState;
use super::types::PositiveScale;
use thiserror::Error;

pub type Subgraph = CanvasState;

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

const MIN_DIMENSION: f64 = 1.0;
const MAX_COORDINATE: f64 = 1_000_000.0;

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

    let updates = calculate_scaled_nodes(subgraph, selection, scale_factor, anchor)?;
    apply_node_updates(subgraph, updates)
}

fn calculate_scaled_nodes(
    subgraph: &Subgraph,
    selection: &[NodeId],
    scale_factor: PositiveScale,
    anchor: Point,
) -> Result<Vec<(NodeId, Node)>, GroupTransformError> {
    let scale = scale_factor.value();

    selection
        .iter()
        .map(|id| transform_single_node(subgraph, id, scale, anchor))
        .collect()
}

fn transform_single_node(
    subgraph: &Subgraph,
    id: &NodeId,
    scale: f64,
    anchor: Point,
) -> Result<(NodeId, Node), GroupTransformError> {
    let node = subgraph
        .nodes
        .get(id)
        .ok_or_else(|| GroupTransformError::NodeNotFound(id.clone()))?;

    if node.lock_state.is_locked() {
        return Err(GroupTransformError::NodeLocked(id.clone()));
    }

    let (new_x, new_y, new_w, new_h) = compute_scaled_dimensions(node, scale, anchor)?;
    let updated_node = create_scaled_node(node, new_x, new_y, new_w, new_h)?;

    Ok((id.clone(), updated_node))
}

fn compute_scaled_dimensions(
    node: &Node,
    scale: f64,
    anchor: Point,
) -> Result<(f64, f64, f64, f64), GroupTransformError> {
    let new_x = (node.x.0 - anchor.x).mul_add(scale, anchor.x);
    let new_y = (node.y.0 - anchor.y).mul_add(scale, anchor.y);
    let new_w = (node.width.0 * scale).max(MIN_DIMENSION);
    let new_h = (node.height.0 * scale).max(MIN_DIMENSION);

    validate_scaled_dimensions(new_x, new_y, new_w, new_h)?;

    Ok((new_x, new_y, new_w, new_h))
}

fn validate_scaled_dimensions(
    new_x: f64,
    new_y: f64,
    new_w: f64,
    new_h: f64,
) -> Result<(), GroupTransformError> {
    if !new_x.is_finite() || !new_y.is_finite() || !new_w.is_finite() || !new_h.is_finite() {
        return Err(GroupTransformError::OutOfBounds);
    }

    if new_x.abs() > MAX_COORDINATE
        || new_y.abs() > MAX_COORDINATE
        || new_w > MAX_COORDINATE
        || new_h > MAX_COORDINATE
    {
        return Err(GroupTransformError::OutOfBounds);
    }
    Ok(())
}

fn create_scaled_node(
    node: &Node,
    new_x: f64,
    new_y: f64,
    new_w: f64,
    new_h: f64,
) -> Result<Node, GroupTransformError> {
    Ok(Node {
        x: OrderedFloat::new(new_x).map_err(|_| GroupTransformError::OutOfBounds)?,
        y: OrderedFloat::new(new_y).map_err(|_| GroupTransformError::OutOfBounds)?,
        width: OrderedFloat::new(new_w).map_err(|_| GroupTransformError::OutOfBounds)?,
        height: OrderedFloat::new(new_h).map_err(|_| GroupTransformError::OutOfBounds)?,
        ..node.clone()
    })
}

#[allow(clippy::unnecessary_wraps)]
fn apply_node_updates(
    subgraph: &mut Subgraph,
    updates: Vec<(NodeId, Node)>,
) -> Result<(), GroupTransformError> {
    subgraph.nodes = updates
        .into_iter()
        .fold(subgraph.nodes.clone(), |nodes, (id, node)| {
            nodes.update(id, node)
        });

    Ok(())
}

#[cfg(test)]
mod tests {
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
