//! Unit tests for execute_apply executor.
//!
//! Behaviors 55–58c, 89.

#![allow(clippy::unwrap_used)]

use std::io::Cursor;
use std::path::{Path, PathBuf};

use super::helpers::*;
use crate::apply::io::execute_apply;
use crate::apply::types::*;

// =========================================================================
// BEHAVIOR 55: execute_apply — Full success pipeline
// =========================================================================

#[test]
fn execute_apply_writes_queued_json_to_stdout_when_all_preconditions_met() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("document.json");
    std::fs::write(&doc_path, valid_document_json(5)).unwrap();

    let proposal_json = valid_proposal_json(5);
    let stdin = Cursor::new(proposal_json.into_bytes());
    let mut writer = Vec::new();
    let cmd = ApplyCommand {
        input_source: ApplySource::Stdin,
    };

    let result = execute_apply(&cmd, stdin, &mut writer, &doc_path);
    assert_eq!(
        result,
        Ok(true),
        "execute_apply must return true (queued): {result:?}"
    );
    let output = String::from_utf8(writer).unwrap();
    assert!(
        output.contains(r#""status":"queued""#),
        "expected 'queued' in output, got: {output}"
    );
    assert!(
        output.contains(r#""change_count""#),
        "expected 'change_count' in output, got: {output}"
    );
    assert!(
        output.contains(r#""base_revision":5"#),
        "expected 'base_revision':5 in output, got: {output}"
    );
}

// =========================================================================
// BEHAVIOR 56: execute_apply — Stale revision
// =========================================================================

#[test]
fn execute_apply_writes_rejected_human_priority_block_when_revision_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("document.json");
    std::fs::write(&doc_path, valid_document_json(5)).unwrap();

    let proposal_json = valid_proposal_json(3);
    let stdin = Cursor::new(proposal_json.into_bytes());
    let mut writer = Vec::new();
    let cmd = ApplyCommand {
        input_source: ApplySource::Stdin,
    };

    let result = execute_apply(&cmd, stdin, &mut writer, &doc_path);

    assert_eq!(
        result,
        Ok(false),
        "execute_apply must return false (rejected) on stale revision: {result:?}"
    );
    let output = String::from_utf8(writer).unwrap();
    assert!(
        output.contains(r#""status":"rejected""#),
        "expected 'rejected' in output, got: {output}"
    );
    assert!(
        output.contains(r#""reason":"Human Priority Block""#),
        "expected 'Human Priority Block' in output, got: {output}"
    );
    assert!(
        output.contains(r#""conflict_details":{"expected_revision":3,"current_revision":5}"#),
        "expected conflict_details in output, got: {output}"
    );
}

// =========================================================================
// BEHAVIOR 57: execute_apply — Schema validation failure
// =========================================================================

#[test]
fn execute_apply_writes_rejected_schema_validation_failed_when_schema_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("document.json");
    std::fs::write(&doc_path, valid_document_json(5)).unwrap();

    let proposal_json = r#"{"base_revision": 5}"#;
    let stdin = Cursor::new(proposal_json.as_bytes());
    let mut writer = Vec::new();
    let cmd = ApplyCommand {
        input_source: ApplySource::Stdin,
    };

    let result = execute_apply(&cmd, stdin, &mut writer, &doc_path);

    assert_eq!(
        result,
        Ok(false),
        "execute_apply must return false (rejected) on schema invalid: {result:?}"
    );
    let output = String::from_utf8(writer).unwrap();
    assert!(
        output.contains(r#""status":"rejected""#),
        "expected 'rejected' in output, got: {output}"
    );
    assert!(
        output.contains(r#""reason":"schema_validation_failed""#),
        "expected 'schema_validation_failed' in output, got: {output}"
    );
    assert!(
        output.contains(r#""validation_issues":["#),
        "expected 'validation_issues' in output, got: {output}"
    );
}

// =========================================================================
// BEHAVIOR 58: execute_apply — Proposal file not found
// =========================================================================

#[test]
fn execute_apply_returns_error_when_proposal_file_not_found() {
    let nonexistent = PathBuf::from("/nonexistent/proposal.json");
    let stdin = Cursor::new(Vec::<u8>::new());
    let mut writer = Vec::new();
    let cmd = ApplyCommand {
        input_source: ApplySource::File(nonexistent.clone()),
    };

    let result = execute_apply(&cmd, stdin, &mut writer, Path::new("."));

    assert_eq!(
        result,
        Err(ApplyCommandError::InputFileNotFound(nonexistent))
    );
    assert!(
        writer.is_empty(),
        "stdout must be empty on error, got: {:?}",
        String::from_utf8_lossy(&writer)
    );
}

// =========================================================================
// BEHAVIOR 58b: execute_apply — Document not found
// =========================================================================

#[test]
fn execute_apply_returns_document_not_found_when_document_path_does_not_exist() {
    let proposal_json = valid_proposal_json(1);
    let stdin = Cursor::new(proposal_json.into_bytes());
    let mut writer = Vec::new();
    let cmd = ApplyCommand {
        input_source: ApplySource::Stdin,
    };
    let nonexistent_doc = PathBuf::from("/nonexistent/doc.json");

    let result = execute_apply(&cmd, stdin, &mut writer, &nonexistent_doc);

    assert!(
        result.is_err(),
        "execute_apply must fail when document cannot be loaded, got: {result:?}"
    );
    assert!(
        writer.is_empty(),
        "stdout must be empty on error, got: {:?}",
        String::from_utf8_lossy(&writer)
    );
}

// =========================================================================
// BEHAVIOR 58c: execute_apply — Output write failure
// =========================================================================

#[test]
fn execute_apply_returns_output_write_failure_when_writer_fails() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("document.json");
    std::fs::write(&doc_path, valid_document_json(1)).unwrap();

    let proposal_json = valid_proposal_json(1);
    let stdin = Cursor::new(proposal_json.into_bytes());
    let writer = AlwaysFailsWriter;
    let cmd = ApplyCommand {
        input_source: ApplySource::Stdin,
    };

    let result = execute_apply(&cmd, stdin, writer, &doc_path);

    match result {
        Err(ApplyCommandError::OutputWriteFailure(msg)) => {
            assert!(
                msg.contains("injected write error"),
                "expected 'injected write error' in message, got: {msg}"
            );
        }
        other => panic!("expected OutputWriteFailure, got: {other:?}"),
    }
}

// =========================================================================
// BEHAVIOR 89: ProposalEmptyChanges reachability
// =========================================================================

#[test]
fn load_proposal_returns_proposal_empty_changes_when_json_has_empty_changes_array() {
    use crate::apply::io::load_proposal;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal.json");
    let json = r#"{
        "base_revision": 1,
        "proposer": "test-agent",
        "proposed_at": 1700000000,
        "summary": "empty changes",
        "changes": []
    }"#;
    std::fs::write(&path, json).unwrap();

    let stdin = Cursor::new(Vec::<u8>::new());
    let result = load_proposal(&ApplySource::File(path), stdin);

    assert_eq!(result, Err(ApplyCommandError::ProposalEmptyChanges));
}
