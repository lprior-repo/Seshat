//! Unit tests for load_proposal I/O layer.
//!
//! Behaviors 40–47c.

#![allow(clippy::unwrap_used)]

use diagram_models::document::types::{AuthorId, Revision};
use std::io::Cursor;
use std::path::PathBuf;

use super::helpers::*;
use crate::apply::io::*;
use crate::apply::types::*;

// =========================================================================
// BEHAVIOR 40: load_proposal — Valid file
// =========================================================================

#[test]
fn load_proposal_returns_proposed_changes_when_file_contains_valid_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.json");
    std::fs::write(&path, valid_proposal_json(1)).unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(path.clone()), stdin);

    assert!(result.is_ok(), "load_proposal must succeed: {result:?}");
    let proposal = result.unwrap();
    assert_eq!(proposal.base_revision, Revision::new(1));
    assert_eq!(proposal.changes.len(), 3);
}

// =========================================================================
// BEHAVIOR 41: load_proposal — File not found
// =========================================================================

#[test]
fn load_proposal_returns_input_file_not_found_when_path_does_not_exist() {
    let nonexistent = PathBuf::from("/nonexistent/path/proposal.json");
    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(nonexistent.clone()), stdin);

    assert_eq!(
        result,
        Err(ApplyCommandError::InputFileNotFound(nonexistent))
    );
}

// =========================================================================
// BEHAVIOR 42: load_proposal — I/O error (directory)
// =========================================================================

#[test]
fn load_proposal_returns_input_io_error_when_directory_provided_as_path() {
    let dir = tempfile::tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();
    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(dir_path), stdin);

    match result {
        Err(ApplyCommandError::InputIoError(msg)) => {
            let lower = msg.to_lowercase();
            assert!(
                lower.contains("directory") || lower.contains("is a directory"),
                "expected 'directory' in error message, got: {msg}"
            );
        }
        other => panic!("expected InputIoError, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 43: load_proposal — Invalid UTF-8
// =========================================================================

#[test]
fn load_proposal_returns_input_invalid_utf8_when_bytes_not_valid_utf8() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.bin");
    std::fs::write(&path, [0xFF, 0xFE, 0x80]).unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(path), stdin);

    assert_eq!(result, Err(ApplyCommandError::InputInvalidUtf8));
}

// =========================================================================
// BEHAVIOR 44: load_proposal — Empty input
// =========================================================================

#[test]
fn load_proposal_returns_input_empty_when_file_is_zero_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.json");
    std::fs::write(&path, "").unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(path), stdin);

    assert_eq!(result, Err(ApplyCommandError::InputEmpty));
}

// =========================================================================
// BEHAVIOR 44b: load_proposal — Whitespace-only input
// =========================================================================

#[test]
fn load_proposal_returns_input_empty_when_file_contains_only_whitespace() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.json");
    std::fs::write(&path, "\n\n  \t\n").unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(path), stdin);

    assert_eq!(result, Err(ApplyCommandError::InputEmpty));
}

// =========================================================================
// BEHAVIOR 45: load_proposal — Malformed JSON
// =========================================================================

#[test]
fn load_proposal_returns_proposal_json_malformed_when_json_is_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.json");
    std::fs::write(&path, "{broken json").unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(path), stdin);

    match result {
        Err(ApplyCommandError::ProposalJsonMalformed(msg)) => {
            assert!(
                msg.contains("line") && msg.contains("column"),
                "expected 'line' AND 'column' in error, got: {msg}"
            );
        }
        other => panic!("expected ProposalJsonMalformed, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 45b: load_proposal — Malformed (wrong type)
// =========================================================================

#[test]
fn load_proposal_returns_proposal_schema_invalid_when_field_has_wrong_type() {
    let json = r#"{"base_revision": "not-a-number", "proposer": "ai", "changes": [{"change_type": "add_node", "id": "n1", "label": "N", "kind": "process"}]}"#;
    let cursor = std::io::Cursor::new(json.as_bytes());
    let result = load_proposal(&ApplySource::Stdin, cursor);
    match result {
        Err(ApplyCommandError::ProposalSchemaInvalid { issues }) => {
            assert!(
                !issues.is_empty(),
                "expected at least 1 schema issue, got: {issues:?}"
            );
        }
        other => panic!("Expected ProposalSchemaInvalid, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 46: load_proposal — Schema invalid
// =========================================================================

#[test]
fn load_proposal_returns_proposal_schema_invalid_when_json_missing_required_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.json");
    std::fs::write(&path, r#"{"version": 1}"#).unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(path), stdin);

    match result {
        Err(ApplyCommandError::ProposalSchemaInvalid { issues }) => {
            assert!(
                !issues.is_empty(),
                "expected at least 1 issue, got: {issues:?}"
            );
        }
        other => panic!("expected ProposalSchemaInvalid, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 47: load_proposal — Stdin
// =========================================================================

#[test]
fn load_proposal_reads_from_stdin_when_source_is_stdin() {
    let json = valid_proposal_json(1);
    let stdin = Cursor::new(json.into_bytes());
    let result = load_proposal(&ApplySource::Stdin, stdin);

    assert!(
        result.is_ok(),
        "load_proposal from stdin must succeed: {result:?}"
    );
    let proposal = result.unwrap();
    assert_eq!(proposal.base_revision, Revision::new(1));
    assert_eq!(proposal.changes.len(), 3);
    assert_eq!(proposal.proposer, AuthorId::new("test-agent".to_string()));
}

// =========================================================================
// BEHAVIOR 47b: load_proposal — Permission-denied
// =========================================================================

#[cfg(unix)]
#[test]
fn load_proposal_returns_input_file_not_found_when_path_is_permission_denied() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.json");
    std::fs::write(&path, "{}").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(path.clone()), stdin);

    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert_eq!(result, Err(ApplyCommandError::InputFileNotFound(path)));
}

// =========================================================================
// BEHAVIOR 47c: load_proposal — File exceeds configurable max_bytes
// =========================================================================

#[test]
fn load_proposal_returns_proposal_json_malformed_when_file_exceeds_configurable_max_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.json");
    let small_limit: u64 = 256;
    let large_content = "x".repeat((small_limit as usize) + 1024);
    std::fs::write(&path, &large_content).unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal_with_limit(&ApplySource::File(path), stdin, small_limit);

    match result {
        Err(ApplyCommandError::ProposalJsonMalformed(msg)) => {
            assert!(
                msg.contains("exceeds maximum size"),
                "expected 'exceeds maximum size' in error, got: {msg}"
            );
        }
        other => panic!("expected ProposalJsonMalformed, got: {other:?}"),
    }
}
