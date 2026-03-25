//! Unit tests for map_apply_subcommand, validate_proposal_schema, check_revision_match.
//!
//! Behaviors 14–27.

#![allow(clippy::unwrap_used)]

use diagram_models::document::types::{AuthorId, Revision};
use std::path::PathBuf;

use super::helpers::*;
use crate::apply::calc::*;
use crate::apply::types::*;

// =========================================================================
// BEHAVIOR 14: map_apply_subcommand — File path
// =========================================================================

#[test]
fn map_apply_subcommand_returns_file_source_when_path_is_some() {
    let path = PathBuf::from("/some/path.json");
    let result = map_apply_subcommand(Some(path.clone()));
    assert_eq!(
        result,
        ApplyCommand {
            input_source: ApplySource::File(path)
        }
    );
}

// =========================================================================
// BEHAVIOR 15: map_apply_subcommand — Stdin
// =========================================================================

#[test]
fn map_apply_subcommand_returns_stdin_source_when_path_is_none() {
    let result = map_apply_subcommand(None);
    assert_eq!(
        result,
        ApplyCommand {
            input_source: ApplySource::Stdin
        }
    );
}

// =========================================================================
// BEHAVIOR 16: map_apply_subcommand — Relative path
// =========================================================================

#[test]
fn map_apply_subcommand_returns_file_source_when_path_is_relative() {
    let path = PathBuf::from("rel/path.json");
    let result = map_apply_subcommand(Some(path.clone()));
    assert_eq!(
        result,
        ApplyCommand {
            input_source: ApplySource::File(path)
        }
    );
}

// =========================================================================
// BEHAVIOR 17: map_apply_subcommand — Never panics
// =========================================================================

#[test]
fn map_apply_subcommand_never_panics_for_any_option_pathbuf() {
    let result_none = map_apply_subcommand(None);
    assert_eq!(result_none.input_source, ApplySource::Stdin);

    let result_some = map_apply_subcommand(Some(PathBuf::from("/normal/path")));
    assert!(matches!(result_some.input_source, ApplySource::File(_)));

    let result_empty = map_apply_subcommand(Some(PathBuf::from("")));
    assert!(matches!(result_empty.input_source, ApplySource::File(_)));
}

// =========================================================================
// BEHAVIOR 17b: map_apply_subcommand — Root path
// =========================================================================

#[test]
fn map_apply_subcommand_returns_file_source_when_path_is_root() {
    let path = PathBuf::from("/");
    let result = map_apply_subcommand(Some(path.clone()));
    assert_eq!(
        result,
        ApplyCommand {
            input_source: ApplySource::File(path)
        }
    );
}

// =========================================================================
// BEHAVIOR 18: validate_proposal_schema — Valid proposal
// =========================================================================

#[test]
fn validate_proposal_schema_returns_empty_vec_when_proposal_is_structurally_valid() {
    let proposal = valid_proposal();
    let issues = validate_proposal_schema(&proposal);
    assert_eq!(issues, vec![]);
}

// =========================================================================
// BEHAVIOR 19: validate_proposal_schema — base_revision=0 is valid
// =========================================================================

#[test]
fn validate_proposal_schema_returns_issue_when_base_revision_missing() {
    let proposal = ApplyProposal {
        base_revision: Revision::new(0),
        ..valid_proposal()
    };
    let issues = validate_proposal_schema(&proposal);
    assert!(
        !issues.iter().any(|i| i.message.contains("base_revision")),
        "base_revision=0 must not produce an issue: {issues:?}"
    );
}

// =========================================================================
// BEHAVIOR 20: validate_proposal_schema — Empty proposer
// =========================================================================

#[test]
fn validate_proposal_schema_returns_issue_when_proposer_is_empty() {
    let proposal = ApplyProposal {
        proposer: AuthorId::new(String::new()),
        ..valid_proposal()
    };
    let issues = validate_proposal_schema(&proposal);
    assert!(
        issues.iter().any(|i| i.message.contains("proposer")),
        "expected proposer issue, got: {issues:?}"
    );
}

// =========================================================================
// BEHAVIOR 21: validate_proposal_schema — Empty changes
// =========================================================================

