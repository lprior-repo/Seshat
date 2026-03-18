// use crate::geometry::primitives::AABB;
use diagram_models::document::{DiagramDocument, NodeId, NodeKind, OrderedFloat};
pub use diagram_models::grouping::calculations::SUBGRAPH_PADDING;

/// Recomputes bounds for all containers that are ancestors of the given nodes.
///
/// # Returns
/// Number of containers whose bounds were updated.
pub fn recompute_container_bounds(doc: &mut DiagramDocument, moved_node_ids: &[NodeId]) -> usize {
    // Find unique parent containers of the moved nodes
    let mut containers_to_update: Vec<NodeId> = Vec::new();

    for node_id in moved_node_ids {
        if let Some(node) = doc.document.nodes.get(node_id) {
            if let Some(parent_id) = &node.parent {
                // Check if this parent is a subgraph container
                if let Some(parent) = doc.document.nodes.get(parent_id) {
                    if parent.kind == NodeKind::Subgraph {
                        // Only add if not already in list
                        if !containers_to_update.contains(parent_id) {
                            containers_to_update.push(parent_id.clone());
                        }
                    }
                }
            }
        }
    }

    // For each container, recompute bounds from children
    let mut updated_count = 0;
    for container_id in containers_to_update {
        // Collect all children bounds
        let children_bounds: Vec<(f64, f64, f64, f64)> = doc
            .document
            .nodes
            .iter()
            .filter(|(_, node)| node.parent.as_ref() == Some(&container_id))
            .map(|(_, node)| (node.x.0, node.y.0, node.width.0, node.height.0))
            .collect();

        // Compute new bounds
        if let Some((x, y, width, height)) = compute_subgraph_bounds(children_bounds) {
            if let Some(container) = doc.document.nodes.get_mut(&container_id) {
                // Add padding to the computed bounds
                container.x = OrderedFloat(x - SUBGRAPH_PADDING);
                container.y = OrderedFloat(y - SUBGRAPH_PADDING);
                container.width = OrderedFloat(width + SUBGRAPH_PADDING * 2.0);
                container.height = OrderedFloat(height + SUBGRAPH_PADDING * 2.0);
                updated_count += 1;
            }
        }
    }

    updated_count
}

/// Computes the bounding box of a container based on its children's bounds.
///
/// # Parameters
/// - `children`: An iterator of (x, y, width, height) tuples representing child nodes
///
/// # Returns
/// - `Some((x, y, width, height))`: The computed bounds if children exist and are valid
/// - `None`: If there are no children or all children have invalid (NaN/Infinity) coordinates
///
/// # Contract (KIRK-001)
/// - Returns bounds that encompass ALL children geometrically
/// - Returns None if children list is empty
/// - Bounds are minimal (tight fit to children)
/// - All coordinate values are finite (not NaN/Infinity)
#[must_use]
pub fn compute_subgraph_bounds(
    children: impl IntoIterator<Item = (f64, f64, f64, f64)>,
) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut has_valid_child = false;

    for (x, y, width, height) in children {
        // Skip invalid child bounds (NaN or Infinity)
        if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
            continue;
        }

        has_valid_child = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + width);
        max_y = max_y.max(y + height);
    }

    if !has_valid_child {
        return None;
    }

    // Verify final bounds are valid
    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}
