//! Event envelope module - defines EventEnvelope and Author metadata
//!
//! This module provides types for encoding/decoding event envelopes
//! with strict validation of required fields.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for envelope operations
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContractError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid author: {0}")]
    InvalidAuthor(String),
    #[error("unknown operation type: {0}")]
    UnknownOpType(String),
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
    /// Parse OpType from string, returning UnknownOpType error for invalid values
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

/// Event envelope containing operation and author metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelope {
    /// Unique identifier for this envelope
    pub id: String,
    /// The operation type
    #[serde(rename = "op")]
    pub operation: OpType,
    /// Author who created this event
    pub author: Author,
    /// Timestamp of when this event was created (Unix timestamp)
    pub timestamp: i64,
    /// Optional payload/data for the event
    pub payload: Option<serde_json::Value>,
}

/// Decode an EventEnvelope from a JSON string
///
/// # Errors
/// Returns `ContractError::InvalidJson` if the JSON is malformed
/// Returns `ContractError::MissingField` if required fields are missing
/// Returns `ContractError::InvalidAuthor` if author validation fails
/// Returns `ContractError::UnknownOpType` if the operation type is invalid
pub fn decode_envelope(raw: &str) -> Result<EventEnvelope, ContractError> {
    // Parse JSON first
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| ContractError::InvalidJson(e.to_string()))?;

    // Extract and validate required fields
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or(ContractError::MissingField("id"))?;

    let op_str = value
        .get("op")
        .and_then(|v| v.as_str())
        .ok_or(ContractError::MissingField("op"))?;

    let operation = OpType::from_str(op_str)?;

    // Parse author object
    let author_value = value
        .get("author")
        .ok_or(ContractError::MissingField("author"))?;

    let author_id = author_value
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| ContractError::InvalidAuthor("missing id field".to_string()))?;

    let author_name = author_value
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| ContractError::InvalidAuthor("missing name field".to_string()))?;

    let email = author_value
        .get("email")
        .and_then(|v| v.as_str())
        .map(String::from);

    let author = Author {
        id: author_id,
        name: author_name,
        email,
    };

    // Parse timestamp
    let timestamp = value
        .get("timestamp")
        .and_then(|v| v.as_i64())
        .ok_or(ContractError::MissingField("timestamp"))?;

    // Optional payload - treat null as None
    let payload =
        value
            .get("payload")
            .and_then(|v| if v.is_null() { None } else { Some(v.clone()) });

    Ok(EventEnvelope {
        id,
        operation,
        author,
        timestamp,
        payload,
    })
}

