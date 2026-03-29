//! Z-order operation parsers
//!
//! This module provides parsing functions for z-order domain operations.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::envelope::domain_ops::DomainOp;
use crate::envelope::types::ContractError;

use super::helpers::parse_node_id_array;

pub fn parse_bring_forward(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::BringForward { ids })
}

pub fn parse_send_backward(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::SendBackward { ids })
}

pub fn parse_bring_to_front(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::BringToFront { ids })
}

pub fn parse_send_to_back(value: &serde_json::Value) -> Result<DomainOp, ContractError> {
    let ids = parse_node_id_array(value.get("ids"))?;
    Ok(DomainOp::SendToBack { ids })
}
