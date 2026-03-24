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
#[must_use]
pub fn apply_duplicate_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut clipboard_signal: Signal<Option<ClipboardData>>,
    history_signal: Signal<History>,
) -> bool {
    let doc = doc_signal.read().clone();
    let selection = selected_node_ids(&doc);

    let Some(new_data) = duplicate_clipboard_contents(&doc, &selection) else {
        return false;
    };

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.set(new_data.0);
    clipboard_signal.set(new_data.1);
    true
}

fn duplicate_clipboard_contents(
    doc: &DiagramDocument,
    selection: &[NodeId],
) -> Option<(DiagramDocument, Option<ClipboardData>)> {
    let mut clipboard = copy_selection(doc, selection)?;
    clipboard.paste_serial = 1;

    let (new_doc, _) = paste_contents(clipboard, doc.clone())?;

    let new_selection = selected_node_ids(&new_doc);
    let new_clipboard = copy_selection(&new_doc, &new_selection);

    Some((new_doc, new_clipboard))
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
