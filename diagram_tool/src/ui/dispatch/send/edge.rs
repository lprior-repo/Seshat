//! Edge dispatch functions

use dioxus::prelude::*;
use im::HashSet;
use uuid::Uuid;

use crate::models::document::{DiagramDocument, EdgeId, NodeId};
use crate::models::envelope::EventEnvelope;

use super::super::create::{create_edge_connect_envelope, create_edge_disconnect_envelope};
use super::super::errors::{DispatchError, DispatchResult};
use super::super::validators::edge_preserves_dag;

/// Validate preconditions for edge connection (P1-P4)
///
/// Returns `Ok(())` if all preconditions are met.
/// Returns `Err(DispatchError::EdgeNotFound)` if any precondition fails.
#[must_use]
pub fn validate_edge_connect_preconditions(
    doc: &DiagramDocument,
    source_id: &str,
    target_id: &str,
) -> Result<(), DispatchError> {
    // P1: source_id non-empty
    if source_id.is_empty() {
        return Err(DispatchError::EdgeNotFound);
    }
    // P2: target_id non-empty
    if target_id.is_empty() {
        return Err(DispatchError::EdgeNotFound);
    }
    // P3: source exists in doc.nodes
    let source_node_id = NodeId::new(source_id.to_string());
    if !doc.document.nodes.contains_key(&source_node_id) {
        return Err(DispatchError::EdgeNotFound);
    }
    // P4: target exists in doc.nodes
    let target_node_id = NodeId::new(target_id.to_string());
    if !doc.document.nodes.contains_key(&target_node_id) {
        return Err(DispatchError::EdgeNotFound);
    }
    Ok(())
}

/// Dispatch `EdgeConnect` operation to `db_tx`
///
/// Returns `Ok(DispatchResult)` if `db_tx` is available and edge passes DAG validation.
///
/// # Errors
/// Returns `Err(DispatchError::ChannelMissing)` if `db_tx` is None (seshat-088).
/// Returns `Err(DispatchError::SelfLoop)` if source equals target.
/// Returns `Err(DispatchError::CycleDetected)` if edge would create a DAG cycle.
/// Returns `Err(DispatchError::EdgeNotFound)` if preconditions P1-P4 fail.
pub fn dispatch_edge_connect(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    doc: &DiagramDocument,
    edge_id: String,
    source: String,
    target: String,
) -> Result<DispatchResult, DispatchError> {
    // P1-P4: Validate preconditions
    validate_edge_connect_preconditions(doc, &source, &target)?;

    // Validate source != target (self-loop)
    if source == target {
        return Err(DispatchError::SelfLoop);
    }

    let source_id = NodeId::new(source.clone());
    let target_id = NodeId::new(target.clone());

    // Validate DAG - check if adding this edge would create a cycle
    if !edge_preserves_dag(
        &doc.document.nodes,
        &doc.document.edges,
        &source_id,
        &target_id,
    ) {
        return Err(DispatchError::CycleDetected);
    }

    match db_tx {
        Some(tx) => {
            let envelope = create_edge_connect_envelope(edge_id, source, target);
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: 1,
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::ChannelMissing),
    }
}

/// Dispatch `EdgeDisconnect` operation to `db_tx`
///
/// Returns `Ok(DispatchResult)` if `db_tx` is available and preconditions are met.
///
/// # Errors
/// Returns `Err(DispatchError::NoTx)` if `db_tx` is None (seshat-5zs).
/// Returns `Err(DispatchError::NotSelected)` if `edge_id` is not in `selected_items`.
/// Returns `Err(DispatchError::EdgeNotFound)` if `edge_id` does not exist in document.edges.
pub fn dispatch_edge_disconnect(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    doc: &DiagramDocument,
    selected_items: &HashSet<String>,
    edge_id: &str,
) -> Result<DispatchResult, DispatchError> {
    // P1: Check edge is in selection
    if !selected_items.contains(edge_id) {
        return Err(DispatchError::NotSelected);
    }

    // P2: Check edge exists in document
    let edge_key = EdgeId::new(edge_id.to_string());
    if !doc.document.edges.contains_key(&edge_key) {
        return Err(DispatchError::EdgeNotFound);
    }

    match db_tx {
        Some(tx) => {
            let envelope = create_edge_disconnect_envelope(edge_id.to_string());
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: 1,
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::NoTx),
    }
}

/// Handle edge drawing completion from UI
///
/// This is the UI-facing function that gets called when a user completes drawing an edge.
/// It validates preconditions, generates an edge ID, and dispatches the `EdgeConnect` operation.
///
/// # Arguments
/// * `db_tx` - The coroutine channel for sending events to the WAL
/// * `doc` - The current diagram document
/// * `source_id` - The source node ID (string)
/// * `target_id` - The target node ID (string)
///
/// # Returns
/// * `Ok(DispatchResult)` - If the edge was successfully created and dispatched
/// * `Err(DispatchError)` - If any precondition fails
///
/// # Preconditions (P1-P4)
/// * P1: `source_id` must be non-empty -> `EdgeNotFound`
/// * P2: `target_id` must be non-empty -> `EdgeNotFound`
/// * P3: `source_id` must exist in doc.nodes -> `EdgeNotFound`
/// * P4: `target_id` must exist in doc.nodes -> `EdgeNotFound`
///
/// # Postconditions (Q1-Q3)
/// * Q1: Returns Ok(DispatchResult) with `EdgeConnect` operation
/// * Q2: Operation dispatched to `db_tx` channel
/// * Q3: source/target `NodeIds` properly mapped
pub fn handle_edge_drawing_complete(
    db_tx: Option<Coroutine<EventEnvelope>>,
    doc: &DiagramDocument,
    source_id: String,
    target_id: String,
) -> Result<DispatchResult, DispatchError> {
    // P1-P4: Validate all preconditions first
    validate_edge_connect_preconditions(doc, &source_id, &target_id)?;

    // Generate unique edge ID (Q1: unique edge ID generated)
    let edge_id = Uuid::new_v4().to_string();

    // Dispatch to db_tx (Q2: operation dispatched)
    dispatch_edge_connect(&db_tx, doc, edge_id, source_id, target_id)
}
