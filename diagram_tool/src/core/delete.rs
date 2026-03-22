use diagram_models::document::DiagramDocument;
use std::collections::HashSet;

pub fn delete_selected(doc: &mut DiagramDocument) -> bool {
    let selected = doc.editor_state.selected_items.clone();
    if selected.is_empty() {
        return false;
    }

    let mut deleted_node_ids = HashSet::new();
    for id_str in &selected {
        let id = diagram_models::document::NodeId::new(id_str.clone());
        if doc.document.nodes.remove(&id).is_some() {
            deleted_node_ids.insert(id);
        }
    }

    // Remove any edges connected to deleted nodes
    let selected_set: HashSet<_> = selected.iter().map(|s| s.as_str()).collect();
    doc.document.edges.retain(|_id, edge| {
        !deleted_node_ids.contains(&edge.source)
            && !deleted_node_ids.contains(&edge.target)
            && !selected_set.contains(edge.source.as_str())
            && !selected_set.contains(edge.target.as_str())
    });

    // Also remove any explicitly selected edges
    for id_str in &selected {
        let id = diagram_models::document::EdgeId::new(id_str.clone());
        doc.document.edges.remove(&id);
    }

    // Reparent children
    for (_id, node) in doc.document.nodes.iter_mut() {
        if let Some(parent_id) = &node.parent {
            if deleted_node_ids.contains(parent_id) {
                node.parent = None;
            }
        }
    }

    doc.editor_state.selected_items.clear();
    true
}

#[cfg(test)]
#[path = "delete_tests.rs"]
mod tests;