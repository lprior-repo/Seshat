use diagram_models::document::{DiagramDocument, NodeId};
use smallvec::SmallVec;

#[must_use]
pub fn selected_node_ids(doc: &DiagramDocument) -> SmallVec<[NodeId; 4]> {
    doc.editor_state
        .selected_items
        .iter()
        .filter_map(|id| {
            let nid = NodeId::new(id.clone());
            // Filter out locked nodes: GEO-024
            doc.document
                .nodes
                .get(&nid)
                .filter(|n| !n.lock_state.is_locked())
                .map(|_| nid)
        })
        .collect()
}

#[must_use]
pub fn selection_bounds(doc: &DiagramDocument) -> Option<(f64, f64, f64, f64)> {
    let selected_items = &doc.editor_state.selected_items;
    if selected_items.is_empty() {
        return None;
    }

    let (min_x, min_y, max_x, max_y) = selected_items
        .iter()
        .filter_map(|id| {
            let nid = NodeId::new(id.clone());
            // Filter out locked nodes: GEO-024 (inlined from selected_node_ids to eliminate intermediate Vec)
            doc.document
                .nodes
                .get(&nid)
                .filter(|n| !n.lock_state.is_locked())
        })
        .fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, min_y, max_x, max_y), n| {
                (
                    min_x.min(n.x.0),
                    min_y.min(n.y.0),
                    max_x.max(n.x.0 + n.width.0),
                    max_y.max(n.y.0 + n.height.0),
                )
            },
        );

    if min_x.is_infinite() {
        None
    } else {
        Some((min_x, min_y, max_x - min_x, max_y - min_y))
    }
}
