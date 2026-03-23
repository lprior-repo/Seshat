//! Clipboard operations - copy, paste, and duplicate
//!
//! This module provides pure functional clipboard operations for the diagram editor.
//! Clipboard state is passed explicitly rather than using mutable state.

use std::collections::{BTreeSet, HashMap};

use dioxus::prelude::*;
use uuid::Uuid;

use crate::history::History;
use diagram_models::clipboard::ClipboardData;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, Node, NodeId, OrderedFloat};

/// Pure function: Checks if the given clipboard has pasteable content
#[must_use]
pub fn clipboard_has_content(clipboard: Option<&ClipboardData>) -> bool {
    clipboard.is_some_and(ClipboardData::has_content)
}

/// Pure function: Creates a clipboard with the selected nodes and edges from the document.
///
/// Returns `None` if no nodes are selected, otherwise returns a new `ClipboardData` with the
/// selected content.
#[must_use]
pub fn copy_selection(doc: &DiagramDocument) -> Option<ClipboardData> {
    let selected_nodes = selected_node_ids(doc);
    if selected_nodes.is_empty() {
        return None;
    }

    let nodes = selected_nodes
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node: &Node| (id.clone(), node.clone()))
        })
        .collect();

    let edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            selected_nodes.contains(&edge.source) && selected_nodes.contains(&edge.target)
        })
        .map(|(_, edge): (&EdgeId, &Edge)| edge.clone())
        .collect();

    Some(ClipboardData {
        nodes,
        edges,
        paste_serial: 0,
    })
}

/// Pure function: Creates a clipboard for duplicate operations.
///
/// Unlike `copy_selection`, this sets `paste_serial` to 1 to indicate
/// the content should be pasted with an offset.
#[must_use]
pub fn copy_selection_for_duplicate(doc: &DiagramDocument) -> Option<ClipboardData> {
    let selected_nodes = selected_node_ids(doc);
    if selected_nodes.is_empty() {
        return None;
    }

    let nodes = selected_nodes
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node: &Node| (id.clone(), node.clone()))
        })
        .collect();

    let edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            selected_nodes.contains(&edge.source) && selected_nodes.contains(&edge.target)
        })
        .map(|(_, edge): (&EdgeId, &Edge)| edge.clone())
        .collect();

    Some(ClipboardData {
        nodes,
        edges,
        paste_serial: 1,
    })
}

/// Pure function: Pastes clipboard content into the document.
///
/// Returns `None` if the clipboard is empty or has no nodes.
/// Otherwise returns a tuple of (`updated_document`, `updated_clipboard`).
#[must_use]
pub fn paste_contents(
    mut clipboard: ClipboardData,
    doc: DiagramDocument,
) -> Option<(DiagramDocument, ClipboardData)> {
    if clipboard.nodes.is_empty() {
        return None;
    }

    clipboard.paste_serial = clipboard.paste_serial.saturating_add(1);
    let serial = clipboard.paste_serial;

    let mut doc = doc;
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
            let new_edge_id = EdgeId::new(Uuid::new_v4().to_string());
            let _ = doc.document.edges.insert(new_edge_id, next);
        }
    }

    doc.editor_state.selected_items = selected;
    doc.revision = doc.revision.increment();

    Some((doc, clipboard))
}

/// Public API: Applies copy operation using a clipboard signal.
///
/// This function maintains backward compatibility with the existing API
/// by using a Dioxus signal for clipboard state management.
#[must_use]
pub fn apply_copy_selection(
    doc_signal: Signal<DiagramDocument>,
    mut clipboard_signal: Signal<Option<ClipboardData>>,
) -> bool {
    let doc = doc_signal.read().clone();
    copy_selection(&doc).is_some_and(|clipboard| {
        clipboard_signal.set(Some(clipboard));
        true
    })
}

/// Public API: Applies paste operation using a clipboard signal.
///
/// Returns true if paste was successful, false otherwise.
#[must_use]
pub fn apply_paste_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut clipboard_signal: Signal<Option<ClipboardData>>,
    history_signal: Signal<History>,
) -> bool {
    let current = doc_signal.read().clone();
    let clipboard = clipboard_signal.read().clone();

    let Some(clipboard) = clipboard else {
        return false;
    };

    let Some((new_doc, new_clipboard)) = paste_contents(clipboard, current) else {
        return false;
    };

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.set(new_doc);
    clipboard_signal.set(Some(new_clipboard));
    true
}

/// Public API: Applies duplicate operation.
///
/// This is equivalent to copy followed by paste, but uses `paste_serial=1`
/// to ensure the duplicated content is offset from the original.
#[must_use]
pub fn apply_duplicate_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut clipboard_signal: Signal<Option<ClipboardData>>,
    history_signal: Signal<History>,
) -> bool {
    let doc = doc_signal.read().clone();
    let Some(clipboard) = copy_selection_for_duplicate(&doc) else {
        return false;
    };

    let Some((new_doc, _)) = paste_contents(clipboard, doc) else {
        return false;
    };

    // Update clipboard with the duplicated content (for subsequent pastes)
    let updated_clipboard = copy_selection_for_duplicate(&new_doc);

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.set(new_doc);
    clipboard_signal.set(updated_clipboard);
    true
}

// Private helper functions

fn selected_node_ids(doc: &DiagramDocument) -> BTreeSet<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .map(|id| diagram_models::document::NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect()
}

fn push_history(mut history_signal: Signal<History>, current: DiagramDocument) {
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);
}

fn remap_pasted_parent(parent: Option<NodeId>, id_map: &HashMap<NodeId, NodeId>) -> Option<NodeId> {
    parent.and_then(|parent_id| id_map.get(&parent_id).cloned().or(Some(parent_id)))
}
