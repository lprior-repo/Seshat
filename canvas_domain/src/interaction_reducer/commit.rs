#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use im::HashMap;

use crate::stubs::dispatch_update_label;
use crate::stubs::mutate_doc_with_history;
use crate::stubs::LabelTargetType;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, Node, NodeId};
use diagram_models::envelope::EventEnvelope;
use diagram_models::history::History;
use diagram_models::validation::is_valid_label;

// Re-export error types for consumers
pub use super::types::{CommitError, LabelEditError};

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
    if !is_valid_label(&new_label) {
        return Err(LabelEditError::ValidationError.into());
    }

    let node_id_clone = node_id.clone();
    let new_label_clone = new_label.clone();
    let mut old_label = String::new();

    mutate_doc_with_history(doc_signal, history_signal, |current_doc| {
        let node = current_doc
            .document
            .nodes
            .get(&node_id_clone)
            .ok_or(LabelEditError::TargetNotFound)?;

        old_label.clone_from(&node.label);

        if old_label == new_label_clone {
            return Ok(current_doc.clone());
        }
        calculate_node_label_edit(current_doc, &node_id_clone, &new_label_clone)
    })?;

    if old_label == new_label {
        return Ok(false);
    }

    dispatch_label_to_db(
        db_tx,
        node_id.as_str(),
        LabelTargetType::Node,
        &old_label,
        &new_label,
    );
    Ok(true)
}

fn dispatch_label_to_db(
    db_tx: Option<&Coroutine<EventEnvelope>>,
    target_id: &str,
    target_type: LabelTargetType,
    old_label: &str,
    new_label: &str,
) {
    dispatch_update_label(db_tx, target_id, target_type, old_label, new_label)
        .map_err(CommitError::UpdateFailed)
        .ok();
}

pub fn calculate_node_label_edit(
    doc: &DiagramDocument,
    node_id: &NodeId,
    new_label: &str,
) -> Result<DiagramDocument, LabelEditError> {
    if !is_valid_label(new_label) {
        return Err(LabelEditError::ValidationError);
    }
    if !doc.document.nodes.contains_key(node_id) {
        return Err(LabelEditError::TargetNotFound);
    }

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

    Ok(build_updated_doc(
        doc,
        new_nodes,
        doc.document.edges.clone(),
    ))
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
    if !is_valid_label(&new_label) {
        return Err(LabelEditError::ValidationError.into());
    }

    let edge_id_clone = edge_id.clone();
    let new_label_clone = new_label.clone();
    let mut old_label = String::new();

    mutate_doc_with_history(doc_signal, history_signal, |current_doc| {
        let edge = current_doc
            .document
            .edges
            .get(&edge_id_clone)
            .ok_or(LabelEditError::TargetNotFound)?;

        old_label.clone_from(&edge.label);

        if old_label == new_label_clone {
            return Ok(current_doc.clone());
        }
        calculate_edge_label_edit(current_doc, &edge_id_clone, &new_label_clone)
    })?;

    if old_label == new_label {
        return Ok(false);
    }

    dispatch_label_to_db(
        db_tx,
        edge_id.as_str(),
        LabelTargetType::Edge,
        &old_label,
        &new_label,
    );

    Ok(true)
}

pub fn calculate_edge_label_edit(
    doc: &DiagramDocument,
    edge_id: &EdgeId,
    new_label: &str,
) -> Result<DiagramDocument, LabelEditError> {
    if !is_valid_label(new_label) {
        return Err(LabelEditError::ValidationError);
    }
    if !doc.document.edges.contains_key(edge_id) {
        return Err(LabelEditError::TargetNotFound);
    }

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

    Ok(build_updated_doc(
        doc,
        doc.document.nodes.clone(),
        new_edges,
    ))
}

#[cfg(test)]
#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
#[path = "commit_tests.rs"]
mod tests;