#[test]
fn validate_proposal_schema_returns_issue_when_changes_array_is_empty() {
    let proposal = ApplyProposal {
        changes: vec![],
        ..valid_proposal()
    };
    let issues = validate_proposal_schema(&proposal);
    assert!(
        issues.iter().any(|i| i.message.contains("changes")),
        "expected changes issue, got: {issues:?}"
    );
}

// =========================================================================
// BEHAVIOR 22: validate_proposal_schema — Multiple violations
// =========================================================================

#[test]
fn validate_proposal_schema_returns_issues_for_multiple_simultaneous_schema_violations() {
    let proposal = ApplyProposal {
        proposer: AuthorId::new(String::new()),
        changes: vec![],
        ..valid_proposal()
    };
    let issues = validate_proposal_schema(&proposal);
    assert_eq!(issues.len(), 2);
    assert!(
        issues.iter().any(|i| i.message.contains("proposer")),
        "expected proposer issue, got: {issues:?}"
    );
    assert!(
        issues.iter().any(|i| i.message.contains("changes")),
        "expected changes issue, got: {issues:?}"
    );
}

// =========================================================================
// BEHAVIOR 22b: validate_proposal_schema — base_revision=0 boundary
// =========================================================================

#[test]
fn validate_proposal_schema_returns_empty_vec_when_base_revision_is_zero() {
    let proposal = ApplyProposal {
        base_revision: Revision::new(0),
        ..valid_proposal()
    };
    let issues = validate_proposal_schema(&proposal);
    assert_eq!(issues, vec![]);
}

// =========================================================================
// BEHAVIOR 23: check_revision_match — Match
// =========================================================================

#[test]
fn check_revision_match_returns_ok_when_revisions_match() {
    let doc = valid_document(42);
    let proposal = ApplyProposal {
        base_revision: Revision::new(42),
        ..valid_proposal()
    };
    let result = check_revision_match(&doc, &proposal);
    assert_eq!(result, Ok(()));
}

// =========================================================================
// BEHAVIOR 24: check_revision_match — Mismatch
// =========================================================================

#[test]
fn check_revision_match_returns_err_when_revisions_differ() {
    let doc = valid_document(44);
    let proposal = ApplyProposal {
        base_revision: Revision::new(42),
        ..valid_proposal()
    };
    let result = check_revision_match(&doc, &proposal);
    assert_eq!(
        result,
        Err(ConflictDetails {
            expected_revision: Revision::new(42),
            current_revision: Revision::new(44),
        })
    );
}

// =========================================================================
// BEHAVIOR 25: check_revision_match — Expected field fidelity
// =========================================================================

#[test]
fn check_revision_match_err_expected_equals_proposal_base_revision() {
    let doc = valid_document(1);
    let proposal = ApplyProposal {
        base_revision: Revision::new(99),
        ..valid_proposal()
    };
    let result = check_revision_match(&doc, &proposal);
    let err = result.expect_err("expected Err for mismatched revisions");
    assert_eq!(err.expected_revision, Revision::new(99));
}

// =========================================================================
// BEHAVIOR 26: check_revision_match — Current field fidelity
// =========================================================================

#[test]
fn check_revision_match_err_current_equals_document_revision() {
    let doc = valid_document(99);
    let proposal = ApplyProposal {
        base_revision: Revision::new(1),
        ..valid_proposal()
    };
    let result = check_revision_match(&doc, &proposal);
    let err = result.expect_err("expected Err for mismatched revisions");
    assert_eq!(err.current_revision, Revision::new(99));
}

// =========================================================================
// BEHAVIOR 27: check_revision_match — Purity
// =========================================================================

#[test]
fn check_revision_match_is_pure_no_side_effects_no_mutation() {
    let doc = valid_document(5);
    let proposal = ApplyProposal {
        base_revision: Revision::new(5),
        ..valid_proposal()
    };

    let result1 = check_revision_match(&doc, &proposal);
    let result2 = check_revision_match(&doc, &proposal);

    assert_eq!(result1, result2, "referential transparency violated");

    assert_eq!(doc.revision, Revision::new(5));
    assert_eq!(proposal.base_revision, Revision::new(5));
}
