//! Clipboard operations - copy, paste, and duplicate
//!
//! This module provides pure functional clipboard operations for the diagram editor.
//! Clipboard state is passed explicitly rather than using mutable state.

use crate::history::History;
pub use diagram_models::clipboard::ClipboardData;
use diagram_models::clipboard::{calculate_paste, copy_selection};
use diagram_models::document::{DiagramDocument, NodeId};
use dioxus::prelude::*;

/// Pure function: Checks if the given clipboard has pasteable content
#[must_use]
pub fn clipboard_has_content(clipboard: Option<&ClipboardData>) -> bool {
    clipboard.is_some_and(ClipboardData::has_content)
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

    let result = calculate_paste(&clipboard, &doc);
    clipboard.paste_serial = clipboard.paste_serial.saturating_add(1);

    let mut doc = doc;
    doc.document.nodes = result.nodes;
    doc.document.edges = result.edges;
    doc.editor_state.selected_items = result.selected;
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
    let selection = selected_node_ids(&doc);
    copy_selection(&doc, &selection).is_some_and(|clipboard| {
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
    let selection = selected_node_ids(&doc);
    let Some(mut clipboard) = copy_selection(&doc, &selection) else {
        return false;
    };

    // Duplicate uses paste_serial=1 for offset
    clipboard.paste_serial = 1;

    let Some((new_doc, _)) = paste_contents(clipboard, doc) else {
        return false;
    };

    push_history(history_signal, doc_signal.read().clone());

    // Copy selection before moving new_doc
    let selection = selected_node_ids(&new_doc);
    let new_clipboard = copy_selection(&new_doc, &selection);

    doc_signal.set(new_doc);
    clipboard_signal.set(new_clipboard);
    true
}

// Private helper functions

fn selected_node_ids(doc: &DiagramDocument) -> Vec<NodeId> {
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
