#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::similar_names,
    clippy::redundant_clone,
    unused_variables,
    unused_imports
)]

//! Tests for the `ai_documents` schema and related types.
//!
//! These tests verify the structure and behavior of:
//! - `SCHEMA_AI_DOCUMENTS_TABLE` constant
//! - `LocationType` enum parsing
//! - `AiDocument` newtype
//!
//! # RED PHASE
//!
//! These tests are written BEFORE the implementation exists. They will compile
//! but fail at test execution time because the types don't exist yet.

use crate::schema_ai_documents::AiDocument;
use crate::schema_ai_documents::LocationType;
use crate::schema_ai_documents::LocationTypeParseError;
use crate::schema_ai_documents::SCHEMA_AI_DOCUMENTS_TABLE;
use std::str::FromStr;

// =============================================================================
// SCHEMA_AI_DOCUMENTS_TABLE Constant Tests
// =============================================================================

#[test]
fn schema_ai_documents_constant_exists_and_is_non_empty_when_accessed() {
    // Given: The SCHEMA_AI_DOCUMENTS_TABLE constant is defined
    // When: The constant is accessed
    let schema = SCHEMA_AI_DOCUMENTS_TABLE;
    // Then: It is a non-empty string
    assert!(
        !schema.is_empty(),
        "SCHEMA_AI_DOCUMENTS_TABLE should not be empty"
    );
}

#[test]
fn schema_ai_documents_starts_with_create_table_when_accessed() {
    // Given: The SCHEMA_AI_DOCUMENTS_TABLE constant is defined
    // When: The constant is accessed
    let schema = SCHEMA_AI_DOCUMENTS_TABLE.trim();
    // Then: It starts with "CREATE TABLE IF NOT EXISTS"
    assert!(
        schema.starts_with("CREATE TABLE IF NOT EXISTS"),
        "SCHEMA_AI_DOCUMENTS_TABLE should start with 'CREATE TABLE IF NOT EXISTS', got: {}",
        schema
    );
}

#[test]
fn schema_ai_documents_contains_id_text_not_null_primary_key() {
    // Given: SCHEMA_AI_DOCUMENTS_TABLE constant
    let schema = SCHEMA_AI_DOCUMENTS_TABLE;
    // Then: It contains `id TEXT NOT NULL PRIMARY KEY`
    assert!(
        schema.contains("id TEXT NOT NULL PRIMARY KEY"),
        "Schema should contain 'id TEXT NOT NULL PRIMARY KEY', got: {}",
        schema
    );
}

#[test]
fn schema_ai_documents_contains_key_text_not_null() {
    // Given: SCHEMA_AI_DOCUMENTS_TABLE constant
    let schema = SCHEMA_AI_DOCUMENTS_TABLE;
    // Then: It contains `key TEXT NOT NULL`
    assert!(
        schema.contains("key TEXT NOT NULL"),
        "Schema should contain 'key TEXT NOT NULL', got: {}",
        schema
    );
}

#[test]
fn schema_ai_documents_contains_json_payload_text_not_null() {
    // Given: SCHEMA_AI_DOCUMENTS_TABLE constant
    let schema = SCHEMA_AI_DOCUMENTS_TABLE;
    // Then: It contains `json_payload TEXT NOT NULL`
    assert!(
        schema.contains("json_payload TEXT NOT NULL"),
        "Schema should contain 'json_payload TEXT NOT NULL', got: {}",
        schema
    );
}

#[test]
fn schema_ai_documents_contains_location_type_text_not_null() {
    // Given: SCHEMA_AI_DOCUMENTS_TABLE constant
    let schema = SCHEMA_AI_DOCUMENTS_TABLE;
    // Then: It contains `location_type TEXT NOT NULL`
    assert!(
        schema.contains("location_type TEXT NOT NULL"),
        "Schema should contain 'location_type TEXT NOT NULL', got: {}",
        schema
    );
}

