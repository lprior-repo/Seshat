use crate::models::document::OrderedFloat;
use crate::models::document::{DiagramDocument, EdgeId, NodeId};
use crate::ui::commands::Clipboard;
use im::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, PartialEq)]
pub enum ClipboardError {
    #[error("Selection is empty")]
    EmptySelection,
    #[error("Clipboard is empty")]
    EmptyClipboard,
    #[error("Edge connects to an unselected node")]
    DanglingEdge(EdgeId),
}

/// Creates a Clipboard payload from the currently selected items.
/// Drops any edges that refer to non-selected nodes to maintain referential integrity.
pub fn copy_selection(doc: &DiagramDocument) -> Result<Clipboard, ClipboardError> {
    let selected = &doc.editor_state.selected_items;

    if selected.is_empty() {
        return Err(ClipboardError::EmptySelection);
    }

    let mut nodes = Vec::new();
    for id_str in selected.iter() {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get(&id) {
            nodes.push((id, node.clone()));
        }
    }

    let mut edges = Vec::new();
    for (id, edge) in &doc.document.edges {
        // Only include edges where both endpoints are in the selection
        if selected.contains(edge.source.as_str()) && selected.contains(edge.target.as_str()) {
            edges.push(edge.clone());
        }
    }

    Ok(Clipboard {
        nodes,
        edges,
        paste_serial: 0,
    })
}

fn remap_pasted_parent(parent: Option<NodeId>, id_map: &HashMap<NodeId, NodeId>) -> Option<NodeId> {
    parent.and_then(|parent_id| id_map.get(&parent_id).cloned().or(Some(parent_id)))
}

pub fn paste_contents(
    mut clipboard: Clipboard,
    doc: &mut DiagramDocument,
) -> Result<Clipboard, ClipboardError> {
    if clipboard.nodes.is_empty() {
        return Err(ClipboardError::EmptyClipboard);
    }

    clipboard.paste_serial = clipboard.paste_serial.saturating_add(1);
    let serial = clipboard.paste_serial;

    let offset = 20.0 * f64::from(serial.max(1));
    let id_map = clipboard
        .nodes
        .iter()
        .map(|(old_id, _)| (old_id.clone(), NodeId::new(Uuid::new_v4().to_string())))
        .collect::<HashMap<_, _>>();
    let mut selected = im::HashSet::new();

    for (old_id, node) in &clipboard.nodes {
        let Some(new_id) = id_map.get(old_id).cloned() else {
            continue;
        };
        let mut next = node.clone();
        next.x = OrderedFloat(next.x.0 + offset);
        next.y = OrderedFloat(next.y.0 + offset);
        next.parent = remap_pasted_parent(next.parent, &id_map);
        let _ = selected.insert(new_id.to_string());
        let _ = doc.document.nodes.insert(new_id, next);
    }

    for edge in &clipboard.edges {
        if let (Some(new_source), Some(new_target)) =
            (id_map.get(&edge.source), id_map.get(&edge.target))
        {
            let mut next = edge.clone();
            next.source = new_source.clone();
            next.target = new_target.clone();
            let new_edge_id = crate::models::document::EdgeId::new(Uuid::new_v4().to_string());
            let _ = doc.document.edges.insert(new_edge_id, next);
        }
    }

    doc.editor_state.selected_items = selected;
    Ok(clipboard)
}

#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod tests;
