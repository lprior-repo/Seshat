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

pub mod domain_ops;
pub mod parsing;
pub mod tests;
pub mod types;

pub use domain_ops::{domain_op_kind, DomainOp};
pub use parsing::parse_domain_op;
pub use types::{Author, ContractError, LabelTargetId, LabelTargetType, OpKind, OpType};

use serde::{Deserialize, Serialize};

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
        // Extract field name from "missing field `field_name` at ..."
        // Split on the opening backtick, then take everything up to the closing backtick
        let field = err_msg
            .split("missing field `")
            .nth(1)
            .and_then(|rest| rest.split('`').next())
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
        "op_id" => ContractError::MissingField("op_id"),
        "operation" => ContractError::MissingField("operation"),
        "id" => ContractError::InvalidAuthor("missing id field".to_string()),
        "name" => ContractError::InvalidAuthor("missing name field".to_string()),
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
