#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use im::HashMap;

use crate::stubs::dispatch_update_label;
use crate::stubs::mutate_doc_with_history;
use crate::stubs::LabelTargetType;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, Node, NodeId};
use diagram_models::envelope::EventEnvelope;
use diagram_models::history::History;

use super::types::CommitError;

/// Commits an inline edit for a node or edge.
///
/// # Errors
///
/// Returns `CommitError` if the target is not found or dispatch fails.
pub fn commit_inline_edit(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    node_target: Option<NodeId>,
    edge_target: Option<EdgeId>,
    edit_value: Signal<String>,
    db_tx: Option<Coroutine<EventEnvelope>>,
) -> Result<bool, CommitError> {
    if let Some(node_id) = node_target {
        let changed = commit_node_edit(
            &mut doc_signal,
            &mut history_signal,
            &node_id,
            &edit_value,
            db_tx.as_ref(),
        )?;
        return Ok(changed);
    }

    if let Some(edge_id) = edge_target {
        let changed = commit_edge_edit(
            &mut doc_signal,
            &mut history_signal,
            &edge_id,
            &edit_value,
            db_tx.as_ref(),
        )?;
        return Ok(changed);
    }

    Ok(false)
}

fn commit_node_edit(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    node_id: &NodeId,
    edit_value: &Signal<String>,
    db_tx: Option<&Coroutine<EventEnvelope>>,
) -> Result<bool, CommitError> {
    let new_label = edit_value.read().clone();
    let current_label = current_node_label(doc_signal, node_id);

    ensure_node_exists(doc_signal, node_id)?;

    if current_label == new_label {
        return Ok(false);
    }

    dispatch_label_to_db(
        db_tx,
        node_id.as_str(),
        LabelTargetType::Node,
        &current_label,
        &new_label,
    );
    apply_node_label_change(doc_signal, history_signal, node_id, &new_label);
    Ok(true)
}

fn current_node_label(doc_signal: &Signal<DiagramDocument>, node_id: &NodeId) -> String {
    doc_signal
        .read()
        .document
        .nodes
        .get(node_id)
        .map_or_else(String::new, |n| n.label.clone())
}

fn ensure_node_exists(
    doc_signal: &Signal<DiagramDocument>,
    node_id: &NodeId,
) -> Result<(), CommitError> {
    doc_signal
        .read()
        .document
        .nodes
        .get(node_id)
        .map_or(Err(CommitError::TargetNotFound), |_| Ok(()))
}

fn dispatch_label_to_db(
    db_tx: Option<&Coroutine<EventEnvelope>>,
    target_id: &str,
    target_type: LabelTargetType,
    old_label: &str,
    new_label: &str,
) {
    dispatch_update_label(db_tx, target_id, target_type, old_label, new_label)
        .map_err(CommitError::DispatchFailed)
        .ok();
}

fn apply_node_label_change(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    node_id: &NodeId,
    new_label: &str,
) {
    let doc = doc_signal.read();
    let new_nodes = doc
        .document
        .nodes
        .iter()
        .map(|(id, node)| {
            let updated = if *id == *node_id {
                Node {
                    label: new_label.to_string(),
                    ..node.clone()
                }
            } else {
                node.clone()
            };
            (id.clone(), updated)
        })
        .collect();

    let new_doc = build_updated_doc(&doc, new_nodes, doc.document.edges.clone());
    drop(doc);

    mutate_doc_with_history(doc_signal, history_signal, |_| Ok::<_, ()>(new_doc)).ok();
}

fn build_updated_doc(
    doc: &DiagramDocument,
    new_nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
) -> DiagramDocument {
    DiagramDocument {
        version: doc.version,
        revision: doc.revision.increment(),
        document: diagram_models::document::DocumentData {
            nodes: new_nodes,
            edges,
        },
        editor_state: doc.editor_state.clone(),
    }
}

fn commit_edge_edit(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    edge_id: &EdgeId,
    edit_value: &Signal<String>,
    db_tx: Option<&Coroutine<EventEnvelope>>,
) -> Result<bool, CommitError> {
    let new_label = edit_value.read().clone();
    let current_label = current_edge_label(doc_signal, edge_id);

    ensure_edge_exists(doc_signal, edge_id)?;

    if current_label == new_label {
        return Ok(false);
    }

    dispatch_label_to_db(
        db_tx,
        edge_id.as_str(),
        LabelTargetType::Edge,
        &current_label,
        &new_label,
    );
    apply_edge_label_change(doc_signal, history_signal, edge_id, &new_label);
    Ok(true)
}

fn current_edge_label(doc_signal: &Signal<DiagramDocument>, edge_id: &EdgeId) -> String {
    doc_signal
        .read()
        .document
        .edges
        .get(edge_id)
        .map_or_else(String::new, |e| e.label.clone())
}

fn ensure_edge_exists(
    doc_signal: &Signal<DiagramDocument>,
    edge_id: &EdgeId,
) -> Result<(), CommitError> {
    doc_signal
        .read()
        .document
        .edges
        .get(edge_id)
        .map_or(Err(CommitError::TargetNotFound), |_| Ok(()))
}

fn apply_edge_label_change(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    edge_id: &EdgeId,
    new_label: &str,
) {
    let doc = doc_signal.read();
    let new_edges = doc
        .document
        .edges
        .iter()
        .map(|(id, edge)| {
            let updated = if *id == *edge_id {
                Edge {
                    label: new_label.to_string(),
                    ..edge.clone()
                }
            } else {
                edge.clone()
            };
            (id.clone(), updated)
        })
        .collect();

    let new_doc = build_updated_doc(&doc, doc.document.nodes.clone(), new_edges);
    drop(doc);

    mutate_doc_with_history(doc_signal, history_signal, |_| Ok::<_, ()>(new_doc)).ok();
}

#[cfg(test)]
#[path = "commit_tests.rs"]
mod tests;
