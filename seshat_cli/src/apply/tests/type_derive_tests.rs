//! Unit tests for apply type derives and equality.
//!
//! Behaviors 1–13.

#![allow(clippy::unwrap_used)]

use diagram_models::document::types::Revision;
use diagram_models::validation::{ValidationCode, ValidationIssue};
use std::path::PathBuf;

use crate::apply::types::*;

// =========================================================================
// BEHAVIOR 1: ApplySource::File stores path
// =========================================================================

#[test]
fn apply_source_file_stores_path_when_constructed_with_pathbuf() {
    let path = PathBuf::from("/tmp/proposal.json");
    let source = ApplySource::File(path.clone());
    assert_eq!(
        source,
        ApplySource::File(PathBuf::from("/tmp/proposal.json"))
    );
}

// =========================================================================
// BEHAVIOR 2: ApplySource::Stdin equality
// =========================================================================

#[test]
fn apply_source_stdin_equals_itself() {
    assert_eq!(ApplySource::Stdin, ApplySource::Stdin);
}

// =========================================================================
// BEHAVIOR 3: ApplyCommand clone equality
// =========================================================================

#[test]
fn apply_command_clone_produces_equal_value() {
    let cmd = ApplyCommand {
        input_source: ApplySource::File(PathBuf::from("/a/b.json")),
    };
    let cloned = cmd.clone();
    assert_eq!(cloned, cmd);
}

// =========================================================================
// BEHAVIOR 4: ApplyCommand debug output
// =========================================================================

#[test]
fn apply_command_debug_output_contains_type_and_variant_names() {
    let cmd = ApplyCommand {
        input_source: ApplySource::Stdin,
    };
    let output = format!("{cmd:?}");
    assert!(
        output.contains("ApplyCommand"),
        "debug output must contain 'ApplyCommand', got: {output:?}"
    );
    assert!(
        output.contains("Stdin"),
        "debug output must contain 'Stdin', got: {output:?}"
    );
}

// =========================================================================
// BEHAVIOR 5: ApplyStatus::Queued JSON serialization
// =========================================================================

#[test]
fn apply_status_queued_serializes_to_json_with_status_tag_and_fields() {
    let status = ApplyStatus::Queued {
        proposal_id: "prop-abc123".to_string(),
        change_count: 5,
        base_revision: 42,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(
        json.contains(r#""status":"queued""#),
        "missing status tag: {json}"
    );
    assert!(
        json.contains(r#""proposal_id":"prop-abc123""#),
        "missing proposal_id: {json}"
    );
    assert!(
        json.contains(r#""change_count":5"#),
        "missing change_count: {json}"
    );
    assert!(
        json.contains(r#""base_revision":42"#),
        "missing base_revision: {json}"
    );
}

// =========================================================================
// BEHAVIOR 6: ApplyStatus::Rejected JSON serialization
// =========================================================================

#[test]
fn apply_status_rejected_serializes_to_json_with_reason_and_optional_fields() {
    let status = ApplyStatus::Rejected {
        reason: RejectionReasonCode::StaleRevision,
        conflict_details: Some(ConflictDetails {
            expected_revision: Revision::new(42),
            current_revision: Revision::new(44),
        }),
        validation_issues: vec![],
        hint: Some("re-fetch".to_string()),
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(
        json.contains(r#""status":"rejected""#),
        "missing status tag: {json}"
    );
    assert!(
        json.contains(r#""reason":"Human Priority Block""#),
        "missing reason: {json}"
    );
    assert!(
        json.contains(r#""conflict_details":{"expected_revision":42,"current_revision":44}"#),
        "missing conflict_details: {json}"
    );
    assert!(
        json.contains(r#""hint":"re-fetch""#),
        "missing hint: {json}"
    );
}

// =========================================================================
// BEHAVIOR 6b: ApplyStatus::Rejected omits None fields
// =========================================================================

#[test]
fn apply_status_rejected_omits_conflict_details_when_none() {
    let status = ApplyStatus::Rejected {
        reason: RejectionReasonCode::SchemaInvalid,
        conflict_details: None,
        validation_issues: vec![],
        hint: None,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(
        !json.contains("conflict_details"),
        "JSON must not contain 'conflict_details' when None, got: {json}"
    );
    assert!(
        !json.contains("hint"),
        "JSON must not contain 'hint' when None, got: {json}"
    );
}

// =========================================================================
// BEHAVIOR 6c: ApplyStatus::Rejected omits empty validation_issues
// =========================================================================

#[test]
fn apply_status_rejected_omits_validation_issues_when_empty() {
    let status = ApplyStatus::Rejected {
        reason: RejectionReasonCode::StaleRevision,
        conflict_details: Some(ConflictDetails {
            expected_revision: Revision::new(1),
            current_revision: Revision::new(2),
        }),
        validation_issues: vec![],
        hint: None,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(
        !json.contains("validation_issues"),
        "JSON must not contain 'validation_issues' when empty, got: {json}"
    );
}

// =========================================================================
// BEHAVIOR 7: ConflictDetails JSON serialization
// =========================================================================

#[test]
fn conflict_details_serializes_to_json_with_revision_fields() {
    let details = ConflictDetails {
        expected_revision: Revision::new(42),
        current_revision: Revision::new(44),
    };
    let json = serde_json::to_string(&details).unwrap();
    assert!(
        json.contains(r#""expected_revision":42"#),
        "missing expected_revision: {json}"
    );
    assert!(
        json.contains(r#""current_revision":44"#),
        "missing current_revision: {json}"
    );
}

// =========================================================================
// BEHAVIORS 8-13: ApplyOutcome / RejectionReason derives
// =========================================================================

#[test]
fn apply_outcome_queued_equals_identical_instance() {
    let outcome = ApplyOutcome::Queued {
        proposal_id: "prop-test-1234".to_string(),
        change_count: 3,
        base_revision: 5,
    };
    assert_eq!(outcome.clone(), outcome);
}

#[test]
fn apply_outcome_rejected_equals_identical_instance() {
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::EmptyChanges,
    };
    assert_eq!(outcome.clone(), outcome);
}

#[test]
fn rejection_reason_stale_revision_equals_identical_instance() {
    let reason = RejectionReason::StaleRevision {
        expected: Revision::new(42),
        current: Revision::new(44),
    };
    assert_eq!(reason.clone(), reason);
}

#[test]
fn rejection_reason_schema_invalid_equals_identical_instance() {
    let issue = ValidationIssue::error(ValidationCode::SCHEMA, "test issue", None);
    let reason = RejectionReason::SchemaInvalid {
        issues: vec![issue],
    };
    assert_eq!(reason.clone(), reason);
}

#[test]
fn rejection_reason_empty_changes_equals_identical_instance() {
    let reason = RejectionReason::EmptyChanges;
    assert_eq!(reason.clone(), reason);
}

#[test]
fn rejection_reason_invalid_proposer_equals_identical_instance() {
    let reason = RejectionReason::InvalidProposer;
    assert_eq!(reason.clone(), reason);
}
