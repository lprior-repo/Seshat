//! Edge dispatch functions

use dioxus::prelude::*;
use im::HashSet;

use crate::models::document::{DiagramDocument, EdgeId, NodeId};
use crate::models::envelope::EventEnvelope;

use super::create::{create_edge_connect_envelope, create_edge_disconnect_envelope};
use super::errors::{DispatchError, DispatchResult};
use super::validators::edge_preserves_dag;

/// Dispatch EdgeConnect operation to db_tx
///
/// Returns `Ok(DispatchResult)` if db_tx is available and edge passes DAG validation.
///
/// # Errors
/// Returns `Err(DispatchError::ChannelMissing)` if db_tx is None (seshat-088).
/// Returns `Err(DispatchError::SelfLoop)` if source equals target.
/// Returns `Err(DispatchError::CycleDetected)` if edge would create a DAG cycle.
pub fn dispatch_edge_connect(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    doc: &DiagramDocument,
    edge_id: String,
    source: String,
    target: String,
) -> Result<DispatchResult, DispatchError> {
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

/// Dispatch EdgeDisconnect operation to db_tx
///
/// Returns `Ok(DispatchResult)` if db_tx is available and preconditions are met.
///
/// # Errors
/// Returns `Err(DispatchError::NoTx)` if db_tx is None (seshat-5zs).
/// Returns `Err(DispatchError::NotSelected)` if edge_id is not in selected_items.
/// Returns `Err(DispatchError::EdgeNotFound)` if edge_id does not exist in document.edges.
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
