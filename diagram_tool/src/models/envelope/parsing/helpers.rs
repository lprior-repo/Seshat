//! Helper functions for parsing
//!
//! This module provides shared helper functions for parsing domain operations.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::document::{EdgeId, NodeId};
use crate::models::envelope::types::ContractError;

pub fn extract_string_field(
    value: &serde_json::Value,
    field: &str,
) -> Result<String, ContractError> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or(ContractError::MissingField(Box::leak(
            field.to_string().into_boxed_str(),
        )))
}

pub fn extract_f64_field(value: &serde_json::Value, field: &str) -> Result<f64, ContractError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_f64)
        .ok_or(ContractError::MissingField(Box::leak(
            field.to_string().into_boxed_str(),
        )))
}

pub fn require_non_empty_id(id: &str) -> Result<NodeId, ContractError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(ContractError::InvalidNodeId(
            "node id cannot be empty".to_string(),
        ));
    }
    Ok(NodeId::new(trimmed.to_string()))
}

pub fn require_non_empty_edge_id(id: &str) -> Result<EdgeId, ContractError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(ContractError::InvalidEdgeId(
            "edge id cannot be empty".to_string(),
        ));
    }
    Ok(EdgeId::new(trimmed.to_string()))
}

pub fn validate_positive_finite(value: f64, field_name: &str) -> Result<f64, ContractError> {
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

pub fn parse_string_array(value: Option<&serde_json::Value>) -> Result<Vec<String>, ContractError> {
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

pub fn parse_node_id_array(
    value: Option<&serde_json::Value>,
) -> Result<Vec<NodeId>, ContractError> {
    let strings = parse_string_array(value)?;
    let mut node_ids = Vec::with_capacity(strings.len());
    for s in strings {
        node_ids.push(require_non_empty_id(&s)?);
    }
    Ok(node_ids)
}
