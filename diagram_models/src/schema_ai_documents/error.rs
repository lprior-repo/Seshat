//! Error types for `AiDocument` validation failures.

/// Error type for `AiDocument` validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiDocumentError {
    /// The id field is empty or whitespace-only.
    EmptyId,
    /// The key field is empty or whitespace-only.
    EmptyKey,
    /// The `location_data` field is not valid JSON.
    InvalidJson,
    /// The `location_data` field does not match the `location_type` format.
    InvalidLocationDataFormat,
    /// The `location_data` field is empty.
    EmptyLocationData,
}
