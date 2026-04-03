#![allow(unused_variables, unused_imports)]

//! Additional adversarial tests for the ai_documents schema.
//!
//! Tests focus on:
//! 1. LocationType Display roundtrip validation
//! 2. Panic/unwrap detection in code paths
//! 3. location_data validation per location_type

use crate::schema_ai_documents::AiDocument;
use crate::schema_ai_documents::LocationType;
use crate::schema_ai_documents::LocationTypeParseError;

// =============================================================================
// Dimension: LocationType Display Roundtrip
// =============================================================================

/// Test: LocationType Display roundtrip should work correctly
/// The Display impl should return a string that can be parsed back
#[test]
fn adversarial_location_type_display_roundtrip_gps() {
    let original = LocationType::Gps;
    let display_str = format!("{}", original);
    assert_eq!(display_str, "GPS", "Display should return 'GPS'");
    let parsed = LocationType::from_str(&display_str);
    assert_eq!(parsed, Ok(original), "GPS should roundtrip through Display");
}

/// Test: FilePath Display roundtrip
#[test]
fn adversarial_location_type_display_roundtrip_filepath() {
    let original = LocationType::FilePath;
    let display_str = format!("{}", original);
    assert_eq!(
        display_str, "file_path",
        "Display should return 'file_path'"
    );
    let parsed = LocationType::from_str(&display_str);
    assert_eq!(
        parsed,
        Ok(original),
        "FilePath should roundtrip through Display"
    );
}

/// Test: DocumentPosition Display roundtrip
#[test]
fn adversarial_location_type_display_roundtrip_docpos() {
    let original = LocationType::DocumentPosition;
    let display_str = format!("{}", original);
    assert_eq!(
        display_str, "document_position",
        "Display should return 'document_position'"
    );
    let parsed = LocationType::from_str(&display_str);
    assert_eq!(
        parsed,
        Ok(original),
        "DocumentPosition should roundtrip through Display"
    );
}

/// Test: Url Display roundtrip
#[test]
fn adversarial_location_type_display_roundtrip_url() {
    let original = LocationType::Url;
    let display_str = format!("{}", original);
    assert_eq!(display_str, "URL", "Display should return 'URL'");
    let parsed = LocationType::from_str(&display_str);
    assert_eq!(parsed, Ok(original), "Url should roundtrip through Display");
}

// =============================================================================
// Dimension: TryFrom Implementation
// =============================================================================

/// Test: TryFrom<&str> should work the same as from_str
#[test]
fn adversarial_try_from_str_gps() {
    let result: Result<LocationType, _> = "GPS".try_into();
    assert_eq!(result, Ok(LocationType::Gps));
}

/// Test: TryFrom<&str> for invalid value
#[test]
fn adversarial_try_from_str_invalid() {
    let result: Result<LocationType, _> = "invalid".try_into();
    assert_eq!(result, Err(LocationTypeParseError::UnknownVariant));
}

// =============================================================================
// Dimension: AiDocument Field Accessors
// =============================================================================

