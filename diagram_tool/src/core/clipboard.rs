use crate::models::document::{DiagramDocument, EdgeId, NodeId};
use crate::ui::commands::Clipboard;
use thiserror::Error;

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

#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod tests;
