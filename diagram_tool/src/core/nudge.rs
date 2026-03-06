#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{DiagramDocument, NodeId, NodeKind, OrderedFloat};

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
    for node_id in selected_nodes {
        if let Some(node) = doc.document.nodes.get_mut(&node_id) {
            if node.locked && node.kind != NodeKind::Subgraph {
                continue;
            }
            node.x = OrderedFloat(node.x.0 + dx);
            node.y = OrderedFloat(node.y.0 + dy);
            any_moved = true;
        }
    }

    if any_moved {
        doc.revision = doc.revision.increment();
    }

    any_moved
}