/// Test: All field accessors should work correctly
#[test]
fn adversarial_ai_document_all_accessors_work() {
    let doc = AiDocument::new(
        "doc-accessors".to_string(),
        "test-key".to_string(),
        r#"{"test":true}"#.to_string(),
        LocationType::Url,
        "https://example.com/doc".to_string(),
        1700000000i64,
    )
    .expect("Creating AiDocument should succeed");

    // All accessors should return the correct values
    assert_eq!(doc.id(), "doc-accessors");
    assert_eq!(doc.key(), "test-key");
    assert_eq!(doc.json_payload(), r#"{"test":true}"#);
    assert_eq!(doc.location_type(), &LocationType::Url);
    assert_eq!(doc.location_data(), "https://example.com/doc");
    assert_eq!(doc.created_at(), &1700000000i64);
}

// =============================================================================
// Dimension: Whitespace-only id and key validation
// =============================================================================

/// Test: Whitespace-only id should be rejected
#[test]
fn adversarial_whitespace_id_rejected() {
    let inputs = [" ", "  ", "\t", "\n", "\r\n", " \t \n "];
    for id in inputs {
        let result = AiDocument::new(
            id.to_string(),
            "test-key".to_string(),
            r#"{"data":"test"}"#.to_string(),
            LocationType::Gps,
            "37.7749,-122.4194".to_string(),
            1700000000i64,
        );
        assert!(
            result.is_err(),
            "Whitespace-only id '{}' should be rejected",
            id
        );
    }
}

/// Test: Whitespace-only key should be rejected
#[test]
fn adversarial_whitespace_key_rejected() {
    let inputs = [" ", "  ", "\t", "\n", "\r\n", " \t \n "];
    for key in inputs {
        let result = AiDocument::new(
            "doc-123".to_string(),
            key.to_string(),
            r#"{"data":"test"}"#.to_string(),
            LocationType::Gps,
            "37.7749,-122.4194".to_string(),
            1700000000i64,
        );
        assert!(
            result.is_err(),
            "Whitespace-only key '{}' should be rejected",
            key
        );
    }
}

// =============================================================================
// Dimension: Special Characters in Fields
// =============================================================================

/// Test: Special characters in id field
#[test]
fn adversarial_id_special_characters() {
    // These should all be accepted since id is just a string
    let ids = [
        "doc with spaces",
        "doc\twith\ttabs",
        "doc\nwith\nnewlines",
        "doc|with|pipes",
        "doc;with;semicolons",
        "doc\"with\"quotes",
        "doc'with'quotes",
        "doc<with>angles",
        "doc(with)parens",
        "doc[with]brackets",
        "doc{with}braces",
        "doc$with$dollars",
        "doc`with`backticks",
        "doc\\with\\backs",
        "doc/with/slashes",
        "日本語ドキュメント",
        "emoji🎉",
    ];

    for id in ids {
        let result = AiDocument::new(
            id.to_string(),
            "test-key".to_string(),
            r#"{"data":"test"}"#.to_string(),
            LocationType::Gps,
            "37.7749,-122.4194".to_string(),
            1700000000i64,
        );
        assert!(result.is_ok(), "Id '{}' should be accepted", id);
    }
}

/// Test: Unicode in location_data
#[test]
fn adversarial_location_data_unicode() {
    let location_data_values = [
        "日本語ファイルパス",
        "emoji🎉",
        "Path mit Ümläüten",
        " Chemin avec accents",
        "路径中文",
        "🌍",
    ];

    for location_data in location_data_values {
        let result = AiDocument::new(
            format!("doc-{}", location_data.len()),
            "test-key".to_string(),
            r#"{"data":"test"}"#.to_string(),
            LocationType::FilePath,
            location_data.to_string(),
            1700000000i64,
        );
        assert!(
            result.is_ok(),
            "Unicode location_data '{}' should be accepted",
            location_data
        );
    }
}

// =============================================================================
// Dimension: Created_at edge cases
// =============================================================================

/// Test: Negative created_at (valid since it's just an i64)
#[test]
fn adversarial_created_at_negative() {
    let result = AiDocument::new(
        "doc-negative".to_string(),
        "test-key".to_string(),
        r#"{"data":"test"}"#.to_string(),
        LocationType::Gps,
        "37.7749,-122.4194".to_string(),
        -1i64,
    );
    assert!(result.is_ok(), "Negative created_at should be accepted");
}

/// Test: Zero created_at
#[test]
fn adversarial_created_at_zero() {
    let result = AiDocument::new(
        "doc-zero".to_string(),
        "test-key".to_string(),
        r#"{"data":"test"}"#.to_string(),
        LocationType::Gps,
        "37.7749,-122.4194".to_string(),
        0i64,
    );
    assert!(result.is_ok(), "Zero created_at should be accepted");
}

/// Test: Very large created_at
#[test]
fn adversarial_created_at_max() {
    let result = AiDocument::new(
        "doc-max".to_string(),
        "test-key".to_string(),
        r#"{"data":"test"}"#.to_string(),
        LocationType::Gps,
        "37.7749,-122.4194".to_string(),
        i64::MAX,
    );
    assert!(result.is_ok(), "i64::MAX created_at should be accepted");
}
