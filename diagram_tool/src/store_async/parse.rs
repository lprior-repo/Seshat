//! Parsing functions for the async store.

use crate::store::types::{
    BoundedBatch, Revision, ValidEvent, ValidOperationId, ValidPayload, ValidTimestamp,
};
use diagram_models::envelope::{encode_event_envelope, EventEnvelope};

use super::error::AsyncStoreError;

/// Converts an `EventEnvelope` to a `ValidEvent` for testing and migration purposes.
///
/// This helper encodes the envelope as JSON payload and creates a `ValidEvent`.
/// It preserves the original envelope structure in the payload field.
///
/// # Errors
/// Returns an error if encoding fails or validation fails.
pub fn envelope_to_valid_event(envelope: &EventEnvelope) -> Result<ValidEvent, AsyncStoreError> {
    let payload =
        encode_event_envelope(envelope).map_err(|e: diagram_models::envelope::ContractError| {
            AsyncStoreError::Serialization(e.to_string())
        })?;

    let timestamp =
        u64::try_from(envelope.timestamp).map_err(|_| AsyncStoreError::InvalidTimestamp)?;

    parse_valid_event(envelope.op_id.clone(), timestamp, payload)
}

/// Converts a batch of `EventEnvelopes` to a `BoundedBatch` for testing.
///
/// # Errors
/// Returns an error if any envelope conversion fails or batch bounds are violated.
pub fn envelope_batch_to_bounded_batch<const MIN: usize, const MAX: usize>(
    envelopes: &[EventEnvelope],
) -> Result<BoundedBatch<MIN, MAX>, AsyncStoreError> {
    let events: Result<Vec<ValidEvent>, _> =
        envelopes.iter().map(envelope_to_valid_event).collect();

    let events = events?;
    parse_bounded_batch::<MIN, MAX>(events)
}

/// Parse raw inputs into `ValidEvent` at boundary.
///
/// This is the entry point for converting external primitive inputs
/// into the validated DDD type.
///
/// # Errors
/// Returns an error if any of the inputs fail validation.
pub fn parse_valid_event(
    op_id: String,
    timestamp: u64,
    payload: String,
) -> Result<ValidEvent, AsyncStoreError> {
    let op_id = ValidOperationId::new(op_id)?;
    let timestamp = ValidTimestamp::new(timestamp)?;
    let payload = ValidPayload::new(payload)?;
    Ok(ValidEvent {
        op_id,
        timestamp,
        payload,
    })
}

/// Parse events into `BoundedBatch` at boundary.
///
/// # Errors
/// Returns an error if the batch size is outside the MIN/MAX bounds.
pub fn parse_bounded_batch<const MIN: usize, const MAX: usize>(
    events: Vec<ValidEvent>,
) -> Result<BoundedBatch<MIN, MAX>, AsyncStoreError> {
    BoundedBatch::try_from(events)
}

/// Parse raw revision input into Revision type.
///
/// # Errors
/// Returns an error if the revision is negative.
pub fn parse_revision(rev: i64) -> Result<Revision, AsyncStoreError> {
    let rev = u64::try_from(rev).map_err(|_| {
        AsyncStoreError::ValidationFailed("Revision cannot be negative".to_string())
    })?;
    Revision::new(rev)
}
