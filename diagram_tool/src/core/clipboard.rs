#![allow(dead_code)]

use crate::ui::commands::ClipboardData;
use diagram_models::document::OrderedFloat;
use diagram_models::document::{DiagramDocument, EdgeId, NodeId};
use im::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, PartialEq, Eq)]
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
///
/// # Errors
///
/// Returns `ClipboardError::EmptySelection` if no items are selected.
pub fn copy_selection(doc: &DiagramDocument) -> Result<ClipboardData, ClipboardError> {
    let selected = &doc.editor_state.selected_items;

    if selected.is_empty() {
        return Err(ClipboardError::EmptySelection);
    }

    let mut nodes = Vec::new();
    for id_str in selected.iter() {
        let id = diagram_models::document::NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get(&id) {
            nodes.push((id, node.clone()));
        }
    }

    let mut edges = Vec::new();
    for (_id, edge) in &doc.document.edges {
        // Only include edges where both endpoints are in the selection
        if selected.contains(edge.source.as_str()) && selected.contains(edge.target.as_str()) {
            edges.push(edge.clone());
        }
    }

    Ok(ClipboardData {
        nodes,
        edges,
        paste_serial: 0,
    })
}

fn remap_pasted_parent(parent: Option<NodeId>, id_map: &HashMap<NodeId, NodeId>) -> Option<NodeId> {
    parent.and_then(|parent_id| id_map.get(&parent_id).cloned().or(Some(parent_id)))
}

/// Pastes clipboard contents into the document at offset positions.
///
/// # Errors
///
/// Returns `ClipboardError::EmptyClipboard` if clipboard has no nodes.
pub fn paste_contents(
    mut clipboard: ClipboardData,
    doc: &mut DiagramDocument,
) -> Result<ClipboardData, ClipboardError> {
    if clipboard.nodes.is_empty() {
        return Err(ClipboardError::EmptyClipboard);
    }

    clipboard.paste_serial = clipboard.paste_serial.saturating_add(1);
    let serial = clipboard.paste_serial;

    let offset = 20.0 * f64::from(serial.max(1));
    let id_map = clipboard
        .nodes
        .iter()
        .map(|(old_id, _): &(NodeId, diagram_models::document::Node)| {
            (old_id.clone(), NodeId::new(Uuid::new_v4().to_string()))
        })
        .collect::<HashMap<NodeId, NodeId>>();
    let mut selected = im::HashSet::new();

    for (old_id, node) in &clipboard.nodes {
        let Some(new_id) = id_map.get(old_id).cloned() else {
            continue;
        };
        let mut next: diagram_models::document::Node = node.clone();
        next.x = OrderedFloat(next.x.0 + offset);
        next.y = OrderedFloat(next.y.0 + offset);
        next.parent = remap_pasted_parent(next.parent, &id_map);
        let _ = selected.insert(new_id.as_str().to_string());
        let _ = doc.document.nodes.insert(new_id, next);
    }

    for edge in &clipboard.edges {
        if let (Some(new_source), Some(new_target)) =
            (id_map.get(&edge.source), id_map.get(&edge.target))
        {
            let mut next = edge.clone();
            next.source = new_source.clone();
            next.target = new_target.clone();
            let new_edge_id = diagram_models::document::EdgeId::new(Uuid::new_v4().to_string());
            let _ = doc.document.edges.insert(new_edge_id, next);
        }
    }

    doc.editor_state.selected_items = selected;
    Ok(clipboard)
}

/// Cuts the currently selected items from the document and returns them.
/// Removes the items and any connected edges from the document.
/// Leaves the document with an empty selection.
///
/// # Errors
///
/// Returns `ClipboardError::EmptySelection` if no items are selected.
pub fn cut_selection(doc: &mut DiagramDocument) -> Result<ClipboardData, ClipboardError> {
    let clipboard = copy_selection(doc)?;
    let selected = &doc.editor_state.selected_items;

    let new_nodes = doc
        .document
        .nodes
        .iter()
        .filter(|(id, _)| !selected.contains(id.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let new_edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            !selected.contains(edge.source.as_str()) && !selected.contains(edge.target.as_str())
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    doc.document.nodes = new_nodes;
    doc.document.edges = new_edges;
    doc.editor_state.selected_items = im::HashSet::new();

    Ok(clipboard)
}

/// Duplicates the currently selected items within the document.
/// Bypasses the external clipboard entirely.
/// Applies a spatial offset to the new nodes.
///
/// # Errors
///
/// Returns `ClipboardError::EmptySelection` if no items are selected.
pub fn duplicate_selection(doc: &mut DiagramDocument) -> Result<(), ClipboardError> {
    copy_selection(doc)
        .and_then(|clipboard| paste_contents(clipboard, doc))
        .map(|_| ())
}

#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod tests;
