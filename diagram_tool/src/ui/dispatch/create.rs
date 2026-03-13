//! Envelope creation functions for dispatch operations

use super::errors::DispatchError;
use super::validators::{validate_coordinates, validate_dimensions};
use crate::models::document::{EdgeId, EdgeStyle, NodeId, NodeStyle};
use crate::models::envelope::{Author, DomainOp, EventEnvelope, LabelTargetId, LabelTargetType};

/// Local author for dispatched operations
fn local_author() -> Author {
    Author {
        id: "local-user".to_string(),
        name: "Local User".to_string(),
        email: None,
    }
}

/// Get current timestamp in milliseconds since Unix epoch
fn current_timestamp() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}

// =============================================================================
// Node Envelope Creation
// =============================================================================

/// Create an EventEnvelope for a NodeAdd operation
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

    Ok(EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::NodeAdd {
            id: NodeId::new(id),
            x,
            y,
            width,
            height,
            label,
        },
        author: local_author(),
        timestamp: current_timestamp(),
    })
}

/// Create an EventEnvelope for a NodeDelete operation
pub fn create_node_delete_envelope(id: String) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::NodeDelete {
            id: NodeId::new(id),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

/// Create an EventEnvelope for a NodeResize operation with original bounds
///
/// # Errors
/// Returns `DispatchError::InvalidCoordinates` if coordinates or dimensions are invalid.
pub fn create_node_resize_envelope(
    id: crate::models::document::NodeId,
    original_x: f64,
    original_y: f64,
    original_width: f64,
    original_height: f64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<EventEnvelope, DispatchError> {
    if !validate_coordinates(x, y) {
        return Err(DispatchError::InvalidCoordinates);
    }
    if !validate_dimensions(width, height) {
        return Err(DispatchError::InvalidCoordinates);
    }

    Ok(EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::NodeResize {
            id,
            original_x,
            original_y,
            original_width,
            original_height,
            x,
            y,
            width,
            height,
        },
        author: local_author(),
        timestamp: current_timestamp(),
    })
}

// =============================================================================
// Edge Envelope Creation
// =============================================================================

/// Create an EventEnvelope for an EdgeConnect operation
pub fn create_edge_connect_envelope(id: String, source: String, target: String) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::EdgeConnect {
            id: EdgeId::new(id),
            source: NodeId::new(source),
            target: NodeId::new(target),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

/// Create an EventEnvelope for an EdgeDisconnect operation
pub fn create_edge_disconnect_envelope(id: String) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::EdgeDisconnect {
            id: EdgeId::new(id),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

// =============================================================================
// Group Envelope Creation
// =============================================================================

/// Create an EventEnvelope for a Group operation
pub fn create_group_envelope(ids: Vec<String>) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::Group {
            ids: ids.into_iter().map(NodeId::new).collect(),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

/// Create an EventEnvelope for an Ungroup operation
pub fn create_ungroup_envelope(id: String) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::Ungroup {
            id: NodeId::new(id),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

// =============================================================================
// Z-Order Envelope Creation
// =============================================================================

/// Create an EventEnvelope for a BringForward operation
pub fn create_bring_forward_envelope(ids: Vec<String>) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::BringForward {
            ids: ids.into_iter().map(NodeId::new).collect(),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

/// Create an EventEnvelope for a SendBackward operation
pub fn create_send_backward_envelope(ids: Vec<String>) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::SendBackward {
            ids: ids.into_iter().map(NodeId::new).collect(),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

/// Create an EventEnvelope for a BringToFront operation
pub fn create_bring_to_front_envelope(ids: Vec<String>) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::BringToFront {
            ids: ids.into_iter().map(NodeId::new).collect(),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

/// Create an EventEnvelope for a SendToBack operation
pub fn create_send_to_back_envelope(ids: Vec<String>) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::SendToBack {
            ids: ids.into_iter().map(NodeId::new).collect(),
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

// =============================================================================
// Style Envelope Creation
// =============================================================================

/// Create an EventEnvelope for an UpdateLabel operation
pub fn create_update_label_envelope(
    target_id: LabelTargetId,
    target_type: LabelTargetType,
    old_label: String,
    new_label: String,
) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::UpdateLabel {
            target_id,
            target_type,
            old_label,
            new_label,
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

/// Create an EventEnvelope for an UpdateNodeStyle operation
pub fn create_update_node_style_envelope(id: String, style: NodeStyle) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::UpdateNodeStyle {
            id: NodeId::new(id),
            style,
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}

/// Create an EventEnvelope for an UpdateEdgeStyle operation
pub fn create_update_edge_style_envelope(id: String, style: EdgeStyle) -> EventEnvelope {
    EventEnvelope {
        op_id: uuid::Uuid::new_v4().to_string(),
        operation: DomainOp::UpdateEdgeStyle {
            id: EdgeId::new(id),
            style,
        },
        author: local_author(),
        timestamp: current_timestamp(),
    }
}
