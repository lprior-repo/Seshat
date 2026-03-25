//! Operations on the current edge selection (toggle direction, apply arrow types).

use dioxus::prelude::*;

use crate::history::History;
use diagram_models::document::edge_direction::toggle_edge_directions;
use diagram_models::document::{DiagramDocument, EdgeId, NodeId};
use diagram_models::multi_select::NonEmptyVec;

// ---------------------------------------------------------------------------
// Pure helper — no I/O, no signals
// ---------------------------------------------------------------------------

/// Collect all edge IDs that are currently selected in the document.
///
/// An item in `selected_items` is considered an edge only when it is present
/// in `doc.document.edges` AND absent from `doc.document.nodes`. This mirrors
/// the rule that node IDs and edge IDs live in the same flat selection set.
fn collect_selected_edge_ids(doc: &DiagramDocument) -> Vec<EdgeId> {
    doc.editor_state
        .selected_items
        .iter()
        .filter_map(|id| {
            let node_id = NodeId::new(id.clone());
            if doc.document.nodes.contains_key(&node_id) {
                return None;
            }
            let edge_id = EdgeId::new(id.clone());
            doc.document.edges.contains_key(&edge_id).then_some(edge_id)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// Toggle the direction of selected edges (1-way → 2-way → 0-way → 1-way).
///
/// Returns `true` if the operation changed at least one edge.
/// Returns `false` if the selection is empty or the toggle produced no change.
#[must_use]
pub fn apply_toggle_edge_direction(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
) -> bool {
    let selected_edge_ids = collect_selected_edge_ids(&doc_signal.read());

    if selected_edge_ids.is_empty() {
        return false;
    }

    let selection: NonEmptyVec<String> = match NonEmptyVec::try_from(
        selected_edge_ids
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>(),
    ) {
        Ok(sel) => sel,
        Err(_) => return false,
    };

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    let result = doc_signal.with_mut(|doc| toggle_edge_directions(doc, &selection));

    match result {
        Ok(()) => {
            doc_signal.with_mut(|doc| {
                doc.revision = doc.revision.increment();
            });
            true
        }
        Err(_toggle_err) => {
            // toggle_edge_directions failed (e.g. edge disappeared between
            // read and write). History snapshot was already pushed; the
            // document state is unchanged because with_mut was not applied
            // on error. The caller receives false — no silent discard.
            false
        }
    }
}

/// Apply a specific arrow type to all currently selected edges.
///
/// Returns `true` if at least one edge was modified, `false` otherwise.
#[must_use]
pub fn apply_arrow_type_to_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    arrow_type: diagram_models::document::ArrowType,
) -> bool {
    let selected_edge_ids = collect_selected_edge_ids(&doc_signal.read());

    if selected_edge_ids.is_empty() {
        return false;
    }

    let will_change = {
        let doc = doc_signal.read();
        selected_edge_ids.iter().any(|edge_id| {
            doc.document
                .edges
                .get(edge_id)
                .is_some_and(|e| e.arrow_type != arrow_type)
        })
    };

    if !will_change {
        return false;
    }

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    doc_signal.with_mut(|doc| {
        for edge_id in selected_edge_ids {
            if let Some(edge) = doc.document.edges.get_mut(&edge_id) {
                edge.arrow_type = arrow_type;
            }
        }
        doc.revision = doc.revision.increment();
    });

    true
}
