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

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    let mut found_any = false;
    for id in selected_items.iter() {
        let nid = NodeId::new(id.clone());
        if let Some(n) = doc.document.nodes.get(&nid) {
            if !n.lock_state.is_locked() {
                found_any = true;
                min_x = min_x.min(n.x.0);
                min_y = min_y.min(n.y.0);
                max_x = max_x.max(n.x.0 + n.width.0);
                max_y = max_y.max(n.y.0 + n.height.0);
            }
        }
    }

    if found_any {
        Some((min_x, min_y, max_x - min_x, max_y - min_y))
    } else {
        None
    }
}
