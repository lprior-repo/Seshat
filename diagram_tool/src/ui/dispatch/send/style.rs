//! Style dispatch functions

use dioxus::prelude::*;

use crate::models::document::{EdgeId, EdgeStyle, NodeId, NodeStyle};
use crate::models::envelope::{EventEnvelope, LabelTargetId, LabelTargetType};

use super::super::create::{
    create_update_edge_style_envelope, create_update_label_envelope,
    create_update_node_style_envelope,
};
use super::super::errors::{DispatchError, DispatchResult};

/// Dispatch UpdateLabel operation to db_tx
pub fn dispatch_update_label(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    target_id: &str,
    target_type: LabelTargetType,
    old_label: &str,
    new_label: &str,
) -> Result<DispatchResult, DispatchError> {
    match db_tx {
        Some(tx) => {
            // Create appropriate LabelTargetId based on target_type
            let label_target_id = match target_type {
                LabelTargetType::Node => LabelTargetId::node(NodeId::new(target_id.to_string())),
                LabelTargetType::Edge => LabelTargetId::edge(EdgeId::new(target_id.to_string())),
            };
            let envelope = create_update_label_envelope(
                label_target_id,
                target_type,
                old_label.to_string(),
                new_label.to_string(),
            );
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: 1,
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}

/// Dispatch UpdateNodeStyle operation to db_tx
pub fn dispatch_update_node_style(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    id: &str,
    style: NodeStyle,
) -> Result<DispatchResult, DispatchError> {
    match db_tx {
        Some(tx) => {
            let envelope = create_update_node_style_envelope(id.to_string(), style);
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: 1,
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}

/// Dispatch UpdateEdgeStyle operation to db_tx
pub fn dispatch_update_edge_style(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    id: &str,
    style: EdgeStyle,
) -> Result<DispatchResult, DispatchError> {
    match db_tx {
        Some(tx) => {
            let envelope = create_update_edge_style_envelope(id.to_string(), style);
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: 0,
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}
