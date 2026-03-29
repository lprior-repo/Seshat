//! Composite operation parsers
//!
//! This module provides parsing functions for composite domain operations.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::envelope::domain_ops::DomainOp;
use crate::envelope::types::ContractError;

use super::helpers::{extract_string_field, parse_node_id_array, require_non_empty_id};

pub fn parse_group(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::Group { id, ids })
}

pub fn parse_ungroup(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let id = require_non_empty_id(&extract_string_field(value, "id")?)?;
    Ok(DomainOp::Ungroup { id })
}
