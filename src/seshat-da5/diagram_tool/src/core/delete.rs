use crate::models::document::{DiagramDocument, NodeId};

pub fn delete_selected(doc: &mut DiagramDocument) -> bool {
    let selected = doc.editor_state.selected_items.clone();
    if selected.is_empty() {
        return false;
    }

    let mut deleted_node_ids = Vec::new();
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        if doc.document.nodes.remove(&id).is_some() {
            deleted_node_ids.push(id);
        }
    }

    // Remove any edges connected to deleted nodes
    doc.document.edges.retain(|_id, edge| {
        !deleted_node_ids.contains(&edge.source)
            && !deleted_node_ids.contains(&edge.target)
            && !selected.contains(edge.source.as_str())
            && !selected.contains(edge.target.as_str())
    });

    // Also remove any explicitly selected edges
    for id_str in &selected {
        let id = crate::models::document::EdgeId::new(id_str.clone());
        doc.document.edges.remove(&id);
    }

    // Reparent children if necessary (omitted for brevity, but needed for subgraphs)

    doc.editor_state.selected_items.clear();
    true
}
