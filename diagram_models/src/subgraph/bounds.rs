//! Logic for recomputing container bounds based on children.

use crate::document::{Node, NodeId, NodeKind, OrderedFloat};
use im::HashMap;
use itertools::Itertools;

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

    containers.into_iter().fold(nodes, |acc, cid| {
        let children_metrics = acc
            .iter()
            .filter(|(_, n)| n.parent.as_ref() == Some(&cid))
            .map(|(_, n)| (n.x.0, n.y.0, n.width.0, n.height.0))
            .collect::<Vec<_>>();

        if let Some((x, y, w, h)) = calculate_subgraph_bounds(&children_metrics) {
            if let Some(container) = acc.get(&cid) {
                let mut updated = container.clone();
                // Use a fixed padding of 24.0
                updated.x = OrderedFloat::new(x - 24.0).unwrap_or(updated.x);
                updated.y = OrderedFloat::new(y - 24.0).unwrap_or(updated.y);
                updated.width = OrderedFloat::new(w + 48.0).unwrap_or(updated.width);
                updated.height = OrderedFloat::new(h + 48.0).unwrap_or(updated.height);
                return acc.update(cid, updated);
            }
        }
        acc
    })
}

fn calculate_subgraph_bounds(nodes: &[(f64, f64, f64, f64)]) -> Option<(f64, f64, f64, f64)> {
    if nodes.is_empty() {
        return None;
    }

    let min_x = nodes.iter().map(|n| n.0).fold(f64::INFINITY, f64::min);
    let min_y = nodes.iter().map(|n| n.1).fold(f64::INFINITY, f64::min);
    let max_x = nodes
        .iter()
        .map(|n| n.0 + n.2)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = nodes
        .iter()
        .map(|n| n.1 + n.3)
        .fold(f64::NEG_INFINITY, f64::max);

    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}
