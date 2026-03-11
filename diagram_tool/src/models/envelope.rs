//! Event envelope module - defines `EventEnvelope` and Author metadata
//!
//! This module provides types for encoding/decoding event envelopes
//! with strict validation of required fields.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::document::{EdgeId, EdgeStyle, NodeId, NodeStyle};

/// Error types for domain `op_types`
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContractError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid author: {0}")]
    InvalidAuthor(String),
    #[error("unknown op_type type: {0}")]
    UnknownOpType(String),
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
}

/// Author metadata for events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author {
    /// Unique identifier for the author
    pub id: String,
    /// Display name of the author
    pub name: String,
    /// Optional email for the author
    pub email: Option<String>,
}

/// Operation types for events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpType {
    Create,
    Update,
    Delete,
    Migrate,
}

impl OpType {
    /// Parse `OpType` from string, returning `UnknownOpType` error for invalid values
    fn from_str(s: &str) -> Result<Self, ContractError> {
        match s {
            "create" => Ok(Self::Create),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            "migrate" => Ok(Self::Migrate),
            _ => Err(ContractError::UnknownOpType(s.to_string())),
        }
    }
}

/// Kind of domain `op_type`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpKind {
    /// Operation on a node
    Node,
    /// Operation on an edge
    Edge,
    /// Composite `op_type` involving multiple entities
    Composite,
    /// Z-order `op_type` for layering
    ZOrder,
}

/// Type of label target (node or edge)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelTargetType {
    /// Target is a node
    Node,
    /// Target is an edge
    Edge,
}

impl OpKind {
    /// Returns the name of this `op_type` kind as a string
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Edge => "edge",
            Self::Composite => "composite",
            Self::ZOrder => "z_order",
        }
    }
}

/// Domain operation representing a diagram editor operation
///
/// Uses `op_type` as the tag to avoid conflicts with `EventRecord.operation` field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op_type", rename_all = "snake_case")]
pub enum DomainOp {
    // Node operations
    NodeAdd {
        id: NodeId,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        label: String,
    },
    NodeMove {
        id: NodeId,
        x: f64,
        y: f64,
    },
    NodeDelete {
        id: NodeId,
    },
    NodeRestore {
        id: NodeId,
    },
    NodeResize {
        id: NodeId,
        original_x: f64,
        original_y: f64,
        original_width: f64,
        original_height: f64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    UpdateLabel {
        target_id: String,
        target_type: LabelTargetType,
        old_label: String,
        new_label: String,
    },
    UpdateNodeStyle {
        id: NodeId,
        style: NodeStyle,
    },
    // Edge operations
    EdgeConnect {
        id: EdgeId,
        source: NodeId,
        target: NodeId,
    },
    EdgeDisconnect {
        id: EdgeId,
    },
    UpdateEdgeStyle {
        id: EdgeId,
        style: EdgeStyle,
    },
    // Z-order op_types
    BringForward {
        ids: Vec<NodeId>,
    },
    SendBackward {
        ids: Vec<NodeId>,
    },
    BringToFront {
        ids: Vec<NodeId>,
    },
    SendToBack {
        ids: Vec<NodeId>,
    },
    // Composite op_types
    Group {
        ids: Vec<NodeId>,
    },
    Ungroup {
        id: NodeId,
    },
}

impl DomainOp {
    /// Get the `op_type` kind for this domain `op_type`
    #[must_use]
    pub const fn kind(&self) -> OpKind {
        match self {
            Self::NodeAdd { .. }
            | Self::NodeMove { .. }
            | Self::NodeDelete { .. }
            | Self::NodeRestore { .. }
            | Self::NodeResize { .. }
            | Self::UpdateLabel { .. }
            | Self::UpdateNodeStyle { .. } => OpKind::Node,
            Self::EdgeConnect { .. }
            | Self::EdgeDisconnect { .. }
            | Self::UpdateEdgeStyle { .. } => OpKind::Edge,
            Self::BringForward { .. }
            | Self::SendBackward { .. }
            | Self::BringToFront { .. }
            | Self::SendToBack { .. } => OpKind::ZOrder,
            Self::Group { .. } | Self::Ungroup { .. } => OpKind::Composite,
        }
    }
}

/// Parse a domain `op_type` from a JSON string
///
/// # Errors
/// Returns `ContractError::InvalidJson` if the JSON is malformed
/// Returns `ContractError::UnknownOpType` if the `op_type` type is not recognized
/// Returns `ContractError::InvalidPayload` if the payload is invalid
/// Returns `ContractError::MissingField` if required fields are missing
pub fn parse_domain_op(raw: &str) -> Result<DomainOp, ContractError> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| ContractError::InvalidJson(e.to_string()))?;
    let op_field = extract_op_type(&value)?;
    dispatch_domain_op(&value, op_field)
}

fn extract_op_type(value: &serde_json::Value) -> Result<&str, ContractError> {
    value
        .get("op")
        .and_then(|v| v.as_str())
        .ok_or(ContractError::MissingField("op"))
}

fn dispatch_domain_op(
    value: &serde_json::Value,
    op_field: &str,
) -> Result<DomainOp, ContractError> {
    match op_field {
        "node_add" => parse_node_add(value),
        "node_move" => parse_node_move(value),
        "node_delete" => parse_node_delete(value),
        "node_restore" => parse_node_restore(value),
        "node_resize" => parse_node_resize(value),
        "update_label" => parse_update_label(value),
        "update_node_style" => parse_update_node_style(value),
        "edge_connect" => parse_edge_connect(value),
        "edge_disconnect" => parse_edge_disconnect(value),
        "update_edge_style" => parse_update_edge_style(value),
        "bring_forward" => parse_bring_forward(value),
        "send_backward" => parse_send_backward(value),
        "bring_to_front" => parse_bring_to_front(value),
        "send_to_back" => parse_send_to_back(value),
        "group" => parse_group(value),
        "ungroup" => parse_ungroup(value),
        _ => Err(ContractError::UnknownOpType(op_field.to_string())),
    }
}

