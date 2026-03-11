#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::geometry::operations::compute_subgraph_bounds;
use crate::models::document::{DiagramDocument, NodeId, NodeKind, OrderedFloat};

/// Recomputes bounds for all containers that are ancestors of the given nodes.
///
/// # Returns
/// Number of containers whose bounds were updated.
fn recompute_container_bounds(doc: &mut DiagramDocument, moved_node_ids: &[NodeId]) -> usize {
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
                let padding = 24.0;
                container.x = OrderedFloat(x - padding);
                container.y = OrderedFloat(y - padding);
                container.width = OrderedFloat(width + padding * 2.0);
                container.height = OrderedFloat(height + padding * 2.0);
                updated_count += 1;
            }
        }
    }

    updated_count
}

/// Nudge the selected nodes by dx and dy.
/// Returns true if any nodes were moved.
#[must_use]
pub fn nudge_selection(doc: &mut DiagramDocument, dx: f64, dy: f64) -> bool {
    let selected_nodes: Vec<NodeId> = doc
        .editor_state
        .selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .collect();

    if selected_nodes.is_empty() || (dx == 0.0 && dy == 0.0) {
        return false;
    }

    let mut any_moved = false;
    for node_id in &selected_nodes {
        if let Some(node) = doc.document.nodes.get_mut(node_id) {
            if node.locked && node.kind != NodeKind::Subgraph {
                continue;
            }
            node.x = OrderedFloat(node.x.0 + dx);
            node.y = OrderedFloat(node.y.0 + dy);
            any_moved = true;
        }
    }

    if any_moved {
        // Recompute container bounds after moving (GEO-025)
        let _ = recompute_container_bounds(doc, &selected_nodes);
        doc.revision = doc.revision.increment();
    }

    any_moved
}
