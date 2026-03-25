//! Unit tests for build_apply_status and serialize_apply_status.
//!
//! Behaviors 28–39.

#![allow(clippy::unwrap_used)]

use diagram_models::document::types::Revision;
use diagram_models::validation::{ValidationCode, ValidationIssue};

use crate::apply::calc::*;
use crate::apply::types::*;

// BEHAVIOR 28: build_apply_status — Queued

#[test]
fn build_apply_status_returns_queued_when_outcome_is_queued() {
    let outcome = ApplyOutcome::Queued {
        proposal_id: "prop-abc123def456".to_string(),
        change_count: 5,
        base_revision: 42,
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Queued {
            proposal_id,
            change_count,
            base_revision,
        } => {
            assert_eq!(proposal_id, "prop-abc123def456");
            assert_eq!(change_count, 5);
            assert_eq!(base_revision, 42);
        }
        ApplyStatus::Rejected { .. } => {
            panic!("expected Queued, got Rejected");
        }
    }
}

// BEHAVIOR 29: build_apply_status — StaleRevision rejection

#[test]
fn build_apply_status_returns_rejected_human_priority_block_when_stale_revision() {
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::StaleRevision {
            expected: Revision::new(42),
            current: Revision::new(44),
        },
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Rejected {
            reason,
            conflict_details,
            ..
        } => {
            assert_eq!(reason, RejectionReasonCode::StaleRevision);
            assert_eq!(
                conflict_details,
                Some(ConflictDetails {
                    expected_revision: Revision::new(42),
                    current_revision: Revision::new(44),
                })
            );
        }
        ApplyStatus::Queued { .. } => {
            panic!("expected Rejected, got Queued");
        }
    }
}

// BEHAVIOR 30: build_apply_status — SchemaInvalid rejection

#[test]
fn build_apply_status_returns_rejected_schema_validation_failed_when_schema_invalid() {
    let issue = ValidationIssue::error(
        ValidationCode::SCHEMA,
        "Missing required field 'base_revision'",
        Some("base_revision".to_string()),
    );
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::SchemaInvalid {
            issues: vec![issue.clone()],
        },
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Rejected {
            reason,
            validation_issues,
            conflict_details,
            ..
        } => {
            assert_eq!(reason, RejectionReasonCode::SchemaInvalid);
            assert_eq!(validation_issues, vec![issue]);
            assert_eq!(conflict_details, None);
        }
        ApplyStatus::Queued { .. } => {
            panic!("expected Rejected, got Queued");
        }
    }
}

// BEHAVIOR 31: build_apply_status — EmptyChanges rejection

#[test]
fn build_apply_status_returns_rejected_with_reason_empty_changes_when_empty_changes() {
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::EmptyChanges,
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Rejected { reason, .. } => {
            assert_eq!(reason, RejectionReasonCode::EmptyChanges);
        }
        ApplyStatus::Queued { .. } => {
            panic!("expected Rejected, got Queued");
        }
    }
}

// BEHAVIOR 32: build_apply_status — InvalidProposer rejection

#[test]
fn build_apply_status_returns_rejected_with_reason_invalid_proposer_when_invalid_proposer() {
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::InvalidProposer,
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Rejected { reason, .. } => {
            assert_eq!(reason, RejectionReasonCode::InvalidProposer);
        }
        ApplyStatus::Queued { .. } => {
            panic!("expected Rejected, got Queued");
        }
    }
}

// BEHAVIOR 33: build_apply_status — Hint on stale

#[test]
fn build_apply_status_includes_hint_on_stale_revision_rejection() {
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::StaleRevision {
            expected: Revision::new(1),
            current: Revision::new(2),
        },
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Rejected { hint, .. } => {
            assert!(hint.is_some(), "hint must be Some on StaleRevision");
            let hint_val = hint.as_ref().unwrap();
            assert!(
                hint_val.contains("seshat show"),
                "hint must contain 'seshat show', got: {hint_val}"
            );
        }
        ApplyStatus::Queued { .. } => {
            panic!("expected Rejected, got Queued");
        }
    }
}

// BEHAVIOR 34: build_apply_status — Issues on schema

#[test]
fn build_apply_status_includes_validation_issues_on_schema_invalid_rejection() {
    let issues = vec![ValidationIssue::error(
        ValidationCode::SCHEMA,
        "test issue".to_string(),
        None,
    )];
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::SchemaInvalid {
            issues: issues.clone(),
        },
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Rejected {
            validation_issues, ..
        } => {
            assert_eq!(validation_issues, issues);
        }
        ApplyStatus::Queued { .. } => {
            panic!("expected Rejected, got Queued");
        }
    }
}

// BEHAVIOR 35: build_apply_status — No conflict_details on non-stale

#[test]
fn build_apply_status_omits_conflict_details_on_non_stale_rejection() {
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::SchemaInvalid { issues: vec![] },
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Rejected {
            conflict_details, ..
        } => {
            assert_eq!(conflict_details, None);
        }
        ApplyStatus::Queued { .. } => {
            panic!("expected Rejected, got Queued");
        }
    }
}

// BEHAVIOR 36: build_apply_status — No hint on non-stale

#[test]
fn build_apply_status_omits_hint_on_non_stale_rejection() {
    let outcome = ApplyOutcome::Rejected {
        reason: RejectionReason::SchemaInvalid { issues: vec![] },
    };
    let status = build_apply_status(outcome);
    match status {
        ApplyStatus::Rejected { hint, .. } => {
            assert_eq!(hint, None);
        }
        ApplyStatus::Queued { .. } => {
            panic!("expected Rejected, got Queued");
        }
    }
}

// BEHAVIOR 37: serialize_apply_status — Queued

#[test]
fn serialize_apply_status_returns_json_string_for_queued_status() {
    let status = ApplyStatus::Queued {
        proposal_id: "prop-abc123".to_string(),
        change_count: 5,
        base_revision: 42,
    };
    let result = serialize_apply_status(&status);
    assert!(result.is_ok(), "serialize must succeed: {result:?}");
    let json = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["status"], "queued");
}

// BEHAVIOR 38: serialize_apply_status — Rejected

#[test]
fn serialize_apply_status_returns_json_string_for_rejected_status() {
    let status = ApplyStatus::Rejected {
        reason: RejectionReasonCode::StaleRevision,
        conflict_details: Some(ConflictDetails {
            expected_revision: Revision::new(1),
            current_revision: Revision::new(3),
        }),
        validation_issues: vec![],
        hint: Some("re-fetch".to_string()),
    };
    let result = serialize_apply_status(&status);
    assert!(result.is_ok(), "serialize must succeed: {result:?}");
    let json = result.unwrap();
    assert!(
        json.contains(r#""status":"rejected""#),
        "missing status: {json}"
    );
    assert!(
        json.contains(r#""reason":"Human Priority Block""#),
        "missing reason: {json}"
    );
}

// BEHAVIOR 39: serialize_apply_status — Round-trip parseable

#[test]
fn serialize_apply_status_output_is_valid_json_round_trip_parseable() {
    let status = ApplyStatus::Queued {
        proposal_id: "prop-roundtrip-42".to_string(),
        change_count: 7,
        base_revision: 99,
    };
    let result = serialize_apply_status(&status);
    assert!(result.is_ok(), "serialize must succeed: {result:?}");
    let json = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["status"], "queued");
    assert_eq!(parsed["change_count"], 7);
    assert_eq!(parsed["base_revision"], 99);
}