#[test]
fn schema_ai_documents_contains_location_data_text_not_null() {
    // Given: SCHEMA_AI_DOCUMENTS_TABLE constant
    let schema = SCHEMA_AI_DOCUMENTS_TABLE;
    // Then: It contains `location_data TEXT NOT NULL`
    assert!(
        schema.contains("location_data TEXT NOT NULL"),
        "Schema should contain 'location_data TEXT NOT NULL', got: {}",
        schema
    );
}

#[test]
fn schema_ai_documents_contains_created_at_integer() {
    // Given: SCHEMA_AI_DOCUMENTS_TABLE constant
    let schema = SCHEMA_AI_DOCUMENTS_TABLE;
    // Then: It contains `created_at INTEGER`
    assert!(
        schema.contains("created_at INTEGER"),
        "Schema should contain 'created_at INTEGER', got: {}",
        schema
    );
}

#[test]
fn schema_ai_documents_creates_table_named_ai_documents() {
    // Given: SCHEMA_AI_DOCUMENTS_TABLE constant
    let schema = SCHEMA_AI_DOCUMENTS_TABLE;
    // Then: It creates a table named `ai_documents`
    assert!(
        schema.contains("CREATE TABLE") && schema.contains("ai_documents"),
        "Schema should contain 'CREATE TABLE' and 'ai_documents', got: {}",
        schema
    );
}

// =============================================================================
// LocationType Enum Tests
// =============================================================================

#[test]
fn location_type_has_four_variants() {
    // Given: The LocationType enum is defined
    // When: All variants are parsed from their string representations
    let parsed_gps = LocationType::from_str("GPS").unwrap();
    let parsed_fpath = LocationType::from_str("file_path").unwrap();
    let parsed_docpos = LocationType::from_str("document_position").unwrap();
    let parsed_url = LocationType::from_str("URL").unwrap();

    // Then: All four variants exist and are distinct from each other
    assert_ne!(
        parsed_gps, parsed_fpath,
        "GPS and file_path should be distinct variants"
    );
    assert_ne!(
        parsed_gps, parsed_docpos,
        "GPS and document_position should be distinct variants"
    );
    assert_ne!(
        parsed_gps, parsed_url,
        "GPS and URL should be distinct variants"
    );
    assert_ne!(
        parsed_fpath, parsed_docpos,
        "file_path and document_position should be distinct variants"
    );
    assert_ne!(
        parsed_fpath, parsed_url,
        "file_path and URL should be distinct variants"
    );
    assert_ne!(
        parsed_docpos, parsed_url,
        "document_position and URL should be distinct variants"
    );
}

#[test]
fn location_type_parses_gps_string_when_input_is_gps() {
    // Given: A string "GPS"
    let input = "GPS";
    // When: LocationType::from_str is called
    let result = LocationType::from_str(input);
    // Then: The result is Ok(LocationType::Gps)
    assert_eq!(result, Ok(LocationType::Gps));
}

#[test]
fn location_type_parses_file_path_string_when_input_is_file_path() {
    // Given: A string "file_path"
    let input = "file_path";
    // When: LocationType::from_str is called
    let result = LocationType::from_str(input);
    // Then: The result is Ok(LocationType::FilePath)
    assert_eq!(result, Ok(LocationType::FilePath));
}

#[test]
fn location_type_parses_document_position_string_when_input_is_document_position() {
    // Given: A string "document_position"
    let input = "document_position";
    // When: LocationType::from_str is called
    let result = LocationType::from_str(input);
    // Then: The result is Ok(LocationType::DocumentPosition)
    assert_eq!(result, Ok(LocationType::DocumentPosition));
}

#[test]
fn location_type_parses_url_string_when_input_is_url() {
    // Given: A string "URL"
    let input = "URL";
    // When: LocationType::from_str is called
    let result = LocationType::from_str(input);
    // Then: The result is Ok(LocationType::Url)
    assert_eq!(result, Ok(LocationType::Url));
}

#[test]
fn location_type_returns_error_when_input_is_invalid() {
    // Given: A string that is not a valid variant
    let input = "invalid";
    // When: LocationType::from_str is called
    let result = LocationType::from_str(input);
    // Then: The result is Err(LocationTypeParseError::UnknownVariant)
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(LocationTypeParseError::UnknownVariant)
    ));
}

