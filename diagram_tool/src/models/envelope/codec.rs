//! Event envelope codec
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::envelope::{ContractError, DomainOp, EventEnvelope};

pub fn parse_domain_op(raw: &str) -> Result<DomainOp, ContractError> {
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|e| ContractError::InvalidJson(e.to_string()))?;
    let op_field = value.get("op").and_then(|v| v.as_str()).ok_or(ContractError::MissingField("op"))?;
    match op_field {
        "node_add" => Ok(DomainOp::NodeAdd { id: value.get("id").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("id"))?, x: value.get("x").and_then(|v| v.as_f64()).ok_or(ContractError::MissingField("x"))?, y: value.get("y").and_then(|v| v.as_f64()).ok_or(ContractError::MissingField("y"))?, width: value.get("width").and_then(|v| v.as_f64()).ok_or(ContractError::MissingField("width"))?, height: value.get("height").and_then(|v| v.as_f64()).ok_or(ContractError::MissingField("height"))?, label: value.get("label").and_then(|v| v.as_str()).map(String::from).unwrap_or_default() }),
        "node_move" => Ok(DomainOp::NodeMove { id: value.get("id").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("id"))?, x: value.get("x").and_then(|v| v.as_f64()).ok_or(ContractError::MissingField("x"))?, y: value.get("y").and_then(|v| v.as_f64()).ok_or(ContractError::MissingField("y"))? }),
        "node_delete" => Ok(DomainOp::NodeDelete { id: value.get("id").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("id"))? }),
        "node_restore" => Ok(DomainOp::NodeRestore { id: value.get("id").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("id"))? }),
        "edge_connect" => Ok(DomainOp::EdgeConnect { id: value.get("id").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("id"))?, source: value.get("source").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("source"))?, target: value.get("target").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("target"))? }),
        "edge_disconnect" => Ok(DomainOp::EdgeDisconnect { id: value.get("id").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("id"))? }),
        "bring_forward" => Ok(DomainOp::BringForward { ids: parse_string_array(value.get("ids"))? }),
        "send_backward" => Ok(DomainOp::SendBackward { ids: parse_string_array(value.get("ids"))? }),
        "bring_to_front" => Ok(DomainOp::BringToFront { ids: parse_string_array(value.get("ids"))? }),
        "send_to_back" => Ok(DomainOp::SendToBack { ids: parse_string_array(value.get("ids"))? }),
        "group" => Ok(DomainOp::Group { ids: parse_string_array(value.get("ids"))? }),
        "ungroup" => Ok(DomainOp::Ungroup { id: value.get("id").and_then(|v| v.as_str()).map(String::from).ok_or(ContractError::MissingField("id"))? }),
        _ => Err(ContractError::UnknownOpType(op_field.to_string())),
    }
}

fn parse_string_array(value: Option<&serde_json::Value>) -> Result<Vec<String>, ContractError> {
    let arr = value.and_then(|v| v.as_array()).ok_or(ContractError::InvalidPayload("expected array".to_string()))?;
    arr.iter().map(|v| v.as_str().map(String::from).ok_or_else(|| ContractError::InvalidPayload("expected string".to_string()))).collect()
}

#[deprecated(since = "0.1.0", note = "use parse_event_envelope instead")]
pub fn decode_envelope(raw: &str) -> Result<EventEnvelope, ContractError> { parse_event_envelope(raw) }

#[deprecated(since = "0.1.0", note = "use encode_event_envelope instead")]
pub fn encode_envelope(op: &EventEnvelope) -> Result<String, ContractError> { encode_event_envelope(op) }

pub fn parse_event_envelope(input: &str) -> Result<EventEnvelope, ContractError> {
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| ContractError::InvalidJson(e.to_string()))?;
    let _ = value.get("op_id").ok_or(ContractError::MissingField("op_id"))?;
    let author_value = value.get("author").ok_or(ContractError::MissingField("author"))?;
    let _ = author_value.get("id").and_then(|v| v.as_str()).ok_or_else(|| ContractError::InvalidAuthor("missing id".to_string()))?;
    let _ = author_value.get("name").and_then(|v| v.as_str()).ok_or_else(|| ContractError::InvalidAuthor("missing name".to_string()))?;
    let _ = value.get("timestamp").ok_or(ContractError::MissingField("timestamp"))?;
    serde_json::from_str(input).map_err(|e| ContractError::InvalidJson(e.to_string()))
}

pub fn encode_event_envelope(op: &EventEnvelope) -> Result<String, ContractError> { serde_json::to_string(op).map_err(|e| ContractError::InvalidJson(e.to_string())) }
