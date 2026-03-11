//! Node dispatch functions

use dioxus::prelude::*;

use crate::models::document::NodeId;
use crate::models::envelope::EventEnvelope;

use super::create::{
    create_node_add_envelope, create_node_delete_envelope, create_node_resize_envelope,
};
use super::errors::{DispatchError, DispatchResult};
use super::send::ResizeBounds;

/// Dispatch NodeAdd operation to db_tx
///
/// Returns `Ok(DispatchResult)` if db_tx is available.
/// The caller is responsible for local document update after dispatch.
/// Note: Dioxus Coroutine::send() returns () - we fire and forget.
///
/// # Errors
/// Returns `Err(DispatchError::WalDisconnected)` if db_tx is None.
pub fn dispatch_node_add(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    envelope: EventEnvelope,
) -> Result<DispatchResult, DispatchError> {
    match db_tx {
        Some(tx) => {
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: 1,
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}

/// Dispatch multiple NodeDelete operations to db_tx
///
/// Returns `Ok(DispatchResult)` if db_tx is available and selection is non-empty.
///
/// # Errors
/// Returns `Err(DispatchError::NoSelection)` if node_ids is empty.
/// Returns `Err(DispatchError::WalDisconnected)` if db_tx is None.
pub fn dispatch_node_delete_batch(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_ids: &[String],
) -> Result<DispatchResult, DispatchError> {
    if node_ids.is_empty() {
        return Err(DispatchError::NoSelection);
    }

    match db_tx {
        Some(tx) => {
            for id in node_ids {
                let envelope = create_node_delete_envelope(id.clone());
                tx.send(envelope);
            }
            Ok(DispatchResult {
                nodes_affected: node_ids.len(),
                dispatches_sent: node_ids.len(),
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}

/// Dispatch NodeDelete for a single node
pub fn dispatch_node_delete(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_id: &str,
) -> Result<DispatchResult, DispatchError> {
    dispatch_node_delete_batch(db_tx, &[node_id.to_string()])
}

/// Bounds for a node resize operation
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeBounds {
    /// Node ID being resized
    pub id: NodeId,
    /// Original position and dimensions
    pub original_x: f64,
    pub original_y: f64,
    pub original_width: f64,
    pub original_height: f64,
    /// New position and dimensions
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ResizeBounds {
    /// Create new resize bounds
    #[must_use]
    pub fn new(
        id: NodeId,
        original_x: f64,
        original_y: f64,
        original_width: f64,
        original_height: f64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            id,
            original_x,
            original_y,
            original_width,
            original_height,
            x,
            y,
            width,
            height,
        }
    }
}

/// Dispatch NodeResize operation to db_tx
///
/// Returns `Ok(DispatchResult)` if db_tx is available.
///
/// # Errors
/// Returns `Err(DispatchError::WalDisconnected)` if db_tx is None.
/// Returns `Err(DispatchError::InvalidCoordinates)` if coordinates or dimensions are invalid.
pub fn dispatch_node_resize(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    bounds: ResizeBounds,
) -> Result<DispatchResult, DispatchError> {
    match db_tx {
        Some(tx) => {
            let envelope = create_node_resize_envelope(
                bounds.id,
                bounds.original_x,
                bounds.original_y,
                bounds.original_width,
                bounds.original_height,
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            )?;
            tx.send(envelope);
            Ok(DispatchResult {
                nodes_affected: 1,
                dispatches_sent: 1,
            })
        }
        None => Err(DispatchError::WalDisconnected),
    }
}