/// Encode an EventEnvelope to a JSON string
///
/// # Errors
/// Returns `ContractError::InvalidJson` if encoding fails
pub fn encode_envelope(op: &EventEnvelope) -> Result<String, ContractError> {
    serde_json::to_string(op).map_err(|e| ContractError::InvalidJson(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_json_when_decoding_then_returns_envelope() {
        let raw = r#"{
            "id": "evt-123",
            "op": "create",
            "author": {
                "id": "user-1",
                "name": "Alice"
            },
            "timestamp": 1699999999,
            "payload": {"key": "value"}
        }"#;

        let result = decode_envelope(raw);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let envelope = result.unwrap();
        assert_eq!(envelope.id, "evt-123");
        assert_eq!(envelope.operation, OpType::Create);
        assert_eq!(envelope.author.id, "user-1");
        assert_eq!(envelope.author.name, "Alice");
        assert_eq!(envelope.timestamp, 1699999999);
    }

    #[test]
    fn given_invalid_json_when_decoding_then_returns_invalid_json_error() {
        let raw = "not valid json";

        let result = decode_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::InvalidJson(_)) => {}
            _ => panic!("Expected InvalidJson error"),
        }
    }

    #[test]
    fn given_missing_id_field_when_decoding_then_returns_missing_field_error() {
        let raw = r#"{
            "op": "create",
            "author": {"id": "user-1", "name": "Alice"},
            "timestamp": 1699999999
        }"#;

        let result = decode_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::MissingField(f)) => assert_eq!(f, "id"),
            _ => panic!("Expected MissingField error for 'id'"),
        }
    }

    #[test]
    fn given_missing_author_field_when_decoding_then_returns_missing_field_error() {
        let raw = r#"{
            "id": "evt-123",
            "op": "create",
            "timestamp": 1699999999
        }"#;

        let result = decode_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::MissingField(f)) => assert_eq!(f, "author"),
            _ => panic!("Expected MissingField error for 'author'"),
        }
    }

    #[test]
    fn given_invalid_author_missing_name_when_decoding_then_returns_invalid_author_error() {
        let raw = r#"{
            "id": "evt-123",
            "op": "create",
            "author": {"id": "user-1"},
            "timestamp": 1699999999
        }"#;

        let result = decode_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::InvalidAuthor(_)) => {}
            _ => panic!("Expected InvalidAuthor error"),
        }
    }

    #[test]
    fn given_unknown_op_type_when_decoding_then_returns_unknown_op_type_error() {
        let raw = r#"{
            "id": "evt-123",
            "op": "unknown_operation",
            "author": {"id": "user-1", "name": "Alice"},
            "timestamp": 1699999999
        }"#;

        let result = decode_envelope(raw);

        assert!(result.is_err());
        match result {
            Err(ContractError::UnknownOpType(s)) => assert_eq!(s, "unknown_operation"),
            _ => panic!("Expected UnknownOpType error"),
        }
    }

    #[test]
    fn given_all_op_types_then_all_parse_correctly() {
        let test_cases = [
            (r#""op": "create""#, OpType::Create),
            (r#""op": "update""#, OpType::Update),
            (r#""op": "delete""#, OpType::Delete),
            (r#""op": "migrate""#, OpType::Migrate),
        ];

        for (op_str, expected) in test_cases {
            let raw = format!(
                r#"{{"id": "evt-1", {}, "author": {{"id": "u1", "name": "A"}}, "timestamp": 1}}"#,
                op_str
            );
            let result = decode_envelope(&raw);
            assert!(result.is_ok(), "Failed for op: {}", op_str);
            assert_eq!(result.unwrap().operation, expected);
        }
    }

    #[test]
    fn given_author_with_email_when_decoding_then_email_is_preserved() {
        let raw = r#"{
            "id": "evt-123",
            "op": "create",
            "author": {
                "id": "user-1",
                "name": "Alice",
                "email": "alice@example.com"
            },
            "timestamp": 1699999999
        }"#;

        let result = decode_envelope(raw);

        assert!(result.is_ok());
        let envelope = result.unwrap();
        assert_eq!(envelope.author.email, Some("alice@example.com".to_string()));
    }

    #[test]
    fn given_author_without_email_when_decoding_then_email_is_none() {
        let raw = r#"{
            "id": "evt-123",
            "op": "create",
            "author": {
                "id": "user-1",
                "name": "Alice"
            },
            "timestamp": 1699999999
        }"#;

        let result = decode_envelope(raw);

        assert!(result.is_ok());
        let envelope = result.unwrap();
        assert_eq!(envelope.author.email, None);
    }

    #[test]
    fn given_envelope_with_payload_when_encoding_then_roundtrip_works() {
        let original = EventEnvelope {
            id: "evt-roundtrip".to_string(),
            operation: OpType::Update,
            author: Author {
                id: "user-1".to_string(),
                name: "Bob".to_string(),
                email: Some("bob@example.com".to_string()),
            },
            timestamp: 1700000000,
            payload: Some(serde_json::json!({"data": "test"})),
        };

        let encoded = encode_envelope(&original);
        assert!(encoded.is_ok(), "Encoding failed: {:?}", encoded.err());

        let decoded = decode_envelope(&encoded.unwrap());
        assert!(decoded.is_ok(), "Decoding failed: {:?}", decoded.err());

        assert_eq!(decoded.unwrap(), original);
    }

    #[test]
    fn given_envelope_without_payload_when_encoding_then_roundtrip_works() {
        let original = EventEnvelope {
            id: "evt-nopayload".to_string(),
            operation: OpType::Delete,
            author: Author {
                id: "user-2".to_string(),
                name: "Charlie".to_string(),
                email: None,
            },
            timestamp: 1700000001,
            payload: None,
        };

        let encoded = encode_envelope(&original);
        assert!(encoded.is_ok(), "Encoding failed: {:?}", encoded.err());

        let decoded = decode_envelope(&encoded.unwrap());
        assert!(decoded.is_ok(), "Decoding failed: {:?}", decoded.err());

        assert_eq!(decoded.unwrap(), original);
    }
}