#[test]
fn location_type_returns_error_when_input_is_empty_string() {
    // Given: An empty string
    let input = "";
    // When: LocationType::from_str is called
    let result = LocationType::from_str(input);
    // Then: The result is Err(LocationTypeParseError::UnknownVariant)
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(LocationTypeParseError::UnknownVariant)
    ));
}

#[test]
fn location_type_returns_error_when_input_is_lowercase_gps() {
    // Given: A lowercase "gps" string
    let input = "gps";
    // When: LocationType::from_str is called
    let result = LocationType::from_str(input);
    // Then: The result is Err(LocationTypeParseError::UnknownVariant)
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(LocationTypeParseError::UnknownVariant)
    ));
}

#[test]
fn location_type_returns_error_when_input_has_typo() {
    // Given: A string with a typo
    let input = "GPSS";
    // When: LocationType::from_str is called
    let result = LocationType::from_str(input);
    // Then: The result is Err(LocationTypeParseError::UnknownVariant)
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(LocationTypeParseError::UnknownVariant)
    ));
}

// =============================================================================
// AiDocument Newtype Tests
// =============================================================================

#[test]
fn ai_document_accepts_valid_input() {
    // Given: Valid AiDocument creation inputs
    let id = "doc-123".to_string();
    let key = "test-key".to_string();
    let json_payload = r#"{"data":"test"}"#.to_string();
    let location_type = LocationType::Gps;
    let location_data = "37.7749,-122.4194".to_string();
    let created_at = 1700000000i64;

    // When: AiDocument::new is called with valid inputs
    let result = AiDocument::new(
        id.clone(),
        key,
        json_payload,
        location_type,
        location_data,
        created_at,
    );

    // Then: The result is Ok with the created document
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.id(), &id);
}

#[test]
fn ai_document_rejects_empty_id() {
    // Given: An empty id string
    let id = "".to_string();
    let key = "test-key".to_string();
    let json_payload = r#"{"data":"test"}"#.to_string();
    let location_type = LocationType::Gps;
    let location_data = "37.7749,-122.4194".to_string();
    let created_at = 1700000000i64;

    // When: AiDocument::new is called with empty id
    let result = AiDocument::new(
        id,
        key,
        json_payload,
        location_type,
        location_data,
        created_at,
    );

    // Then: The result is Err indicating invalid id
    assert!(result.is_err());
}

#[test]
fn ai_document_rejects_empty_key() {
    // Given: An empty key string
    let id = "doc-123".to_string();
    let key = "".to_string();
    let json_payload = r#"{"data":"test"}"#.to_string();
    let location_type = LocationType::Gps;
    let location_data = "37.7749,-122.4194".to_string();
    let created_at = 1700000000i64;

    // When: AiDocument::new is called with empty key
    let result = AiDocument::new(
        id,
        key,
        json_payload,
        location_type,
        location_data,
        created_at,
    );

    // Then: The result is Err indicating invalid key
    assert!(result.is_err());
}

#[test]
fn ai_document_preserves_all_fields_after_creation() {
    // Given: Valid AiDocument creation inputs
    let id = "doc-456".to_string();
    let key = "my-key".to_string();
    let json_payload = r#"{"data":"value"}"#.to_string();
    let location_type = LocationType::FilePath;
    let location_data = "/path/to/file.txt".to_string();
    let created_at = 1700000001i64;

    // When: AiDocument::new succeeds
    let doc = AiDocument::new(
        id.clone(),
        key.clone(),
        json_payload.clone(),
        location_type.clone(),
        location_data.clone(),
        created_at,
    )
    .unwrap();

    // Then: All fields are preserved
    assert_eq!(doc.id(), &id);
    assert_eq!(doc.key(), &key);
    assert_eq!(doc.json_payload(), &json_payload);
    assert_eq!(doc.location_type(), &location_type);
    assert_eq!(doc.location_data(), &location_data);
    assert_eq!(doc.created_at(), &created_at);
}
