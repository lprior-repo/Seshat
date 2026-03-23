//! Selection operations - select all, clear, delete, group, ungroup, nudge

use std::collections::BTreeSet;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::history::History;
use diagram_models::document::edge_direction::toggle_edge_directions;
use diagram_models::document::{DiagramDocument, EdgeId, Node, NodeId, OrderedFloat};
use diagram_models::envelope::EventEnvelope;
use diagram_models::multi_select::NonEmptyVec;

/// Select all nodes and edges in the document
pub fn apply_select_all(mut doc_signal: Signal<DiagramDocument>) {
    doc_signal.with_mut(|doc| {
        doc.editor_state.selected_items = doc
            .document
            .nodes
            .keys()
            .map(ToString::to_string)
            .chain(doc.document.edges.keys().map(ToString::to_string))
            .collect();
    });
}

/// Clear the current selection
pub fn apply_clear_selection(mut doc_signal: Signal<DiagramDocument>) {
    doc_signal.with_mut(|doc| {
        doc.editor_state.selected_items.clear();
    });
}

/// Delete all selected nodes and edges
#[must_use]
pub fn apply_delete_selected(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
) -> bool {
    let selected = doc_signal.read().editor_state.selected_items.clone();
    if selected.is_empty() {
        return false;
    }

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    doc_signal.with_mut(|doc| {
        let deleted_node_ids =
            selected_nodes_from_selection(&doc.editor_state.selected_items, &doc.document.nodes);
        doc.document.nodes = doc
            .document
            .nodes
            .iter()
            .filter(|(id, _)| !selected.contains(id.as_str()))
            .map(|(id, node)| {
                let mut next = node.clone();
                next.parent = reparent_if_deleted(next.parent, &deleted_node_ids);
                (id.clone(), next)
            })
            .collect();

        let node_ids: im::HashSet<NodeId> = doc.document.nodes.keys().cloned().collect();
        doc.document.edges = doc
            .document
            .edges
            .iter()
            .filter(|(id, edge)| {
                node_ids.contains(&edge.source)
                    && node_ids.contains(&edge.target)
                    && !selected.contains(id.as_str())
            })
            .map(|(id, edge)| (id.clone(), edge.clone()))
            .collect();

        doc.editor_state.selected_items.clear();
        doc.revision = doc.revision.increment();
    });
    true
}

/// Nudge selected nodes by the given delta
#[must_use]
pub fn apply_nudge_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    dx: f64,
    dy: f64,
    push_undo: bool,
) -> bool {
    let selected_nodes = {
        let doc = doc_signal.read();
        selected_node_ids(&doc)
    };
    if selected_nodes.is_empty() || (dx == 0.0 && dy == 0.0) {
        return false;
    }

    if push_undo {
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(doc_signal.read().clone());
    }
    doc_signal.with_mut(|doc| {
        for node_id in selected_nodes {
            if let Some(node) = doc.document.nodes.get_mut(&node_id) {
                if !node.lock_state.is_movable(&node.kind) {
                    continue;
                }
                node.x = OrderedFloat(node.x.0 + dx);
                node.y = OrderedFloat(node.y.0 + dy);
            }
        }
        doc.revision = doc.revision.increment();
    });
    true
}

/// Group selected nodes into a parent container
#[must_use]
pub fn apply_group_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> bool {
    let group_id = NodeId::new(Uuid::new_v4().to_string());

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    let (result, selected_ids): (Result<(), _>, im::HashSet<String>) = doc_signal.with_mut(|doc| {
        let selected = doc.editor_state.selected_items.clone();
        let res = crate::core::grouping::group_selection(doc, &group_id);
        if res.is_ok() {
            doc.revision = doc.revision.increment();
        }
        (res, selected)
    });

    if result.is_ok() {
        if let Some(tx) = db_tx {
            let ids: Vec<String> = selected_ids.iter().cloned().collect();
            let envelope =
                crate::ui::dispatch::create::create_group_envelope(group_id.to_string(), ids);
            tx.send(envelope);
        }
        true
    } else {
        false
    }
}

/// Ungroup a selected group node, releasing its children
#[must_use]
pub fn apply_ungroup_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> bool {
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    let (result, target_ids): (Result<(), _>, Vec<String>) = doc_signal.with_mut(|doc| {
        let targets: Vec<String> = doc.editor_state.selected_items.iter().cloned().collect();
        let res = crate::core::grouping::ungroup_selection(doc);
        if res.is_ok() {
            doc.revision = doc.revision.increment();
        }
        (res, targets)
    });

    if result.is_ok() {
        if let Some(tx) = db_tx {
            for id in target_ids {
                // We only dispatch ungroup for nodes that were actually subgraphs.
                // The reducer in group_ops will handle the validation.
                let envelope = crate::ui::dispatch::create::create_ungroup_envelope(id);
                tx.send(envelope);
            }
        }
        true
    } else {
        false
    }
}

/// Toggle the direction of selected edges (1-way -> 2-way -> 0-way -> 1-way)
///
/// Returns true if the operation succeeded, false if no edges were selected or an error occurred.
#[must_use]
pub fn apply_toggle_edge_direction(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
) -> bool {
    let selected_edge_ids: Vec<EdgeId> = {
        let doc = doc_signal.read();
        doc.editor_state
            .selected_items
            .iter()
            .filter_map(|id| {
                let node_id = diagram_models::document::NodeId::new(id.clone());
                if doc.document.nodes.contains_key(&node_id) {
                    return None;
                }
                let edge_id = EdgeId::new(id.clone());
                doc.document.edges.contains_key(&edge_id).then_some(edge_id)
            })
            .collect()
    };

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
        Err(e) => {
            let _ = e;
            false
        }
    }
}

/// Apply a specific arrow type to all currently selected edges.
/// Returns true if at least one edge was modified.
#[must_use]
pub fn apply_arrow_type_to_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    arrow_type: diagram_models::document::ArrowType,
) -> bool {
    let selected_edge_ids: Vec<EdgeId> = {
        let doc = doc_signal.read();
        doc.editor_state
            .selected_items
            .iter()
            .filter_map(|id| {
                let node_id = diagram_models::document::NodeId::new(id.clone());
                if doc.document.nodes.contains_key(&node_id) {
                    return None;
                }
                let edge_id = EdgeId::new(id.clone());
                doc.document.edges.contains_key(&edge_id).then_some(edge_id)
            })
            .collect()
    };

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

// Private helper functions

fn selected_node_ids(doc: &DiagramDocument) -> BTreeSet<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .filter_map(|id| {
            let node_id = diagram_models::document::NodeId::new(id.clone());
            doc.document.nodes.contains_key(&node_id).then_some(node_id)
        })
        .collect()
}

fn selected_nodes_from_selection(
    selected: &im::HashSet<String>,
    nodes: &im::HashMap<NodeId, Node>,
) -> BTreeSet<NodeId> {
    selected
        .iter()
        .filter_map(|id| {
            let node_id = diagram_models::document::NodeId::new(id.clone());
            nodes.contains_key(&node_id).then_some(node_id)
        })
        .collect()
}

fn reparent_if_deleted(parent: Option<NodeId>, deleted_ids: &BTreeSet<NodeId>) -> Option<NodeId> {
    parent.filter(|parent_id| !deleted_ids.contains(parent_id))
}
