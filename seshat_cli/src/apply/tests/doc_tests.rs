//! Unit tests for load_current_document I/O layer.
//!
//! Behaviors 48–54d.

#![allow(clippy::unwrap_used)]

use diagram_models::document::types::Revision;
use std::io::Cursor;
use std::path::PathBuf;

use super::helpers::*;
use crate::apply::io::*;
use crate::apply::types::*;

// =========================================================================
// BEHAVIOR 48: load_current_document — Valid file
// =========================================================================

#[test]
fn load_current_document_returns_diagram_document_when_file_is_valid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.json");
    std::fs::write(&path, valid_document_json(5)).unwrap();

    let result = load_current_document(&path);

    assert!(
        result.is_ok(),
        "load_current_document must succeed: {result:?}"
    );
    let doc = result.unwrap();
    assert_eq!(doc.revision, Revision::new(5));
}

// =========================================================================
// BEHAVIOR 49: load_current_document — Not found
// =========================================================================

#[test]
fn load_current_document_returns_document_not_found_when_path_does_not_exist() {
    let nonexistent = PathBuf::from("/nonexistent/path/document.json");
    let result = load_current_document(&nonexistent);

    assert_eq!(
        result,
        Err(ApplyCommandError::DocumentNotFound(nonexistent))
    );
}

// =========================================================================
// BEHAVIOR 50: load_current_document — I/O error (directory)
// =========================================================================

#[test]
fn load_current_document_returns_document_io_error_when_path_is_directory() {
    let dir = tempfile::tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();
    let result = load_current_document(&dir_path);

    match result {
        Err(ApplyCommandError::DocumentIoError(msg)) => {
            let lower = msg.to_lowercase();
            assert!(
                lower.contains("directory"),
                "expected 'directory' in error message, got: {msg}"
            );
        }
        other => panic!("expected DocumentIoError, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 51: load_current_document — Invalid UTF-8
// =========================================================================

#[test]
fn load_current_document_returns_document_invalid_utf8_when_bytes_not_utf8() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.bin");
    std::fs::write(&path, [0xFF, 0xFE]).unwrap();

    let result = load_current_document(&path);

    assert_eq!(result, Err(ApplyCommandError::DocumentInvalidUtf8));
}

// =========================================================================
// BEHAVIOR 52: load_current_document — Empty
// =========================================================================

#[test]
fn load_current_document_returns_document_empty_when_file_is_zero_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.json");
    std::fs::write(&path, "").unwrap();

    let result = load_current_document(&path);

    assert_eq!(result, Err(ApplyCommandError::DocumentEmpty));
}

// =========================================================================
// BEHAVIOR 53: load_current_document — Malformed JSON
// =========================================================================

#[test]
fn load_current_document_returns_document_json_malformed_when_json_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.json");
    std::fs::write(&path, "not json at all").unwrap();

    let result = load_current_document(&path);

    match result {
        Err(ApplyCommandError::DocumentJsonMalformed(msg)) => {
            assert!(
                msg.contains("line") && msg.contains("column"),
                "expected 'line' AND 'column' in error, got: {msg}"
            );
        }
        other => panic!("expected DocumentJsonMalformed, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 54: load_current_document — Schema invalid
// =========================================================================

#[test]
fn load_current_document_returns_document_schema_invalid_when_json_has_unknown_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.json");
    let json_with_unknown = r#"{
            "version": 2,
            "revision": 1,
            "document": {"nodes": {}, "edges": {}},
            "editor_state": {"camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0},
            "unknown_field": true
        }"#;
    std::fs::write(&path, json_with_unknown).unwrap();

    let result = load_current_document(&path);

    match result {
        Err(ApplyCommandError::DocumentSchemaInvalid(msg)) => {
            assert!(
                msg.contains("unknown field"),
                "expected 'unknown field' in error, got: {msg}"
            );
        }
        other => panic!("expected DocumentSchemaInvalid, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 54b: load_current_document — Permission-denied
// =========================================================================

#[cfg(unix)]
#[test]
fn load_current_document_returns_document_not_found_when_path_is_permission_denied() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.json");
    std::fs::write(&path, "{}").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

    let result = load_current_document(&path);

    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert_eq!(result, Err(ApplyCommandError::DocumentNotFound(path)));
}

// =========================================================================
// BEHAVIOR 54c: load_current_document — File exceeds configurable max_bytes
// =========================================================================

#[test]
fn load_current_document_returns_document_json_malformed_when_file_exceeds_configurable_max_bytes()
{
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.json");
    let small_limit: u64 = 256;
    let valid_prefix = r#"{"version": 2, "revision": 1, "document": {"nodes": {}, "edges": {}}, "editor_state": {"camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0}, "#;
    let padding = " ".repeat((small_limit as usize) + 1024 - valid_prefix.len());
    std::fs::write(&path, format!("{valid_prefix}{padding}")).unwrap();

    let result = load_current_document_with_limit(&path, small_limit);

    match result {
        Err(ApplyCommandError::DocumentJsonMalformed(msg)) => {
            assert!(
                msg.contains("exceeds maximum size"),
                "expected 'exceeds maximum size' in error, got: {msg}"
            );
        }
        other => panic!("expected DocumentJsonMalformed, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 54d: load_current_document — Wrong-type JSON
// =========================================================================

#[test]
fn load_current_document_returns_document_schema_invalid_when_field_has_wrong_type() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.json");
    std::fs::write(&path, r#"{"version": 1, "revision": "not-a-number"}"#).unwrap();

    let result = load_current_document(&path);

    match result {
        Err(ApplyCommandError::DocumentSchemaInvalid(msg)) => {
            let lower = msg.to_lowercase();
            assert!(
                lower.contains("invalid type"),
                "expected 'invalid type' in error, got: {msg}"
            );
        }
        other => panic!("expected DocumentSchemaInvalid, got: {other:?}"),
    }
}
