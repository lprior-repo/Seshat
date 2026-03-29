#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::HashSet;

// Re-export from canvas_math
pub use canvas_math::{safe_zoom, within};

use crate::selection_geometry::selected_node_ids;
use diagram_models::document::{DiagramDocument, NodeId, NodeKind};

#[must_use]
pub fn resize_target_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    let selected = selected_node_ids(doc);
    let selected_set: HashSet<NodeId> = selected.iter().cloned().collect();

    let selected_subgraphs = selected
        .iter()
        .filter_map(|id| doc.document.nodes.get(id).map(|node| (id, node)))
        .filter(|(_, node)| node.kind == NodeKind::Subgraph)
        .map(|(_, node)| (node.x.0, node.y.0, node.width.0, node.height.0))
        .collect::<Vec<_>>();

    if selected_subgraphs.is_empty() {
        return selected;
    }

    doc.document
        .nodes
        .iter()
        .fold(
            selected_set,
            |acc: HashSet<NodeId>, (id, node): (&NodeId, &diagram_models::document::Node)| {
                let node_rect = (node.x.0, node.y.0, node.width.0, node.height.0);
                let included = selected_subgraphs
                    .iter()
                    .any(|subgraph_rect| within(*subgraph_rect, node_rect));
                if included {
                    let mut updated = acc;
                    let _ = updated.insert(id.clone());
                    updated
                } else {
                    acc
                }
            },
        )
        .into_iter()
        .collect::<Vec<_>>()
}
