#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::store_async::{
    parse::{
        envelope_batch_to_bounded_batch, envelope_to_valid_event, parse_bounded_batch,
        parse_revision, parse_valid_event,
    },
    AsyncStoreError,
};
use diagram_models::envelope::{Author, DomainOp, EventEnvelope, OpKind, OpType};

fn create_valid_envelope() -> EventEnvelope {
    EventEnvelope {
        op_id: "test-op-123".to_string(),
        operation: DomainOp::NodeMove {
            id: diagram_models::document::NodeId::new("n1".to_string()),
            x: 0.0,
            y: 0.0,
        },
        author: Author {
            id: "author-1".to_string(),
            name: "Author One".to_string(),
            email: None,
        },
        timestamp: 1600000000,
    }
}

#[test]
fn test_parse_valid_event_success() -> Result<(), Box<dyn std::error::Error>> {
    let op_id = "valid-id".to_string();
    let timestamp = 1600000000;
    let payload = r#"{"test":true}"#.to_string();

    let event = parse_valid_event(op_id.clone(), timestamp, payload.clone())?;
    assert_eq!(event.op_id.as_str(), "valid-id");
    assert_eq!(event.timestamp.get(), 1600000000);
    assert_eq!(event.payload.as_str(), r#"{"test":true}"#);
    Ok(())
}

#[test]
fn test_parse_valid_event_invalid_op_id() {
    let result = parse_valid_event("".to_string(), 1600000000, "{}".to_string());
    assert!(matches!(result, Err(AsyncStoreError::InvalidOperationId)));

    let long_id = "a".repeat(256);
    let result = parse_valid_event(long_id, 1600000000, "{}".to_string());
    assert!(matches!(result, Err(AsyncStoreError::OperationIdTooLong)));
}

#[test]
fn test_parse_valid_event_invalid_timestamp() {
    let result = parse_valid_event("op-1".to_string(), 0, "{}".to_string());
    assert!(matches!(result, Err(AsyncStoreError::InvalidTimestamp)));
}

#[test]
fn test_envelope_to_valid_event_success() -> Result<(), Box<dyn std::error::Error>> {
    let envelope = create_valid_envelope();
    let valid_event = envelope_to_valid_event(&envelope)?;

    assert_eq!(valid_event.op_id.as_str(), "test-op-123");
    assert_eq!(valid_event.timestamp.get(), 1600000000);

    // Check that it serialized correctly
    let deserialized: EventEnvelope = serde_json::from_str(valid_event.payload.as_str())?;
    assert_eq!(deserialized.op_id, "test-op-123");
    assert_eq!(deserialized.author.id, "author-1");
    Ok(())
}

#[test]
fn test_envelope_to_valid_event_invalid_timestamp() {
    let mut envelope = create_valid_envelope();
    envelope.timestamp = -1; // Negative timestamp

    let result = envelope_to_valid_event(&envelope);
    assert!(matches!(result, Err(AsyncStoreError::InvalidTimestamp)));
}

#[test]
fn test_parse_revision() -> Result<(), Box<dyn std::error::Error>> {
    let rev = parse_revision(5)?;
    assert_eq!(rev.get(), 5);

    let result = parse_revision(-1);
    assert!(matches!(result, Err(AsyncStoreError::ValidationFailed(_))));
    Ok(())
}

#[test]
fn test_bounded_batch_limits() {
    let envelope = create_valid_envelope();

    // Test minimum limit (MIN = 1)
    let empty: Vec<EventEnvelope> = vec![];
    let result = envelope_batch_to_bounded_batch::<1, 10>(&empty);
    assert!(matches!(result, Err(AsyncStoreError::EmptyBatch)));

    // Test maximum limit (MAX = 1)
    let too_many = vec![envelope.clone(), envelope.clone()];
    let result = envelope_batch_to_bounded_batch::<1, 1>(&too_many);
    assert!(matches!(result, Err(AsyncStoreError::BatchTooLarge)));

    // Test happy path
    let just_right = vec![envelope.clone()];
    let result = envelope_batch_to_bounded_batch::<1, 10>(&just_right);
    assert!(result.is_ok());
}
