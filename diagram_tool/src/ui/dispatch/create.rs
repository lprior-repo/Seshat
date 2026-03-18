//! Envelope creation functions for dispatch operations

use super::errors::DispatchError;
use super::validators::{validate_coordinates, validate_dimensions};
use diagram_models::document::{EdgeId, EdgeStyle, NodeId, NodeStyle};
use diagram_models::envelope::{Author, DomainOp, EventEnvelope, LabelTargetId, LabelTargetType};

fn wrap(operation: DomainOp) -> EventEnvelope {
    let author = Author {
        id: "local-user".to_string(),
        name: "Local User".to_string(),
        email: None,
    };

    #[cfg(target_arch = "wasm32")]
    let timestamp = js_sys::Date::now() as i64;

    #[cfg(not(target_arch = "wasm32"))]
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation,
        author,
        timestamp,
    }
}

/// Create an `EventEnvelope` for a `NodeAdd` operation
///
/// # Errors
/// Returns `DispatchError::InvalidCoordinates` if x/y are not finite or width/height are not positive
pub fn create_node_add_envelope(
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    label: String,
) -> Result<EventEnvelope, DispatchError> {
    if !validate_coordinates(x, y) || !validate_dimensions(width, height) {
        return Err(DispatchError::InvalidCoordinates);
    }
    Ok(wrap(DomainOp::NodeAdd {
        id: NodeId::new(id),
        x,
        y,
        width,
        height,
        label,
    }))
}

/// Create an `EventEnvelope` for a `NodeDelete` operation
#[must_use]
pub fn create_node_delete_envelope(id: String) -> EventEnvelope {
    wrap(DomainOp::NodeDelete {
        id: NodeId::new(id),
    })
}

/// Create an `EventEnvelope` for a `NodeResize` operation with original bounds
///
/// # Errors
/// Returns `DispatchError::InvalidCoordinates` if coordinates or dimensions are invalid.
#[allow(clippy::too_many_arguments)]
pub fn create_node_resize_envelope(
    id: NodeId,
    original_x: f64,
    original_y: f64,
    original_width: f64,
    original_height: f64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<EventEnvelope, DispatchError> {
    if !validate_coordinates(x, y) || !validate_dimensions(width, height) {
        return Err(DispatchError::InvalidCoordinates);
    }
    Ok(wrap(DomainOp::NodeResize {
        id,
        original_x,
        original_y,
        original_width,
        original_height,
        x,
        y,
        width,
        height,
    }))
}

/// Create an `EventEnvelope` for an `EdgeConnect` operation
#[must_use]
pub fn create_edge_connect_envelope(id: String, source: String, target: String) -> EventEnvelope {
    wrap(DomainOp::EdgeConnect {
        id: EdgeId::new(id),
        source: NodeId::new(source),
        target: NodeId::new(target),
    })
}

/// Create an `EventEnvelope` for an `EdgeDisconnect` operation
#[must_use]
pub fn create_edge_disconnect_envelope(id: String) -> EventEnvelope {
    wrap(DomainOp::EdgeDisconnect {
        id: EdgeId::new(id),
    })
}

fn to_node_ids(ids: Vec<String>) -> Vec<NodeId> {
    ids.into_iter().map(NodeId::new).collect()
}

/// Create an `EventEnvelope` for a Group operation
pub fn create_group_envelope(group_id: String, ids: Vec<String>) -> EventEnvelope {
    wrap(DomainOp::Group {
        id: NodeId::new(group_id),
        ids: to_node_ids(ids),
    })
}

/// Create an `EventEnvelope` for an Ungroup operation
#[must_use]
pub fn create_ungroup_envelope(id: String) -> EventEnvelope {
    wrap(DomainOp::Ungroup {
        id: NodeId::new(id),
    })
}

/// Create an `EventEnvelope` for a `BringForward` operation
pub fn create_bring_forward_envelope(ids: Vec<String>) -> EventEnvelope {
    wrap(DomainOp::BringForward {
        ids: to_node_ids(ids),
    })
}

/// Create an `EventEnvelope` for a `SendBackward` operation
pub fn create_send_backward_envelope(ids: Vec<String>) -> EventEnvelope {
    wrap(DomainOp::SendBackward {
        ids: to_node_ids(ids),
    })
}

/// Create an `EventEnvelope` for a `BringToFront` operation
pub fn create_bring_to_front_envelope(ids: Vec<String>) -> EventEnvelope {
    wrap(DomainOp::BringToFront {
        ids: to_node_ids(ids),
    })
}

/// Create an `EventEnvelope` for a `SendToBack` operation
pub fn create_send_to_back_envelope(ids: Vec<String>) -> EventEnvelope {
    wrap(DomainOp::SendToBack {
        ids: to_node_ids(ids),
    })
}

/// Create an `EventEnvelope` for an `UpdateLabel` operation
#[must_use]
pub fn create_update_label_envelope(
    target_id: LabelTargetId,
    target_type: LabelTargetType,
    old_label: String,
    new_label: String,
) -> EventEnvelope {
    wrap(DomainOp::UpdateLabel {
        target_id,
        target_type,
        old_label,
        new_label,
    })
}

/// Create an `EventEnvelope` for an `UpdateNodeStyle` operation
#[must_use]
pub fn create_update_node_style_envelope(id: String, style: NodeStyle) -> EventEnvelope {
    wrap(DomainOp::UpdateNodeStyle {
        id: NodeId::new(id),
        style,
    })
}

/// Create an `EventEnvelope` for an `UpdateEdgeStyle` operation
#[must_use]
pub fn create_update_edge_style_envelope(id: String, style: EdgeStyle) -> EventEnvelope {
    wrap(DomainOp::UpdateEdgeStyle {
        id: EdgeId::new(id),
        style,
    })
}
