//! Z-order dispatch functions

use dioxus::prelude::*;

use crate::models::envelope::EventEnvelope;

use super::super::create::{
    create_bring_forward_envelope, create_bring_to_front_envelope, create_send_backward_envelope,
    create_send_to_back_envelope,
};
use super::super::errors::{DispatchError, DispatchResult};

/// Dispatch `BringForward` operation to `db_tx`
///
/// Returns `Ok(DispatchResult)` if `db_tx` is available.
/// Returns Ok with 0 `nodes_affected` when selection is empty (no-op).
pub fn dispatch_bring_forward(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError> {
    if node_ids.is_empty() {
        return Ok(DispatchResult {
            nodes_affected: 0,
            dispatches_sent: 0,
        });
    }

    match db_tx {
        Some(tx) => {
            let envelope = create_bring_forward_envelope(node_ids.to_vec());
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: node_ids.len(),
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}

/// Dispatch `SendBackward` operation to `db_tx`
pub fn dispatch_send_backward(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError> {
    if node_ids.is_empty() {
        return Ok(DispatchResult {
            nodes_affected: 0,
            dispatches_sent: 0,
        });
    }

    match db_tx {
        Some(tx) => {
            let envelope = create_send_backward_envelope(node_ids.to_vec());
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: node_ids.len(),
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}

/// Dispatch `BringToFront` operation to `db_tx`
pub fn dispatch_bring_to_front(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError> {
    if node_ids.is_empty() {
        return Ok(DispatchResult {
            nodes_affected: 0,
            dispatches_sent: 0,
        });
    }

    match db_tx {
        Some(tx) => {
            let envelope = create_bring_to_front_envelope(node_ids.to_vec());
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: node_ids.len(),
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}

/// Dispatch `SendToBack` operation to `db_tx`
pub fn dispatch_send_to_back(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError> {
    if node_ids.is_empty() {
        return Ok(DispatchResult {
            nodes_affected: 0,
            dispatches_sent: 0,
        });
    }

    match db_tx {
        Some(tx) => {
            let envelope = create_send_to_back_envelope(node_ids.to_vec());
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: node_ids.len(),
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}
