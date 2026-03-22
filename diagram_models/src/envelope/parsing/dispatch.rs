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

use crate::envelope::domain_ops::DomainOp;
use crate::envelope::types::ContractError;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain_op_invalid_json() {
        let result = parse_domain_op("not json");
        assert!(matches!(result, Err(ContractError::InvalidJson(_))));
    }

    #[test]
    fn test_parse_domain_op_missing_op() {
        let result = parse_domain_op(r#"{"id": "n1"}"#);
        assert!(matches!(result, Err(ContractError::MissingField("op"))));
    }

    #[test]
    fn test_parse_domain_op_unknown_op() {
        let result = parse_domain_op(r#"{"op": "unknown_action"}"#);
        assert!(matches!(result, Err(ContractError::UnknownOpType(op)) if op == "unknown_action"));
    }

    #[test]
    fn test_dispatch_domain_op_node_add() {
        let valid_node_add = r#"{"op": "node_add", "id": "n1", "x": 10.0, "y": 20.0, "width": 100.0, "height": 50.0, "label": "test"}"#;
        let result = parse_domain_op(valid_node_add);
        assert!(matches!(result, Ok(DomainOp::NodeAdd { .. })));
    }

    #[test]
    fn test_dispatch_domain_op_node_move() {
        let valid_node_move = r#"{"op": "node_move", "id": "n1", "x": 10.0, "y": 20.0}"#;
        assert!(matches!(
            parse_domain_op(valid_node_move),
            Ok(DomainOp::NodeMove { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_node_delete() {
        let valid = r#"{"op": "node_delete", "id": "n1"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::NodeDelete { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_node_restore() {
        let valid = r#"{"op": "node_restore", "id": "n1"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::NodeRestore { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_node_resize() {
        let valid = r#"{"op": "node_resize", "id": "n1", "original_x": 0.0, "original_y": 0.0, "original_width": 10.0, "original_height": 10.0, "x": 10.0, "y": 20.0, "width": 100.0, "height": 50.0}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::NodeResize { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_update_label() {
        let valid = r#"{"op": "update_label", "target_id": "n1", "target_type": "node", "old_label": "a", "new_label": "b"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::UpdateLabel { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_update_node_style() {
        let valid = r#"{"op": "update_node_style", "id": "n1", "style": "box"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::UpdateNodeStyle { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_edge_connect() {
        let valid = r#"{"op": "edge_connect", "id": "e1", "source": "n1", "target": "n2"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::EdgeConnect { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_edge_disconnect() {
        let valid = r#"{"op": "edge_disconnect", "id": "e1"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::EdgeDisconnect { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_update_edge_style() {
        let valid = r#"{"op": "update_edge_style", "id": "e1", "style": "solid"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::UpdateEdgeStyle { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_bring_forward() {
        let valid = r#"{"op": "bring_forward", "ids": ["n1"]}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::BringForward { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_send_backward() {
        let valid = r#"{"op": "send_backward", "ids": ["n1"]}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::SendBackward { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_bring_to_front() {
        let valid = r#"{"op": "bring_to_front", "ids": ["n1"]}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::BringToFront { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_send_to_back() {
        let valid = r#"{"op": "send_to_back", "ids": ["n1"]}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::SendToBack { .. })
        ));
    }

    #[test]
    fn test_dispatch_domain_op_group() {
        let valid = r#"{"op": "group", "id": "g1", "ids": ["n1"]}"#;
        assert!(matches!(parse_domain_op(valid), Ok(DomainOp::Group { .. })));
    }

    #[test]
    fn test_dispatch_domain_op_ungroup() {
        let valid = r#"{"op": "ungroup", "id": "g1"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::Ungroup { .. })
        ));
    }
}
