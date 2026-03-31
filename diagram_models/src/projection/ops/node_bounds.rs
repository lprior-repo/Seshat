//! Node bounds propagation for diagram projection
//!
//! This module handles the GEO-026 feature: nested container bounds propagation.
//! When a node moves, its ancestor containers must be resized to contain all children.

use im::HashMap;

use crate::document::{Node, NodeId, NodeKind, OrderedFloat};
use smallvec::SmallVec;

use crate::projection::types::DiagramProjection;
use crate::projection::validation::validate_dimensions;

/// Type alias for node map
type NodeMap = HashMap<NodeId, Node>;

/// Padding added to container bounds around children
pub(crate) const CONTAINER_PADDING: f64 = 24.0;

/// Find all direct children of a container node
pub(crate) fn find_direct_children(
    nodes: &NodeMap,
    container_id: &NodeId,
) -> SmallVec<[NodeId; 8]> {
    nodes
        .iter()
        .filter(|(_, node)| node.parent.as_ref() == Some(container_id))
        .map(|(id, _)| id.clone())
        .collect()
}

/// Recursively find all descendants of a container (children and nested container children)
pub(crate) fn find_all_descendants(nodes: &NodeMap, container_id: &NodeId) -> Vec<NodeId> {
    let mut descendants = Vec::new();
    let direct_children = find_direct_children(nodes, container_id);

    for child_id in direct_children {
        descendants.push(child_id.clone());
        // If child is a subgraph, recursively get its children
        if let Some(child) = nodes.get(&child_id) {
            if child.kind == NodeKind::Subgraph {
                descendants.extend(find_all_descendants(nodes, &child_id));
            }
        }
    }

    descendants
}

/// Compute bounding box for a set of nodes
/// Returns (min_x, min_y, max_x, max_y)
pub(crate) fn compute_bounding_box(
    nodes: &NodeMap,
    node_ids: &[NodeId],
) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for node_id in node_ids {
        if let Some(node) = nodes.get(node_id) {
            min_x = min_x.min(node.x.0);
            min_y = min_y.min(node.y.0);
            max_x = max_x.max(node.x.0 + node.width.0);
            max_y = max_y.max(node.y.0 + node.height.0);
        }
    }

    if min_x == f64::INFINITY {
        None
    } else {
        Some((min_x, min_y, max_x, max_y))
    }
}

/// Recompute bounds for a single container based on its descendants
pub(crate) fn recompute_container_bounds(nodes: &NodeMap, container_id: &NodeId) -> Option<Node> {
    let descendants = find_all_descendants(nodes, container_id);

    if descendants.is_empty() {
        // Empty container - keep current bounds or set to minimum
        return nodes.get(container_id).cloned();
    }

    if let Some((min_x, min_y, max_x, max_y)) = compute_bounding_box(nodes, &descendants) {
        if let Some(container) = nodes.get(container_id) {
            let mut updated = container.clone();
            updated.x = OrderedFloat(min_x - CONTAINER_PADDING);
            updated.y = OrderedFloat(min_y - CONTAINER_PADDING);
            updated.width = OrderedFloat((max_x - min_x) + (CONTAINER_PADDING * 2.0));
            updated.height = OrderedFloat((max_y - min_y) + (CONTAINER_PADDING * 2.0));
            return Some(updated);
        }
    }

    None
}

/// Traverse up the parent chain and collect all ancestor container IDs
pub(crate) fn get_parent_containers(nodes: &NodeMap, node_id: &NodeId) -> SmallVec<[NodeId; 5]> {
    let mut ancestors = SmallVec::new();
    let mut current = nodes.get(node_id).and_then(|n| n.parent.clone());

    while let Some(parent_id) = current {
        if let Some(parent) = nodes.get(&parent_id) {
            if parent.kind == NodeKind::Subgraph {
                ancestors.push(parent_id.clone());
            }
            current = parent.parent.clone();
        } else {
            break;
        }
    }

    ancestors
}

/// Propagate bounds changes to all ancestor containers after a node move
pub(crate) fn propagate_bounds_to_ancestors(nodes: NodeMap, moved_node_id: &NodeId) -> NodeMap {
    let mut current_nodes = nodes;
    let ancestors = get_parent_containers(&current_nodes, moved_node_id);

    // Process from closest parent to furthest (inner to outer)
    for container_id in ancestors {
        if let Some(updated_container) = recompute_container_bounds(&current_nodes, &container_id) {
            current_nodes = current_nodes.update(container_id, updated_container);
        }
    }

    current_nodes
}

/// Apply node resize operation
pub fn apply_node_resize(
    state: DiagramProjection,
    id: &NodeId,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<DiagramProjection, crate::projection::types::ProjectionError> {
    use crate::projection::types::ProjectionError;

    validate_dimensions(width, height)?;
    let node = state
        .nodes
        .get(id)
        .ok_or_else(|| ProjectionError::NodeNotFound(id.to_string()))?;
    let mut updated = node.clone();
    updated.x = OrderedFloat(x);
    updated.y = OrderedFloat(y);
    updated.width = OrderedFloat(width);
    updated.height = OrderedFloat(height);
    let new_nodes = state.nodes.update(id.clone(), updated);
    let new_nodes = propagate_bounds_to_ancestors(new_nodes, id);
    Ok(DiagramProjection {
        nodes: new_nodes,
        ..state
    })
}
