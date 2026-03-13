//! Dispatch logic for domain operations
//!
//! This module provides the main entry point and dispatch logic.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::envelope::domain_ops::DomainOp;
use crate::models::envelope::types::ContractError;

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
        "node_add" => super::node_ops::parse_node_add(value),
        "node_move" => super::node_ops::parse_node_move(value),
        "node_delete" => super::node_ops::parse_node_delete(value),
        "node_restore" => super::node_ops::parse_node_restore(value),
        "node_resize" => super::node_ops::parse_node_resize(value),
        "update_label" => super::node_ops::parse_update_label(value),
        "update_node_style" => super::node_ops::parse_update_node_style(value),
        "edge_connect" => super::edge_ops::parse_edge_connect(value),
        "edge_disconnect" => super::edge_ops::parse_edge_disconnect(value),
        "update_edge_style" => super::edge_ops::parse_update_edge_style(value),
        "bring_forward" => super::zorder_ops::parse_bring_forward(value),
        "send_backward" => super::zorder_ops::parse_send_backward(value),
        "bring_to_front" => super::zorder_ops::parse_bring_to_front(value),
        "send_to_back" => super::zorder_ops::parse_send_to_back(value),
        "group" => super::composite_ops::parse_group(value),
        "ungroup" => super::composite_ops::parse_ungroup(value),
        _ => Err(ContractError::UnknownOpType(op_field.to_string())),
    }
}