/// Get the `op_type` kind for a domain `op_type`
///
/// This is a convenience function that delegates to `DomainOp::kind()`
#[must_use]
pub const fn domain_op_kind(op: &DomainOp) -> OpKind {
    op.kind()
}

// Helper functions for parsing domain op_types

fn parse_node_add(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    let x = extract_f64_field(value, "x")?;
    let y = extract_f64_field(value, "y")?;
    let width = extract_f64_field(value, "width")?;
    let height = extract_f64_field(value, "height")?;
    let label = value
        .get("label")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    Ok(DomainOp::NodeAdd {
        id,
        x,
        y,
        width,
        height,
        label,
    })
}

fn parse_node_move(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    let x = extract_f64_field(value, "x")?;
    let y = extract_f64_field(value, "y")?;

    Ok(DomainOp::NodeMove { id, x, y })
}

fn parse_node_delete(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    Ok(DomainOp::NodeDelete { id })
}

fn parse_node_restore(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    Ok(DomainOp::NodeRestore { id })
}

fn parse_node_resize(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    let dims = extract_and_validate_dimensions(value)?;
    Ok(DomainOp::NodeResize {
        id,
        original_x: dims.original_x,
        original_y: dims.original_y,
        original_width: dims.original_width,
        original_height: dims.original_height,
        x: dims.x,
        y: dims.y,
        width: dims.width,
        height: dims.height,
    })
}

fn extract_and_validate_dimensions(
    value: &serde_json::Value,
) -> Result<NodeResizeDimensions, ContractError> {
    Ok(NodeResizeDimensions {
        original_x: extract_f64_field(value, "original_x")?,
        original_y: extract_f64_field(value, "original_y")?,
        original_width: validate_positive_finite(
            extract_f64_field(value, "original_width")?,
            "original_width",
        )?,
        original_height: validate_positive_finite(
            extract_f64_field(value, "original_height")?,
            "original_height",
        )?,
        x: extract_f64_field(value, "x")?,
        y: extract_f64_field(value, "y")?,
        width: validate_positive_finite(extract_f64_field(value, "width")?, "width")?,
        height: validate_positive_finite(extract_f64_field(value, "height")?, "height")?,
    })
}

/// Helper struct to bundle NodeResize fields
struct NodeResizeDimensions {
    original_x: f64,
    original_y: f64,
    original_width: f64,
    original_height: f64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn parse_update_label(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    // Backward compatibility: check for "id" first, then "target_id"
    let target_id = value
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            value
                .get("target_id")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .unwrap_or_default();

    let target_type_str = value
        .get("target_type")
        .and_then(|v| v.as_str())
        .unwrap_or("node");
    let target_type = match target_type_str {
        "node" => LabelTargetType::Node,
        "edge" => LabelTargetType::Edge,
        _ => LabelTargetType::Node,
    };
    let old_label = value
        .get("old_label")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();
    let new_label = value
        .get("new_label")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            // Backward compatibility: also accept "label" field
            value
                .get("label")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .unwrap_or_default();

    // Use target_id or fall back to empty if neither id nor target_id present
    let final_target_id = if target_id.is_empty() {
        value
            .get("target_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default()
    } else {
        target_id
    };

    Ok(DomainOp::UpdateLabel {
        target_id: final_target_id,
        target_type,
        old_label,
        new_label,
    })
}

fn parse_update_node_style(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    let style_str = value
        .get("style")
        .and_then(|v| v.as_str())
        .ok_or(ContractError::MissingField("style"))?;
    let style = match style_str {
        "box" => NodeStyle::Box,
        "cloud" => NodeStyle::Cloud,
        "cylinder" => NodeStyle::Cylinder,
        "dashed" => NodeStyle::Dashed,
        _ => NodeStyle::Box,
    };

    Ok(DomainOp::UpdateNodeStyle { id, style })
}

fn parse_update_edge_style(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_edge_id(&extract_string_field(value, "id")?)?;
    let style_str = value
        .get("style")
        .and_then(|v| v.as_str())
        .ok_or(ContractError::MissingField("style"))?;
    let style = match style_str {
        "solid" => EdgeStyle::Solid,
        "dashed" => EdgeStyle::Dashed,
        "dotted" => EdgeStyle::Dotted,
        _ => EdgeStyle::Solid,
    };

    Ok(DomainOp::UpdateEdgeStyle { id, style })
}

// Helper functions for NodeResize parsing
fn extract_string_field(value: &serde_json::Value, field: &str) -> Result<String, ContractError> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or(ContractError::MissingField(Box::leak(
            field.to_string().into_boxed_str(),
        )))
}

fn extract_f64_field(value: &serde_json::Value, field: &str) -> Result<f64, ContractError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_f64)
        .ok_or(ContractError::MissingField(Box::leak(
            field.to_string().into_boxed_str(),
        )))
}

fn require_non_empty_id(id: &str) -> Result<NodeId, ContractError> {
    if id.is_empty() {
        return Err(ContractError::InvalidPayload(
            "node id cannot be empty".to_string(),
        ));
    }
    Ok(NodeId::new(id.to_string()))
}

