//! Pure calculation functions for the `seshat apply` subcommand.
//!
//! All functions in this module are pure — no I/O, no mutation.

use diagram_models::document::DiagramDocument;
use diagram_models::validation::{ValidationCode, ValidationIssue};

use crate::apply::types::*;

// ---------------------------------------------------------------------------
// Pure functions (Calc Layer)
// ---------------------------------------------------------------------------

/// Maps clap-parsed apply args into the domain `ApplyCommand` type.
/// Pure mapping function — no I/O.
pub(crate) fn map_apply_subcommand(file: Option<std::path::PathBuf>) -> ApplyCommand {
    ApplyCommand {
        input_source: file.map_or(ApplySource::Stdin, ApplySource::File),
    }
}

/// Validates a proposal's structural schema (fields, types, constraints).
/// Returns all validation issues found. Does NOT check revision.
///
/// Pure function — no I/O.
pub fn validate_proposal_schema(proposal: &ApplyProposal) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if proposal.proposer.as_str().is_empty() {
        issues.push(ValidationIssue::error(
            ValidationCode::SCHEMA,
            "proposer must be non-empty",
            Some("proposer".to_string()),
        ));
    }

    if proposal.changes.is_empty() {
        issues.push(ValidationIssue::error(
            ValidationCode::SCHEMA,
            "changes array must not be empty",
            Some("changes".to_string()),
        ));
    }

    issues
}

/// Checks if the proposal's base_revision matches the document's revision.
///
/// Pure function — no I/O, no mutation.
pub fn check_revision_match(
    doc: &DiagramDocument,
    proposal: &ApplyProposal,
) -> Result<(), ConflictDetails> {
    if proposal.base_revision == doc.revision {
        Ok(())
    } else {
        Err(ConflictDetails {
            expected_revision: proposal.base_revision,
            current_revision: doc.revision,
        })
    }
}

/// Builds the structured [`ApplyStatus`] JSON for stdout output.
///
/// Pure function — no I/O. Takes ownership of `outcome` to avoid cloning
/// inner data (e.g., validation issues). The `proposal_id` is injected
/// by the Action layer (`execute_apply`) so this function stays referentially
/// transparent.
pub fn build_apply_status(outcome: ApplyOutcome) -> ApplyStatus {
    match outcome {
        ApplyOutcome::Queued {
            proposal_id,
            change_count,
            base_revision,
        } => ApplyStatus::Queued {
            proposal_id,
            change_count,
            base_revision,
        },
        ApplyOutcome::Rejected { reason } => match reason {
            RejectionReason::StaleRevision { expected, current } => ApplyStatus::Rejected {
                reason: RejectionReasonCode::StaleRevision,
                conflict_details: Some(ConflictDetails {
                    expected_revision: expected,
                    current_revision: current,
                }),
                validation_issues: vec![],
                hint: Some(
                    "Run 'seshat show --json' to get the latest document state, then regenerate your proposal against the current revision.".to_string(),
                ),
            },
            RejectionReason::SchemaInvalid { issues } => ApplyStatus::Rejected {
                reason: RejectionReasonCode::SchemaInvalid,
                conflict_details: None,
                validation_issues: issues,
                hint: None,
            },
            RejectionReason::EmptyChanges => ApplyStatus::Rejected {
                reason: RejectionReasonCode::EmptyChanges,
                conflict_details: None,
                validation_issues: vec![],
                hint: None,
            },
            RejectionReason::InvalidProposer => ApplyStatus::Rejected {
                reason: RejectionReasonCode::InvalidProposer,
                conflict_details: None,
                validation_issues: vec![],
                hint: None,
            },
        },
    }
}

/// Simple nanoid-like ID generator for proposal IDs.
pub(crate) fn nanoid_like_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{seed:016x}")
}

/// Serializes [`ApplyStatus`] to compact JSON.
///
/// # Errors
/// Returns `ApplyCommandError::OutputWriteFailure` if serialization fails.
pub fn serialize_apply_status(status: &ApplyStatus) -> Result<String, ApplyCommandError> {
    serde_json::to_string(status).map_err(|e| ApplyCommandError::OutputWriteFailure(e.to_string()))
}
