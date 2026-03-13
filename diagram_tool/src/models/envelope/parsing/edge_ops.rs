//! Edge operation parsers
//!
//! This module provides parsing functions for edge-related domain operations.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::document::{EdgeId, EdgeStyle};
use crate::models::envelope::domain_ops::DomainOp;
use crate::models::envelope::types::ContractError;

use super::helpers::{extract_string_field, require_non_empty_edge_id, require_non_empty_id};

pub fn parse_edge_connect(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_edge_id(&extract_string_field(value, "id")?)?;
    let source = require_non_empty_id(&extract_string_field(value, "source")?)?;
    let target = require_non_empty_id(&extract_string_field(value, "target")?)?;

    Ok(DomainOp::EdgeConnect { id, source, target })
}

pub fn parse_edge_disconnect(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_edge_id(&extract_string_field(value, "id")?)?;
    Ok(DomainOp::EdgeDisconnect { id })
}

pub fn parse_update_edge_style(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_edge_id(&extract_string_field(value, "id")?)?;
    let style_str = value
        .get("style")
        .and_then(|v| v.as_str())
        .ok_or(ContractError::MissingField("style"))?;
    let style = parse_edge_style(style_str)?;

    Ok(DomainOp::UpdateEdgeStyle { id, style })
}

/// Parse an EdgeStyle from a string, returning an error for invalid values
fn parse_edge_style(s: &str) -> Result<EdgeStyle, ContractError> {
    match s {
        "solid" => Ok(EdgeStyle::Solid),
        "dashed" => Ok(EdgeStyle::Dashed),
        "dotted" => Ok(EdgeStyle::Dotted),
        _ => Err(ContractError::InvalidPayload(format!(
            "unknown edge style: {s}"
        ))),
    }
}
