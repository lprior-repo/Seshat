//! Logic for recomputing container bounds based on children.

use crate::document::{Node, NodeId, NodeKind, OrderedFloat};
use crate::geometry::{Coordinate, RectMetrics};
use crate::subgraph::LayoutConstants;
use im::HashMap;
use itertools::Itertools;
use smallvec::SmallVec;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LayoutError {
    #[error("Coordinate math failure")]
    MathFailure,
}

/// Recomputes the bounds of subgraph containers that contain the moved nodes.
#[must_use]
pub fn recompute_affected_container_bounds(
    nodes: HashMap<NodeId, Node>,
    moved_node_ids: &[NodeId],
) -> HashMap<NodeId, Node> {
    // Need to collect to end the borrow of nodes before folding
    #[allow(clippy::needless_collect)]
    let containers = moved_node_ids
        .iter()
        .filter_map(|id| nodes.get(id))
        .filter_map(|node| node.parent.as_ref())
        .filter(|pid| {
            nodes
                .get(*pid)
                .is_some_and(|p| p.kind == NodeKind::Subgraph)
        })
        .unique()
        .cloned()
        .collect::<Vec<_>>();

    containers.into_iter().fold(nodes, update_container_bounds)
}

fn update_container_bounds(mut nodes: HashMap<NodeId, Node>, cid: NodeId) -> HashMap<NodeId, Node> {
    let metrics = collect_child_metrics(&nodes, &cid);
    if let Some(bounds) = calculate_subgraph_bounds(&metrics) {
        if let Some(container) = nodes.get(&cid) {
            if let Ok(updated) = apply_bounds_to_container(container, bounds) {
                nodes = nodes.update(cid, updated);
            }
        }
    }
    nodes
}

fn collect_child_metrics(
    nodes: &HashMap<NodeId, Node>,
    parent_id: &NodeId,
) -> SmallVec<[RectMetrics; 8]> {
    nodes
        .iter()
        .filter(|(_, n)| n.parent.as_ref() == Some(parent_id))
        .map(|(_, n)| RectMetrics::new(n.x.0, n.y.0, n.width.0, n.height.0))
        .collect()
}

fn apply_bounds_to_container(container: &Node, bounds: RectMetrics) -> Result<Node, LayoutError> {
    let mut updated = container.clone();
    let padding = LayoutConstants::SUBGRAPH_PADDING;

    updated.x = to_ordered(bounds.x - padding).map_err(|()| LayoutError::MathFailure)?;
    updated.y = to_ordered(bounds.y - padding).map_err(|()| LayoutError::MathFailure)?;
    updated.width =
        to_ordered(bounds.width + (padding * 2.0)).map_err(|()| LayoutError::MathFailure)?;
    updated.height =
        to_ordered(bounds.height + (padding * 2.0)).map_err(|()| LayoutError::MathFailure)?;

    Ok(updated)
}

fn to_ordered(coord: Coordinate) -> Result<OrderedFloat, ()> {
    OrderedFloat::new(coord.value()).map_err(|_| ())
}

fn calculate_subgraph_bounds(metrics: &[RectMetrics]) -> Option<RectMetrics> {
    if metrics.is_empty() {
        return None;
    }

    let min_x = metrics
        .iter()
        .map(|m| m.x)
        .fold(Coordinate::MAX, Coordinate::min);
    let min_y = metrics
        .iter()
        .map(|m| m.y)
        .fold(Coordinate::MAX, Coordinate::min);

    let (max_x, max_y) = calculate_max_extents(metrics);

    Some(RectMetrics::new(
        min_x.value(),
        min_y.value(),
        (max_x - min_x).value(),
        (max_y - min_y).value(),
    ))
}

fn calculate_max_extents(metrics: &[RectMetrics]) -> (Coordinate, Coordinate) {
    let max_x = metrics
        .iter()
        .map(RectMetrics::right)
        .fold(Coordinate::MIN, Coordinate::max);
    let max_y = metrics
        .iter()
        .map(RectMetrics::bottom)
        .fold(Coordinate::MIN, Coordinate::max);
    (max_x, max_y)
}
