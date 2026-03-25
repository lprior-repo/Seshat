//! Data types for the `seshat apply` subcommand.
//!
//! All types in this module are inert data — no I/O, no mutation.

use diagram_models::document::types::Revision;
use diagram_models::validation::ValidationIssue;

// ---------------------------------------------------------------------------
// Data types (Data Layer)
// ---------------------------------------------------------------------------

/// Source for the apply command input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplySource {
    /// Read from the filesystem at the given path.
    File(std::path::PathBuf),
    /// Read from stdin.
    Stdin,
}

/// Parsed arguments for the `seshat apply` subcommand. Data-only, no I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyCommand {
    /// Path to the proposal JSON file. `None` means stdin.
    pub input_source: ApplySource,
}

/// Structured status returned as JSON on stdout.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApplyStatus {
    Queued {
        proposal_id: String,
        change_count: usize,
        base_revision: u64,
    },
    Rejected {
        reason: RejectionReasonCode,
        #[serde(skip_serializing_if = "Option::is_none")]
        conflict_details: Option<ConflictDetails>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        validation_issues: Vec<ValidationIssue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
    },
}

/// Conflict details returned on revision mismatch (Human Priority Block).
/// Uses `Revision` newtype to prevent construction with raw non-revision values.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConflictDetails {
    pub expected_revision: Revision,
    pub current_revision: Revision,
}

/// Internal outcome of the apply pipeline before serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    Queued {
        proposal_id: String,
        change_count: usize,
        base_revision: u64,
    },
    Rejected {
        reason: RejectionReason,
    },
}

/// Reason for rejection in the apply pipeline (internal, with full context).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionReason {
    StaleRevision {
        expected: Revision,
        current: Revision,
    },
    SchemaInvalid {
        issues: Vec<ValidationIssue>,
    },
    EmptyChanges,
    InvalidProposer,
}

/// Typed rejection reason for JSON serialization.
/// Each variant maps to the exact contract-specified string in JSON output.
/// Replaces the previous `reason: String` to make illegal states unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum RejectionReasonCode {
    #[serde(rename = "Human Priority Block")]
    StaleRevision,
    #[serde(rename = "schema_validation_failed")]
    SchemaInvalid,
    #[serde(rename = "empty_changes")]
    EmptyChanges,
    #[serde(rename = "invalid_proposer")]
    InvalidProposer,
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors specific to the `apply` subcommand.
#[derive(Debug, PartialEq, Eq)]
pub enum ApplyCommandError {
    InputFileNotFound(std::path::PathBuf),
    InputIoError(String),
    InputInvalidUtf8,
    InputEmpty,
    ProposalJsonMalformed(String),
    ProposalSchemaInvalid {
        issues: Vec<ValidationIssue>,
    },
    ProposalEmptyChanges,
    ProposalInvalidProposer,
    DocumentNotFound(std::path::PathBuf),
    DocumentIoError(String),
    DocumentInvalidUtf8,
    DocumentEmpty,
    DocumentJsonMalformed(String),
    DocumentSchemaInvalid(String),
    StaleRevision {
        expected: Revision,
        current: Revision,
    },
    OutputWriteFailure(String),
    /// Proposal was validly rejected (status JSON written to stdout).
    /// Used as a sentinel to signal exit code 1 without stderr noise.
    ProposalRejected,
}

impl std::fmt::Display for ApplyCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputFileNotFound(path) => {
                write!(f, "error: apply: input file not found: {}", path.display())
            }
            Self::InputIoError(msg) => write!(f, "error: apply: I/O error: {msg}"),
            Self::InputInvalidUtf8 => write!(f, "error: apply: invalid UTF-8 in input"),
            Self::InputEmpty => write!(f, "error: apply: empty input"),
            Self::ProposalJsonMalformed(msg) => {
                write!(f, "error: apply: JSON parse error: {msg}")
            }
            Self::ProposalSchemaInvalid { issues } => {
                write!(
                    f,
                    "error: apply: schema validation failed: {} issues",
                    issues.len()
                )
            }
            Self::ProposalEmptyChanges => write!(f, "error: apply: empty changes"),
            Self::ProposalInvalidProposer => write!(f, "error: apply: invalid proposer"),
            Self::DocumentNotFound(path) => {
                write!(f, "error: apply: document not found: {}", path.display())
            }
            Self::DocumentIoError(msg) => write!(f, "error: apply: document I/O error: {msg}"),
            Self::DocumentInvalidUtf8 => write!(f, "error: apply: document invalid UTF-8"),
            Self::DocumentEmpty => write!(f, "error: apply: document empty"),
            Self::DocumentJsonMalformed(msg) => {
                write!(f, "error: apply: document JSON parse error: {msg}")
            }
            Self::DocumentSchemaInvalid(msg) => {
                write!(f, "error: apply: document schema invalid: {msg}")
            }
            Self::StaleRevision { expected, current } => {
                write!(
                    f,
                    "error: apply: stale revision: expected {}, current {}",
                    expected.value(),
                    current.value()
                )
            }
            Self::OutputWriteFailure(msg) => {
                write!(f, "error: apply: output write failure: {msg}")
            }
            Self::ProposalRejected => Ok(()),
        }
    }
}

impl std::error::Error for ApplyCommandError {}

// ---------------------------------------------------------------------------
// Proposal type for CLI parsing
// ---------------------------------------------------------------------------

/// A proposal as parsed by the CLI apply command.
///
/// Composes the domain `ProposedChanges` with the `changes` array that the CLI
/// requires. Use [`ApplyProposal::to_proposed_changes()`] to extract the base
/// domain type. The `From<&ApplyProposal>` impl for `ProposedChanges` provides
/// ergonomic conversion.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyProposal {
    pub base_revision: Revision,
    pub proposer: diagram_models::document::types::AuthorId,
    pub proposed_at: diagram_models::document::types::Timestamp,
    pub summary: String,
    pub changes: Vec<diagram_models::proposed_changes::ProposedChange>,
}

impl ApplyProposal {
    /// Extracts the base [`ProposedChanges`] domain type from this CLI proposal.
    #[must_use]
    pub fn to_proposed_changes(&self) -> diagram_models::proposed_changes::ProposedChanges {
        diagram_models::proposed_changes::ProposedChanges {
            base_revision: self.base_revision,
            proposer: self.proposer.clone(),
            proposed_at: self.proposed_at,
            summary: self.summary.clone(),
        }
    }
}

impl From<&ApplyProposal> for diagram_models::proposed_changes::ProposedChanges {
    fn from(proposal: &ApplyProposal) -> Self {
        proposal.to_proposed_changes()
    }
}
