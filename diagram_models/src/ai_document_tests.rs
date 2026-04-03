#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::similar_names,
    clippy::redundant_clone,
    unused_variables,
    unused_imports
)]

//! Tests for AiDocument newtype behavior and invariants.
//!
//! These tests cover:
//! - AiDocument field access
//! - AiDocument validation rules
//! - AiDocument with all LocationType variants
//!
//! # RED PHASE
//!
//! These tests are written BEFORE the implementation exists. They will compile
//! but fail at test execution time because the types don't exist yet.

use crate::schema_ai_documents::AiDocument;
use crate::schema_ai_documents::LocationType;

// =============================================================================
// AiDocument Field Access Tests
// =============================================================================

#[test]
fn ai_document_id_accessor_returns_correct_value() {
    // Given: An AiDocument with known id
    let id = "test-doc-id".to_string();
    let doc = AiDocument::new(
        id.clone(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Gps,
        "0,0".to_string(),
        0,
    )
    .unwrap();

    // When: The id is accessed via accessor
    // Then: It returns the correct value
    assert_eq!(doc.id(), &id);
}

#[test]
fn ai_document_key_accessor_returns_correct_value() {
    // Given: An AiDocument with known key
    let key = "test-key-value".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        key.clone(),
        "{}".to_string(),
        LocationType::Gps,
        "0,0".to_string(),
        0,
    )
    .unwrap();

    // When: The key is accessed via accessor
    // Then: It returns the correct value
    assert_eq!(doc.key(), &key);
}

#[test]
fn ai_document_json_payload_accessor_returns_correct_value() {
    // Given: An AiDocument with known json_payload
    let json_payload = r#"{"foo":"bar","baz":123}"#.to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        json_payload.clone(),
        LocationType::Gps,
        "0,0".to_string(),
        0,
    )
    .unwrap();

    // When: The json_payload is accessed via accessor
    // Then: It returns the correct value
    assert_eq!(doc.json_payload(), &json_payload);
}

#[test]
fn ai_document_created_at_accessor_returns_correct_value() {
    // Given: An AiDocument with known created_at
    let created_at = 1700000000i64;
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Gps,
        "0,0".to_string(),
        created_at,
    )
    .unwrap();

    // When: The created_at is accessed via accessor
    // Then: It returns the correct value
    assert_eq!(doc.created_at(), &created_at);
}

// =============================================================================
// AiDocument with All LocationType Variants
// =============================================================================

#[test]
fn ai_document_accepts_gps_location_type() {
    // Given: GPS location type and valid data
    let location_data = "37.7749,-122.4194".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Gps,
        location_data.clone(),
        0,
    )
    .unwrap();

    // Then: The document is created with GPS location
    assert_eq!(doc.location_type(), &LocationType::Gps);
    assert_eq!(doc.location_data(), &location_data);
}

#[test]
fn ai_document_accepts_file_path_location_type() {
    // Given: file_path location type and valid data
    let location_data = "/home/user/document.md".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::FilePath,
        location_data.clone(),
        0,
    )
    .unwrap();

    // Then: The document is created with file_path location
    assert_eq!(doc.location_type(), &LocationType::FilePath);
    assert_eq!(doc.location_data(), &location_data);
}

#[test]
fn ai_document_accepts_document_position_location_type() {
    // Given: document_position location type and valid data
    let location_data = "line:col 42:10".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::DocumentPosition,
        location_data.clone(),
        0,
    )
    .unwrap();

    // Then: The document is created with document_position location
    assert_eq!(doc.location_type(), &LocationType::DocumentPosition);
    assert_eq!(doc.location_data(), &location_data);
}

#[test]
fn ai_document_accepts_url_location_type() {
    // Given: URL location type and valid data
    let location_data = "https://example.com/document".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Url,
        location_data.clone(),
        0,
    )
    .unwrap();

    // Then: The document is created with URL location
    assert_eq!(doc.location_type(), &LocationType::Url);
    assert_eq!(doc.location_data(), &location_data);
}

// =============================================================================
// AiDocument Validation Edge Cases
// =============================================================================

#[test]
fn ai_document_rejects_whitespace_only_id() {
    // Given: A whitespace-only id string
    let id = "   ".to_string();
    let doc = AiDocument::new(
        id,
        "key".to_string(),
        "{}".to_string(),
        LocationType::Gps,
        "0,0".to_string(),
        0,
    );

    // Then: Creation fails
    assert!(doc.is_err());
}

#[test]
fn ai_document_rejects_whitespace_only_key() {
    // Given: A whitespace-only key string
    let key = "   ".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        key,
        "{}".to_string(),
        LocationType::Gps,
        "0,0".to_string(),
        0,
    );

    // Then: Creation fails
    assert!(doc.is_err());
}

#[test]
fn ai_document_accepts_empty_json_payload() {
    // Given: An empty json_payload string
    let json_payload = "".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        json_payload,
        LocationType::Gps,
        "0,0".to_string(),
        0,
    );

    // Then: Creation succeeds (empty string is valid JSON)
    assert!(doc.is_ok());
}

#[test]
fn ai_document_accepts_any_created_at_value() {
    // Given: Valid fields but zero created_at
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Gps,
        "0,0".to_string(),
        0,
    );

    // Then: Creation succeeds
    assert!(doc.is_ok());
    assert_eq!(doc.unwrap().created_at(), &0i64);
}

#[test]
fn ai_document_accepts_negative_created_at() {
    // Given: Valid fields but negative created_at (valid Unix timestamp before epoch)
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Gps,
        "0,0".to_string(),
        -1,
    );

    // Then: Creation succeeds
    assert!(doc.is_ok());
}

#[test]
fn ai_document_accepts_large_created_at() {
    // Given: Valid fields with a large created_at timestamp
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Gps,
        "0,0".to_string(),
        i64::MAX,
    );

    // Then: Creation succeeds
    assert!(doc.is_ok());
}

// =============================================================================
// AiDocument with Various Location Data
// =============================================================================

#[test]
fn ai_document_stores_gps_coordinates_with_decimals() {
    // Given: GPS coordinates with decimal places
    let location_data = "-33.8688,151.2093".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Gps,
        location_data.clone(),
        0,
    )
    .unwrap();

    // Then: Location data is preserved exactly
    assert_eq!(doc.location_data(), &location_data);
}

#[test]
fn ai_document_stores_file_path_with_special_characters() {
    // Given: A file path with special characters
    let location_data = "/home/user/Documents/file (copy).txt".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::FilePath,
        location_data.clone(),
        0,
    )
    .unwrap();

    // Then: Location data is preserved exactly
    assert_eq!(doc.location_data(), &location_data);
}

#[test]
fn ai_document_stores_url_with_query_parameters() {
    // Given: A URL with query parameters
    let location_data = "https://example.com/path?foo=bar&baz=qux".to_string();
    let doc = AiDocument::new(
        "id".to_string(),
        "key".to_string(),
        "{}".to_string(),
        LocationType::Url,
        location_data.clone(),
        0,
    )
    .unwrap();

    // Then: Location data is preserved exactly
    assert_eq!(doc.location_data(), &location_data);
}
