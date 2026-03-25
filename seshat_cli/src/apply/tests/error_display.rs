//! Unit tests for ApplyCommandError display output.
//!
//! Behaviors 59–75.
#![allow(clippy::unwrap_used)]
use diagram_models::document::types::Revision;
use diagram_models::validation::{ValidationCode, ValidationIssue};
use std::path::PathBuf;
use crate::apply::types::ApplyCommandError;
// BEHAVIOR 59: ApplyCommandError::InputFileNotFound display
#[test]
fn apply_command_error_input_file_not_found_display_starts_with_error_apply_prefix_and_contains_path(
) {
    let err = ApplyCommandError::InputFileNotFound(PathBuf::from("/tmp/missing.json"));
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix 'error: apply: ', got: {output:?}"
    );
    assert!(
        output.contains("/tmp/missing.json"),
        "expected path in output, got: {output:?}"
    );
}
// BEHAVIOR 60: ApplyCommandError::InputIoError display
#[test]
fn apply_command_error_input_io_error_display_starts_with_error_apply_prefix_and_contains_payload()
{
    let err = ApplyCommandError::InputIoError("disk fail".to_string());
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("disk fail"),
        "expected payload in output, got: {output:?}"
    );
}
// BEHAVIOR 61: ApplyCommandError::InputInvalidUtf8 display
#[test]
fn apply_command_error_input_invalid_utf8_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::InputInvalidUtf8;
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    let lower = output.to_lowercase();
    assert!(
        lower.contains("utf-8") || lower.contains("utf8"),
        "expected utf-8 mention, got: {output:?}"
    );
}
// BEHAVIOR 62: ApplyCommandError::InputEmpty display
#[test]
fn apply_command_error_input_empty_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::InputEmpty;
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("empty"),
        "expected 'empty' in output, got: {output:?}"
    );
}
// BEHAVIOR 63: ApplyCommandError::ProposalJsonMalformed display
#[test]
fn apply_command_error_proposal_json_malformed_display_starts_with_error_apply_prefix_and_contains_payload(
) {
    let err = ApplyCommandError::ProposalJsonMalformed("parse error at line 3".to_string());
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("parse error at line 3"),
        "expected payload in output, got: {output:?}"
    );
}
// BEHAVIOR 64: ApplyCommandError::ProposalSchemaInvalid display
#[test]
fn apply_command_error_proposal_schema_invalid_display_starts_with_error_apply_prefix_and_mentions_issue_count(
) {
    let issues = vec![
        ValidationIssue::error(ValidationCode::SCHEMA, "issue 1", None),
        ValidationIssue::error(ValidationCode::SCHEMA, "issue 2", None),
    ];
    let err = ApplyCommandError::ProposalSchemaInvalid { issues };
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("2 issues"),
        "expected '2 issues' in output, got: {output:?}"
    );
}
// BEHAVIOR 65: ApplyCommandError::ProposalEmptyChanges display
#[test]
fn apply_command_error_proposal_empty_changes_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::ProposalEmptyChanges;
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("empty") || output.contains("changes"),
        "expected 'empty' or 'changes' in output, got: {output:?}"
    );
}
// BEHAVIOR 66: ApplyCommandError::ProposalInvalidProposer display
#[test]
fn apply_command_error_proposal_invalid_proposer_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::ProposalInvalidProposer;
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("proposer"),
        "expected 'proposer' in output, got: {output:?}"
    );
}
// BEHAVIOR 67: ApplyCommandError::DocumentNotFound display
#[test]
fn apply_command_error_document_not_found_display_starts_with_error_apply_prefix_and_contains_path()
{
    let err = ApplyCommandError::DocumentNotFound(PathBuf::from("/tmp/doc.json"));
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("/tmp/doc.json"),
        "expected path in output, got: {output:?}"
    );
}
// BEHAVIOR 68: ApplyCommandError::DocumentIoError display
#[test]
fn apply_command_error_document_io_error_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::DocumentIoError("read error".to_string());
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("read error"),
        "expected payload in output, got: {output:?}"
    );
}
// BEHAVIOR 69: ApplyCommandError::DocumentInvalidUtf8 display
#[test]
fn apply_command_error_document_invalid_utf8_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::DocumentInvalidUtf8;
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    let lower = output.to_lowercase();
    assert!(
        lower.contains("utf-8") || lower.contains("utf8"),
        "expected utf-8 mention, got: {output:?}"
    );
}
// BEHAVIOR 70: ApplyCommandError::DocumentEmpty display
#[test]
fn apply_command_error_document_empty_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::DocumentEmpty;
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("empty"),
        "expected 'empty' in output, got: {output:?}"
    );
}
// BEHAVIOR 71: ApplyCommandError::DocumentJsonMalformed display
#[test]
fn apply_command_error_document_json_malformed_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::DocumentJsonMalformed("json error".to_string());
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("json error"),
        "expected payload in output, got: {output:?}"
    );
}
// BEHAVIOR 72: ApplyCommandError::DocumentSchemaInvalid display
#[test]
fn apply_command_error_document_schema_invalid_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::DocumentSchemaInvalid("unknown field".to_string());
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("unknown field"),
        "expected payload in output, got: {output:?}"
    );
}
// BEHAVIOR 73: ApplyCommandError::StaleRevision display
#[test]
fn apply_command_error_stale_revision_display_starts_with_error_apply_prefix_and_contains_both_revisions(
) {
    let err = ApplyCommandError::StaleRevision {
        expected: Revision::new(42),
        current: Revision::new(44),
    };
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("expected") && output.contains("current"),
        "expected 'expected' AND 'current' in output, got: {output:?}"
    );
    assert!(
        output.contains("42") && output.contains("44"),
        "expected revision values in output, got: {output:?}"
    );
}
// BEHAVIOR 74: ApplyCommandError::OutputWriteFailure display
#[test]
fn apply_command_error_output_write_failure_display_starts_with_error_apply_prefix() {
    let err = ApplyCommandError::OutputWriteFailure("write fail".to_string());
    let output = err.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix, got: {output:?}"
    );
    assert!(
        output.contains("write fail"),
        "expected payload in output, got: {output:?}"
    );
}
// BEHAVIOR 75: ExecutionError::Apply delegation
#[test]
fn execution_error_apply_variant_delegates_display_to_apply_command_error() {
    let e = crate::error::ExecutionError::Apply(ApplyCommandError::InputEmpty);
    let output = e.to_string();
    assert!(
        output.starts_with("error: apply: "),
        "expected prefix 'error: apply: ', got: {output:?}"
    );
    assert!(
        output.contains("empty"),
        "expected 'empty' in output, got: {output:?}"
    );
}