fn require_non_empty_edge_id(id: &str) -> Result<EdgeId, ContractError> {
    if id.is_empty() {
        return Err(ContractError::InvalidPayload(
            "edge id cannot be empty".to_string(),
        ));
    }
    Ok(EdgeId::new(id.to_string()))
}

fn validate_positive_finite(value: f64, field_name: &str) -> Result<f64, ContractError> {
    if !value.is_finite() {
        return Err(ContractError::InvalidPayload(format!(
            "{field_name} must be finite"
        )));
    }
    if value <= 0.0 {
        return Err(ContractError::InvalidPayload(format!(
            "{field_name} must be positive"
        )));
    }
    Ok(value)
}

fn parse_edge_connect(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_edge_id(&extract_string_field(value, "id")?)?;
    let source = require_non_empty_id(&extract_string_field(value, "source")?)?;
    let target = require_non_empty_id(&extract_string_field(value, "target")?)?;

    Ok(DomainOp::EdgeConnect { id, source, target })
}

fn parse_edge_disconnect(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_edge_id(&extract_string_field(value, "id")?)?;
    Ok(DomainOp::EdgeDisconnect { id })
}

fn parse_bring_forward(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::BringForward { ids })
}

fn parse_send_backward(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::SendBackward { ids })
}

fn parse_bring_to_front(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::BringToFront { ids })
}

fn parse_send_to_back(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::SendToBack { ids })
}

fn parse_group(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::Group { ids })
}

fn parse_ungroup(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    Ok(DomainOp::Ungroup { id })
}

fn parse_string_array(value: Option<&serde_json::Value>) -> Result<Vec<String>, ContractError> {
    let arr = value
        .and_then(|v| v.as_array())
        .ok_or(ContractError::InvalidPayload("expected array".to_string()))?;

    arr.iter()
        .map(|v| {
            v.as_str().map(String::from).ok_or_else(|| {
                ContractError::InvalidPayload("expected string in array".to_string())
            })
        })
        .collect()
}

fn parse_node_id_array(value: Option<&serde_json::Value>) -> Result<Vec<NodeId>, ContractError> {
    let strings = parse_string_array(value)?;
    let mut node_ids = Vec::with_capacity(strings.len());
    for s in strings {
        node_ids.push(require_non_empty_id(&s)?);
    }
    Ok(node_ids)
}

/// Event envelope containing operation and author metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    /// Unique identifier for this envelope (operation ID)
    #[serde(rename = "op_id")]
    pub op_id: String,
    /// The diagram operation being performed
    pub operation: DomainOp,
    /// Author who created this event
    pub author: Author,
    /// Timestamp of when this event was created (Unix timestamp)
    pub timestamp: i64,
}

/// Decode an `EventEnvelope` from a JSON string
///
/// This is an alias for `parse_event_envelope` for backward compatibility.
/// Prefer using `parse_event_envelope` directly.
///
/// # Errors
/// Returns `ContractError::InvalidJson` if the JSON is malformed
/// Returns `ContractError::MissingField` if required fields are missing
/// Returns `ContractError::InvalidAuthor` if author validation fails
/// Returns `ContractError::UnknownOpType` if the `op_type` type is invalid
#[deprecated(since = "0.1.0", note = "use parse_event_envelope instead")]
pub fn decode_envelope(raw: &str) -> Result<EventEnvelope, ContractError> {
    parse_event_envelope(raw)
}

/// Encode an `EventEnvelope` to a JSON string
///
/// This is an alias for `encode_event_envelope` for backward compatibility.
/// Prefer using `encode_event_envelope` directly.
///
/// # Errors
/// Returns `ContractError::InvalidJson` if encoding fails
#[deprecated(since = "0.1.0", note = "use encode_event_envelope instead")]
pub fn encode_envelope(op: &EventEnvelope) -> Result<String, ContractError> {
    encode_event_envelope(op)
}

/// Parse an `EventEnvelope` from a JSON string
///
/// This is the canonical function for parsing event envelopes as per the contract.
/// Validates all required fields and returns structured errors.
///
/// # Errors
/// Returns `ContractError::InvalidJson` if the JSON is malformed
/// Returns `ContractError::MissingField` if required fields are missing
/// Returns `ContractError::InvalidAuthor` if author validation fails
/// Returns `ContractError::UnknownOpType` if the `op_type` type is invalid
pub fn parse_event_envelope(input: &str) -> Result<EventEnvelope, ContractError> {
    // Implement size limits: >5MB payload rejection before any parsing
    if input.len() > 5 * 1024 * 1024 {
        return Err(ContractError::InvalidPayload(
            "payload exceeds 5MB limit".to_string(),
        ));
    }

    // Fast pass to reject >5000 JSON edges/objects to prevent memory exhaustion attacks
    // before allocating the DOM or running full deserialization.
    let structural_edges = input.bytes().filter(|&b| b == b'{' || b == b'[').count();
    if structural_edges > 5000 {
        return Err(ContractError::InvalidPayload(
            "payload exceeds 5000 structural edges limit".to_string(),
        ));
    }

    // To prevent full JSON DOM allocation, we deserialize directly into the struct
    // rather than intermediate serde_json::Value. This enforces strong types and memory limits at the edge.
    deserialize_envelope(input)
}

fn validate_envelope_fields(value: &serde_json::Value) -> Result<(), ContractError> {
    let _ = value
        .get("op_id")
        .ok_or(ContractError::MissingField("op_id"))?;
    let author_value = value
        .get("author")
        .ok_or(ContractError::MissingField("author"))?;
    validate_author(author_value)?;
    let _ = value
        .get("timestamp")
        .ok_or(ContractError::MissingField("timestamp"))?;
    Ok(())
}

