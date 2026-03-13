//! Node operation parsers
//!
//! This module provides parsing functions for node-related domain operations.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::document::{NodeId, NodeStyle};
use crate::models::envelope::domain_ops::DomainOp;
use crate::models::envelope::types::{ContractError, LabelTargetId, LabelTargetType};

use super::helpers::{
    extract_f64_field, extract_string_field, require_non_empty_id, validate_positive_finite,
};

/// Helper struct for NodeResize dimensions
pub(super) struct NodeResizeDimensions {
    pub(super) original_x: f64,
    pub(super) original_y: f64,
    pub(super) original_width: f64,
    pub(super) original_height: f64,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
}

pub(super) fn extract_and_validate_dimensions(
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

pub fn parse_node_add(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
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

pub fn parse_node_move(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    let x = extract_f64_field(value, "x")?;
    let y = extract_f64_field(value, "y")?;

    Ok(DomainOp::NodeMove { id, x, y })
}

pub fn parse_node_delete(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    Ok(DomainOp::NodeDelete { id })
}

pub fn parse_node_restore(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    Ok(DomainOp::NodeRestore { id })
}

pub fn parse_node_resize(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
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

pub fn parse_update_label(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    // Determine target type first
    let target_type_str = value
        .get("target_type")
        .and_then(|v| v.as_str())
        .unwrap_or("node");
    let target_type = match target_type_str {
        "node" => LabelTargetType::Node,
        "edge" => LabelTargetType::Edge,
        _ => LabelTargetType::Node,
    };

    // Parse target_id based on target type
    let target_id = match target_type {
        LabelTargetType::Node => {
            let id_str = value
                .get("id")
                .and_then(|v| v.as_str())
                .or_else(|| value.get("target_id").and_then(|v| v.as_str()))
                .unwrap_or("");
            LabelTargetId::Node(require_non_empty_id(id_str)?)
        }
        LabelTargetType::Edge => {
            let id_str = value
                .get("id")
                .and_then(|v| v.as_str())
                .or_else(|| value.get("target_id").and_then(|v| v.as_str()))
                .unwrap_or("");
            LabelTargetId::Edge(super::helpers::require_non_empty_edge_id(id_str)?)
        }
    };

    let old_label = value
        .get("old_label")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();
    let new_label = value
        .get("new_label")
        .or_else(|| value.get("label"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    Ok(DomainOp::UpdateLabel {
        target_id,
        target_type,
        old_label,
        new_label,
    })
}

pub fn parse_update_node_style(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    let style_str = value
        .get("style")
        .and_then(|v| v.as_str())
        .ok_or(ContractError::MissingField("style"))?;
    let style = parse_node_style(style_str)?;

    Ok(DomainOp::UpdateNodeStyle { id, style })
}

/// Parse a NodeStyle from a string, returning an error for invalid values
fn parse_node_style(s: &str) -> Result<NodeStyle, ContractError> {
    match s {
        "box" => Ok(NodeStyle::Box),
        "cloud" => Ok(NodeStyle::Cloud),
        "cylinder" => Ok(NodeStyle::Cylinder),
        "dashed" => Ok(NodeStyle::Dashed),
        _ => Err(ContractError::InvalidPayload(format!(
            "unknown node style: {s}"
        ))),
    }
}
