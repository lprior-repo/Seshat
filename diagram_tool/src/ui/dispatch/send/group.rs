//! Group dispatch functions

use dioxus::prelude::*;

use crate::models::envelope::EventEnvelope;

use super::super::create::{create_group_envelope, create_ungroup_envelope};
use super::super::errors::{DispatchError, DispatchResult};

/// Dispatch Group operation to `db_tx`
pub fn dispatch_group(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    group_id: &str,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError> {
    // No-op if fewer than 2 nodes selected
    if node_ids.len() < 2 {
        return Ok(DispatchResult {
            nodes_affected: 0,
            dispatches_sent: 0,
        });
    }

    match db_tx {
        Some(tx) => {
            let envelope = create_group_envelope(group_id.to_string(), node_ids.to_vec());
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: node_ids.len(),
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}

/// Dispatch Ungroup operation to `db_tx`
///
/// Returns `Ok(DispatchResult)` if `db_tx` is available.
///
/// # Errors
/// Returns `Err(DispatchError::WalDisconnected)` if `db_tx` is None.
pub fn dispatch_ungroup(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    group_id: &str,
) -> Result<DispatchResult, DispatchError> {
    match db_tx {
        Some(tx) => {
            let envelope = create_ungroup_envelope(group_id.to_string());
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: 1,
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}