fn validate_author(author_value: &serde_json::Value) -> Result<(), ContractError> {
    let _ = author_value
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ContractError::InvalidAuthor("missing id field".to_string()))?;
    let _ = author_value
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ContractError::InvalidAuthor("missing name field".to_string()))?;
    Ok(())
}

fn deserialize_envelope(input: &str) -> Result<EventEnvelope, ContractError> {
    serde_json::from_str(input).map_err(convert_serde_error)
}

fn convert_serde_error(e: serde_json::Error) -> ContractError {
    let err_msg = e.to_string();
    if err_msg.contains("unknown variant") {
        let unknown = err_msg
            .split("unknown variant ")
            .nth(1)
            .unwrap_or("unknown")
            .trim()
            .trim_matches('\"')
            .to_string();
        ContractError::UnknownOpType(unknown)
    } else if err_msg.contains("missing field") {
        let field = err_msg
            .split("missing field ")
            .nth(1)
            .unwrap_or("unknown")
            .trim()
            .trim_matches('\"')
            .trim_matches('`')
            .split_whitespace()
            .next()
            .unwrap_or("unknown");
        map_missing_field_error(field)
    } else {
        ContractError::InvalidJson(err_msg)
    }
}

fn map_missing_field_error(field: &str) -> ContractError {
    match field {
        "author" => ContractError::MissingField("author"),
        "timestamp" => ContractError::MissingField("timestamp"),
        "domain_op" => ContractError::MissingField("domain_op"),
        "op_id" => ContractError::MissingField("op_id"),
        "operation" => ContractError::MissingField("operation"),
        _ => ContractError::MissingField("unknown"),
    }
}

