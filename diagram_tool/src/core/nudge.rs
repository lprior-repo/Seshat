#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::geometry::operations::recompute_container_bounds;
use diagram_models::document::{DiagramDocument, NodeId, OrderedFloat};

/// Nudge the selected nodes by dx and dy.
/// Returns true if any nodes were moved.
#[must_use]
pub fn nudge_selection(doc: &mut DiagramDocument, dx: f64, dy: f64) -> bool {
    let selected_nodes: Vec<NodeId> = doc
        .editor_state
        .selected_items
        .iter()
        .map(|id| diagram_models::document::NodeId::new(id.clone()))
        .collect();

    if selected_nodes.is_empty() || (dx == 0.0 && dy == 0.0) {
        return false;
    }

    let mut any_moved = false;
    for node_id in &selected_nodes {
        if let Some(node) = doc.document.nodes.get_mut(node_id) {
            if !node.lock_state.is_movable(&node.kind) {
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