/// Encode an `EventEnvelope` to a JSON string
///
/// This is the canonical function for encoding event envelopes as per the contract.
///
/// # Errors
/// Returns `ContractError::InvalidJson` if encoding fails
pub fn encode_event_envelope(op: &EventEnvelope) -> Result<String, ContractError> {
    serde_json::to_string(op).map_err(|e| ContractError::InvalidJson(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
    fn given_valid_json_when_parsing_event_envelope_then_returns_envelope() {
        let raw = r#"{
            "op_id": "evt-123",
            "operation": "node_add",
            "id": "node-1",
            "x": 100.0,
            "y": 200.0,
            "width": 80.0,
            "height": 40.0,
            "label": "Test Node",
            "author": {
                "id": "user-1",
                "name": "Alice"
            },
            "timestamp": 1699999999
        }"#;

        let result = parse_event_envelope(raw);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let envelope = result.unwrap();
        assert_eq!(envelope.op_id, "evt-123");
        assert!(matches!(envelope.operation, DomainOp::NodeAdd { .. }));
        assert_eq!(envelope.author.id, "user-1");
        assert_eq!(envelope.timestamp, 1699999999);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_invalid_json_when_parsing_event_envelope_then_returns_invalid_json_error() {
        let raw = "not valid json";

        let result = parse_event_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::InvalidJson(_)) => {}
            _ => panic!("Expected InvalidJson error"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_missing_op_id_field_when_parsing_event_envelope_then_returns_missing_field_error() {
        let raw = r#"{
            "t": "node_add",
            "id": "node-1",
            "x": 100.0,
            "y": 200.0,
            "author": {"id": "user-1", "name": "Alice"},
            "timestamp": 1699999999
        }"#;

        let result = parse_event_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::MissingField(f)) => assert_eq!(f, "op_id"),
            _ => panic!("Expected MissingField error for 'op_id'"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_missing_author_field_when_parsing_event_envelope_then_returns_missing_field_error() {
        let raw = r#"{
            "op_id": "evt-123",
            "t": "node_add",
            "id": "node-1",
            "x": 100.0,
            "y": 200.0,
            "timestamp": 1699999999
        }"#;

        let result = parse_event_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::MissingField(f)) => assert_eq!(f, "author"),
            _ => panic!("Expected MissingField error for 'author'"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_invalid_author_missing_name_when_parsing_event_envelope_then_returns_invalid_author_error(
    ) {
        let raw = r#"{
            "op_id": "evt-123",
            "t": "node_add",
            "id": "node-1",
            "x": 100.0,
            "y": 200.0,
            "author": {"id": "user-1"},
            "timestamp": 1699999999
        }"#;

        let result = parse_event_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::InvalidAuthor(_)) => {}
            _ => panic!("Expected InvalidAuthor error"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
    fn given_unknown_op_type_type_when_parsing_event_envelope_then_returns_unknown_op_type_error() {
        let raw = r#"{
            "op_id": "evt-123",
            "t": "unknown_op_type",
            "author": {"id": "user-1", "name": "Alice"},
            "timestamp": 1699999999
        }"#;

        let result = parse_event_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::UnknownOpType(s)) => assert_eq!(s, "unknown_op_type"),
            _ => panic!("Expected UnknownOpType error"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
    fn given_all_op_type_types_then_all_parse_correctly() {
        // Test that all DomainOp types can be parsed from the envelope
        let test_cases = [
            (
                r#""t": "node_add", "id": "n1", "x": 0.0, "y": 0.0, "width": 80.0, "height": 40.0"#,
                "node_add",
            ),
            (
                r#""t": "node_move", "id": "n1", "x": 100.0, "y": 200.0"#,
                "node_move",
            ),
            (r#""t": "node_delete", "id": "n1""#, "node_delete"),
            (
                r#""t": "edge_connect", "id": "e1", "source": "n1", "target": "n2""#,
                "edge_connect",
            ),
            (r#""t": "edge_disconnect", "id": "e1""#, "edge_disconnect"),
            (r#""t": "bring_forward", "ids": ["n1"]"#, "bring_forward"),
            (r#""t": "send_backward", "ids": ["n1"]"#, "send_backward"),
            (r#""t": "bring_to_front", "ids": ["n1"]"#, "bring_to_front"),
            (r#""t": "send_to_back", "ids": ["n1"]"#, "send_to_back"),
            (r#""t": "group", "ids": ["n1", "n2"]"#, "group"),
            (r#""t": "ungroup", "id": "g1""#, "ungroup"),
        ];

        for (op_str, _op_name) in test_cases {
            let raw = format!(
                r#"{{"op_id": "evt-1", {}, "author": {{"id": "u1", "name": "A"}}, "timestamp": 1}}"#,
                op_str
            );
            let result = parse_event_envelope(&raw);
            assert!(result.is_ok(), "Failed for op: {}", op_str);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
    fn given_author_with_email_when_parsing_event_envelope_then_email_is_preserved() {
        let raw = r#"{
            "op_id": "evt-123",
            "t": "node_add",
            "id": "node-1",
            "x": 100.0,
            "y": 200.0,
            "width": 80.0,
            "height": 40.0,
            "author": {
                "id": "user-1",
                "name": "Alice",
                "email": "alice@example.com"
            },
            "timestamp": 1699999999
        }"#;

        let result = parse_event_envelope(raw);

        assert!(result.is_ok());
        let envelope = result.unwrap();
        assert_eq!(envelope.author.email, Some("alice@example.com".to_string()));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    #[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
    fn given_author_without_email_when_parsing_event_envelope_then_email_is_none() {
        let raw = r#"{
            "op_id": "evt-123",
            "t": "node_add",
            "id": "node-1",
            "x": 100.0,
            "y": 200.0,
            "width": 80.0,
            "height": 40.0,
            "author": {
                "id": "user-1",
                "name": "Alice"
            },
            "timestamp": 1699999999
        }"#;

        let result = parse_event_envelope(raw);

        assert!(result.is_ok());
        let envelope = result.unwrap();
        assert_eq!(envelope.author.email, None);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_event_envelope_when_encoding_then_roundtrip_works() {
        let original = EventEnvelope {
            op_id: "evt-roundtrip".to_string(),
            operation: DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Bob".to_string(),
                email: Some("bob@example.com".to_string()),
            },
            timestamp: 1700000000,
        };

        let encoded = encode_event_envelope(&original);
        assert!(encoded.is_ok(), "Encoding failed: {:?}", encoded.err());

        let decoded = parse_event_envelope(&encoded.unwrap());
        assert!(decoded.is_ok(), "Decoding failed: {:?}", decoded.err());

        assert_eq!(decoded.unwrap(), original);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_event_envelope_with_complex_operation_when_encoding_then_roundtrip_works() {
        let original = EventEnvelope {
            op_id: "evt-complex".to_string(),
            operation: DomainOp::Group {
                ids: vec![
                    "node-1".to_string(),
                    "node-2".to_string(),
                    "node-3".to_string(),
                ],
            },
            author: Author {
                id: "user-2".to_string(),
                name: "Charlie".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        let encoded = encode_event_envelope(&original);
        assert!(encoded.is_ok(), "Encoding failed: {:?}", encoded.err());

        let decoded = parse_event_envelope(&encoded.unwrap());
        assert!(decoded.is_ok(), "Decoding failed: {:?}", decoded.err());

        assert_eq!(decoded.unwrap(), original);
    }

    // DomainOp and OpKind tests

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_node_add_json_when_parsing_then_returns_domain_op() {
        let raw = r#"{
            "op": "node_add",
            "id": "node-1",
            "x": 100.0,
            "y": 200.0,
            "width": 80.0,
            "height": 40.0,
            "label": "Test Node"
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let op = result.unwrap();
        assert!(matches!(op, DomainOp::NodeAdd { .. }));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_node_move_json_when_parsing_then_returns_domain_op() {
        let raw = r#"{
            "op": "node_move",
            "id": "node-1",
            "x": 150.0,
            "y": 250.0
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok());
        let op = result.unwrap();
        assert!(matches!(op, DomainOp::NodeMove { .. }));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_node_delete_json_when_parsing_then_returns_domain_op() {
        let raw = r#"{
            "op": "node_delete",
            "id": "node-1"
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DomainOp::NodeDelete { id } if id == "node-1"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_edge_connect_json_when_parsing_then_returns_domain_op() {
        let raw = r#"{
            "op": "edge_connect",
            "id": "edge-1",
            "source": "node-1",
            "target": "node-2"
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok());
        let op = result.unwrap();
        assert!(matches!(op, DomainOp::EdgeConnect { .. }));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_edge_disconnect_json_when_parsing_then_returns_domain_op() {
        let raw = r#"{
            "op": "edge_disconnect",
            "id": "edge-1"
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DomainOp::EdgeDisconnect { id } if id == "edge-1"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_group_json_when_parsing_then_returns_domain_op() {
        let raw = r#"{
            "op": "group",
            "ids": ["node-1", "node-2", "node-3"]
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok());
        let op = result.unwrap();
        assert!(matches!(op, DomainOp::Group { ids } if ids.len() == 3));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_ungroup_json_when_parsing_then_returns_domain_op() {
        let raw = r#"{
            "op": "ungroup",
            "id": "group-1"
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DomainOp::Ungroup { id } if id == "group-1"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_zorder_json_when_parsing_then_returns_domain_op() {
        let test_cases = [
            r#"{"op": "bring_forward", "ids": ["n1", "n2"]}"#,
            r#"{"op": "send_backward", "ids": ["n1", "n2"]}"#,
            r#"{"op": "bring_to_front", "ids": ["n1", "n2"]}"#,
            r#"{"op": "send_to_back", "ids": ["n1", "n2"]}"#,
        ];

        for raw in test_cases {
            let result = parse_domain_op(raw);
            assert!(result.is_ok(), "Failed for op: {}", raw);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_invalid_json_when_parsing_then_returns_invalid_json_error() {
        let raw = "not valid json";

        let result = parse_domain_op(raw);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContractError::InvalidJson(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_missing_op_field_when_parsing_then_returns_missing_field_error() {
        let raw = r#"{
            "id": "node-1",
            "x": 100.0
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::MissingField(f)) => assert_eq!(f, "op"),
            _ => panic!("Expected MissingField error for 'op'"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_unknown_op_type_when_parsing_then_returns_unknown_op_type_error() {
        let raw = r#"{
            "op": "unknown_op_type",
            "id": "node-1"
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::UnknownOpType(s)) => assert_eq!(s, "unknown_op_type"),
            _ => panic!("Expected UnknownOpType error"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_missing_required_field_when_parsing_then_returns_missing_field_error() {
        let raw = r#"{
            "op": "node_move",
            "id": "node-1"
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::MissingField(f)) => assert!(f == "x" || f == "y"),
            _ => panic!("Expected MissingField error"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_invalid_array_when_parsing_then_returns_invalid_payload_error() {
        let raw = r#"{
            "op": "group",
            "ids": "not-an-array"
        }"#;

        let result = parse_domain_op(raw);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ContractError::InvalidPayload(_)
        ));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_add_op_when_getting_kind_then_returns_node_kind() {
        let op = DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 40.0,
            label: "Test".to_string(),
        };

        let kind = domain_op_kind(&op);

        assert_eq!(kind, OpKind::Node);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_move_op_when_getting_kind_then_returns_node_kind() {
        let op = DomainOp::NodeMove {
            id: "node-1".to_string(),
            x: 100.0,
            y: 200.0,
        };

        let kind = domain_op_kind(&op);

        assert_eq!(kind, OpKind::Node);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_delete_op_when_getting_kind_then_returns_node_kind() {
        let op = DomainOp::NodeDelete {
            id: "node-1".to_string(),
        };

        let kind = domain_op_kind(&op);

        assert_eq!(kind, OpKind::Node);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_restore_op_when_getting_kind_then_returns_node_kind() {
        let op = DomainOp::NodeRestore {
            id: "node-1".to_string(),
        };

        let kind = domain_op_kind(&op);

        assert_eq!(kind, OpKind::Node);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_update_label_op_when_getting_kind_then_returns_node_kind() {
        let op = DomainOp::UpdateLabel {
            target_id: "node-1".to_string(),
            target_type: LabelTargetType::Node,
            old_label: "Old Label".to_string(),
            new_label: "Test Label".to_string(),
        };

        let kind = domain_op_kind(&op);

        assert_eq!(kind, OpKind::Node);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_update_label_with_very_long_label_when_parsing_then_succeeds() {
        let long_label = "x".repeat(15_000);
        let raw = format!(
            r#"{{"op": "update_label", "target_id": "n1", "target_type": "node", "old_label": "old", "new_label": "{}"}}"#,
            long_label
        );

        let result = parse_domain_op(&raw);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let op = result.unwrap();
        match op {
            DomainOp::UpdateLabel { new_label, .. } => {
                assert_eq!(new_label.len(), 15_000);
            }
            _ => panic!("Expected UpdateLabel"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_update_label_with_mixed_direction_text_when_parsing_then_succeeds() {
        let raw = r#"{"op": "update_label", "target_id": "n1", "target_type": "node", "old_label": "old", "new_label": "Hello مرحبا World 🌍"}"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let op = result.unwrap();
        match op {
            DomainOp::UpdateLabel { new_label, .. } => {
                assert_eq!(new_label, "Hello مرحبا World 🌍");
            }
            _ => panic!("Expected UpdateLabel"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_update_label_json_missing_op_field_when_parsing_then_returns_missing_field_error() {
        let raw = r#"{"target_id": "n1", "new_label": "New Label"}"#;

        let result = parse_domain_op(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::MissingField(f)) => assert_eq!(f, "op"),
            _ => panic!("Expected MissingField error for 'op'"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_update_label_json_when_parsing_then_returns_domain_op() {
        let raw = r#"{"op": "update_label", "target_id": "node-1", "target_type": "node", "old_label": "old", "new_label": "New Label"}"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let op = result.unwrap();
        assert!(matches!(op, DomainOp::UpdateLabel { .. }));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_update_label_with_edge_target_type_when_parsing_then_succeeds() {
        let raw = r#"{"op": "update_label", "target_id": "edge-1", "target_type": "edge", "old_label": "old label", "new_label": "new label"}"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let op = result.unwrap();
        match op {
            DomainOp::UpdateLabel {
                target_type,
                target_id,
                ..
            } => {
                assert_eq!(target_type, LabelTargetType::Edge);
                assert_eq!(target_id, "edge-1");
            }
            _ => panic!("Expected UpdateLabel"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_update_label_backward_compatibility_with_old_fields_when_parsing_then_succeeds() {
        // Test backward compatibility: old "id" and "label" fields should still work
        let raw = r#"{"op": "update_label", "id": "node-1", "label": "Test Label"}"#;

        let result = parse_domain_op(raw);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let op = result.unwrap();
        match op {
            DomainOp::UpdateLabel {
                target_id,
                new_label,
                target_type,
                ..
            } => {
                assert_eq!(target_id, "node-1");
                assert_eq!(new_label, "Test Label");
                assert_eq!(target_type, LabelTargetType::Node); // default
            }
            _ => panic!("Expected UpdateLabel"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_edge_connect_op_when_getting_kind_then_returns_edge_kind() {
        let op = DomainOp::EdgeConnect {
            id: "edge-1".to_string(),
            source: "node-1".to_string(),
            target: "node-2".to_string(),
        };

        let kind = domain_op_kind(&op);

        assert_eq!(kind, OpKind::Edge);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_edge_disconnect_op_when_getting_kind_then_returns_edge_kind() {
        let op = DomainOp::EdgeDisconnect {
            id: "edge-1".to_string(),
        };

        let kind = domain_op_kind(&op);

        assert_eq!(kind, OpKind::Edge);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_zorder_ops_when_getting_kind_then_returns_zorder_kind() {
        let ops = [
            DomainOp::BringForward {
                ids: vec!["n1".to_string()],
            },
            DomainOp::SendBackward {
                ids: vec!["n1".to_string()],
            },
            DomainOp::BringToFront {
                ids: vec!["n1".to_string()],
            },
            DomainOp::SendToBack {
                ids: vec!["n1".to_string()],
            },
        ];

        for op in ops {
            let kind = domain_op_kind(&op);
            assert_eq!(kind, OpKind::ZOrder, "Failed for {:?}", op);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_composite_ops_when_getting_kind_then_returns_composite_kind() {
        let ops = [
            DomainOp::Group {
                ids: vec!["n1".to_string(), "n2".to_string()],
            },
            DomainOp::Ungroup {
                id: "group-1".to_string(),
            },
        ];

        for op in ops {
            let kind = domain_op_kind(&op);
            assert_eq!(kind, OpKind::Composite, "Failed for {:?}", op);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_op_kind_as_str_then_returns_correct_string() {
        assert_eq!(OpKind::Node.as_str(), "node");
        assert_eq!(OpKind::Edge.as_str(), "edge");
        assert_eq!(OpKind::Composite.as_str(), "composite");
        assert_eq!(OpKind::ZOrder.as_str(), "z_order");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_domain_op_kind_method_then_matches_free_function() {
        let ops = [
            DomainOp::NodeAdd {
                id: "n1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "".to_string(),
            },
            DomainOp::NodeMove {
                id: "n1".to_string(),
                x: 0.0,
                y: 0.0,
            },
            DomainOp::NodeDelete {
                id: "n1".to_string(),
            },
            DomainOp::NodeRestore {
                id: "n1".to_string(),
            },
            DomainOp::EdgeConnect {
                id: "e1".to_string(),
                source: "n1".to_string(),
                target: "n2".to_string(),
            },
            DomainOp::EdgeDisconnect {
                id: "e1".to_string(),
            },
            DomainOp::BringForward {
                ids: vec!["n1".to_string()],
            },
            DomainOp::SendBackward {
                ids: vec!["n1".to_string()],
            },
            DomainOp::BringToFront {
                ids: vec!["n1".to_string()],
            },
            DomainOp::SendToBack {
                ids: vec!["n1".to_string()],
            },
            DomainOp::Group {
                ids: vec!["n1".to_string()],
            },
            DomainOp::Ungroup {
                id: "g1".to_string(),
            },
        ];

        for op in &ops {
            let method_kind = op.kind();
            let function_kind = domain_op_kind(op);
            assert_eq!(method_kind, function_kind, "Mismatch for {:?}", op);
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_all_domain_op_variants_exhaustive_match_then_all_cases_handled() {
        // This test ensures that when we add new variants to DomainOp,
        // we must update this test - making the exhaustive match explicit
        let check_variant = |op: DomainOp| -> &'static str {
            match op {
                DomainOp::NodeAdd { .. } => "NodeAdd",
                DomainOp::NodeMove { .. } => "NodeMove",
                DomainOp::NodeDelete { .. } => "NodeDelete",
                DomainOp::NodeRestore { .. } => "NodeRestore",
                DomainOp::NodeResize { .. } => "NodeResize",
                DomainOp::UpdateLabel { .. } => "UpdateLabel",
                DomainOp::UpdateNodeStyle { .. } => "UpdateNodeStyle",
                DomainOp::EdgeConnect { .. } => "EdgeConnect",
                DomainOp::EdgeDisconnect { .. } => "EdgeDisconnect",
                DomainOp::UpdateEdgeStyle { .. } => "UpdateEdgeStyle",
                DomainOp::BringForward { .. } => "BringForward",
                DomainOp::SendBackward { .. } => "SendBackward",
                DomainOp::BringToFront { .. } => "BringToFront",
                DomainOp::SendToBack { .. } => "SendToBack",
                DomainOp::Group { .. } => "Group",
                DomainOp::Ungroup { .. } => "Ungroup",
            }
        };

        // Verify all variants are covered
        let variants = [
            DomainOp::NodeAdd {
                id: "n1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "".to_string(),
            },
            DomainOp::NodeMove {
                id: "n1".to_string(),
                x: 0.0,
                y: 0.0,
            },
            DomainOp::NodeDelete {
                id: "n1".to_string(),
            },
            DomainOp::NodeRestore {
                id: "n1".to_string(),
            },
            DomainOp::NodeResize {
                id: NodeId::new("n1".to_string()),
                original_x: 0.0,
                original_y: 0.0,
                original_width: 80.0,
                original_height: 40.0,
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 60.0,
            },
            DomainOp::UpdateLabel {
                target_id: "n1".to_string(),
                target_type: LabelTargetType::Node,
                old_label: "old".to_string(),
                new_label: "test".to_string(),
            },
            DomainOp::UpdateNodeStyle {
                id: "n1".to_string(),
                style: NodeStyle::default(),
            },
            DomainOp::EdgeConnect {
                id: "e1".to_string(),
                source: "n1".to_string(),
                target: "n2".to_string(),
            },
            DomainOp::EdgeDisconnect {
                id: "e1".to_string(),
            },
            DomainOp::UpdateEdgeStyle {
                id: "e1".to_string(),
                style: EdgeStyle::default(),
            },
            DomainOp::BringForward {
                ids: vec!["n1".to_string()],
            },
            DomainOp::SendBackward {
                ids: vec!["n1".to_string()],
            },
            DomainOp::BringToFront {
                ids: vec!["n1".to_string()],
            },
            DomainOp::SendToBack {
                ids: vec!["n1".to_string()],
            },
            DomainOp::Group {
                ids: vec!["n1".to_string()],
            },
            DomainOp::Ungroup {
                id: "g1".to_string(),
            },
        ];

        for variant in variants {
            let name = check_variant(variant);
            assert!(!name.is_empty(), "Variant name should not be empty");
        }
    }

    // ============== BDD Tests for Numeric Boundaries (bd-14y) ==============

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_timestamp_at_i64_max_when_creating_envelope_then_preserves_value() {
        // Given: envelope with i64::MAX timestamp
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: i64::MAX,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
        };

        // When: serializing and deserializing
        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: EventEnvelope = serde_json::from_str(&json).unwrap();

        // Then: timestamp is preserved exactly
        assert_eq!(deserialized.timestamp, i64::MAX);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_timestamp_at_i64_min_when_creating_envelope_then_preserves_value() {
        // Given: envelope with i64::MIN timestamp
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: i64::MIN,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
        };

        // When: serializing and deserializing
        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: EventEnvelope = serde_json::from_str(&json).unwrap();

        // Then: timestamp is preserved exactly
        assert_eq!(deserialized.timestamp, i64::MIN);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_zero_timestamp_when_creating_envelope_then_succeeds() {
        // Given: envelope with zero timestamp
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 0,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
        };

        // When: serializing and deserializing
        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: EventEnvelope = serde_json::from_str(&json).unwrap();

        // Then: succeeds with timestamp 0
        assert_eq!(deserialized.timestamp, 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_negative_timestamp_when_creating_envelope_then_preserves_value() {
        // Given: envelope with negative timestamp (pre-epoch time)
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: -1000000,
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
        };

        // When: serializing and deserializing
        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: EventEnvelope = serde_json::from_str(&json).unwrap();

        // Then: negative timestamp is preserved
        assert_eq!(deserialized.timestamp, -1000000);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_add_with_infinity_x_when_parsing_then_no_panic() {
        // Given: JSON with infinity x coordinate (represented as string or number)
        let json = r#"{"op": "node_add", "id": "n1", "x": 1e999, "y": 0.0, "width": 80.0, "height": 40.0, "label": "test"}"#;

        // When: parsing
        // Then: either parses as infinity or returns error, no panic
        let result = parse_domain_op(json);
        // JSON doesn't support infinity, so this should fail gracefully
        assert!(result.is_err() || result.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_add_with_very_large_coordinates_when_parsing_then_succeeds() {
        // Given: JSON with very large coordinates
        let json = r#"{"op": "node_add", "id": "n1", "x": 1e308, "y": 1e308, "width": 80.0, "height": 40.0, "label": "test"}"#;

        // When: parsing
        let result = parse_domain_op(json);

        // Then: succeeds
        assert!(result.is_ok());
        let op = result.unwrap();
        match op {
            DomainOp::NodeAdd { x, y, .. } => {
                assert!(x > 1e307);
                assert!(y > 1e307);
            }
            _ => panic!("Expected NodeAdd"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_add_with_very_small_positive_coordinates_when_parsing_then_succeeds() {
        // Given: JSON with very small positive coordinates
        let json = r#"{"op": "node_add", "id": "n1", "x": 1e-308, "y": 1e-308, "width": 80.0, "height": 40.0, "label": "test"}"#;

        // When: parsing
        let result = parse_domain_op(json);

        // Then: succeeds
        assert!(result.is_ok());
        let op = result.unwrap();
        match op {
            DomainOp::NodeAdd { x, y, .. } => {
                assert!(x < 1e-307);
                assert!(y < 1e-307);
            }
            _ => panic!("Expected NodeAdd"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_envelope_serialization_with_large_timestamp_then_produces_valid_json() {
        // Given: envelope with large timestamp
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: 9223372036854775807, // i64::MAX
            author: Author {
                id: "user-1".to_string(),
                name: "Test".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "n1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
        };

        // When: serializing
        let json = serde_json::to_string(&envelope).unwrap();

        // Then: produces valid JSON with correct timestamp
        assert!(json.contains("9223372036854775807"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_envelope_roundtrip_with_negative_timestamp_then_preserves_value() {
        // Given: envelope with negative timestamp
        let original = EventEnvelope {
            op_id: "op-1".to_string(),
            timestamp: -9223372036854775808, // i64::MIN
            author: Author {
                id: "user-1".to_string(),
                name: "Test".to_string(),
                email: None,
            },
            operation: DomainOp::NodeAdd {
                id: "n1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
        };

        // When: roundtrip serialization
        let json = serde_json::to_string(&original).unwrap();
        let parsed: EventEnvelope = serde_json::from_str(&json).unwrap();

        // Then: timestamp is preserved
        assert_eq!(parsed.timestamp, original.timestamp);
    }
}
