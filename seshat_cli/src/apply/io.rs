//! I/O functions (Action Layer) for the `seshat apply` subcommand.

use std::io::Read;
use std::path::Path;

use diagram_models::document::DiagramDocument;
use diagram_models::validation::{ValidationCode, ValidationIssue};

use crate::apply::calc::{build_apply_status, serialize_apply_status};
use crate::apply::types::*;

/// Maximum bytes read from any reader to prevent infinite-stream hangs.
pub(crate) const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;

/// Reads and parses a proposal from the given source.
/// Internal variant with configurable max_bytes for testing.
///
/// # Errors
/// Returns `ApplyCommandError` for any I/O, UTF-8, JSON, or schema failure.
pub(crate) fn load_proposal_with_limit(
    source: &ApplySource,
    stdin: impl std::io::Read,
    max_bytes: u64,
) -> Result<ApplyProposal, ApplyCommandError> {
    let bytes = match source {
        ApplySource::File(path) => read_file_bytes(path, max_bytes)?,
        ApplySource::Stdin => read_reader_bytes(stdin, max_bytes)?,
    };
    let text = String::from_utf8(bytes).map_err(|_| ApplyCommandError::InputInvalidUtf8)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ApplyCommandError::InputEmpty);
    }
    let proposal = parse_proposal_json(trimmed)?;
    if proposal.changes.is_empty() {
        return Err(ApplyCommandError::ProposalEmptyChanges);
    }
    if proposal.proposer.as_str().is_empty() {
        return Err(ApplyCommandError::ProposalInvalidProposer);
    }
    Ok(proposal)
}

/// Reads and parses a proposal from the given source.
///
/// # Errors
/// Returns `ApplyCommandError` for any I/O, UTF-8, JSON, or schema failure.
pub fn load_proposal(
    source: &ApplySource,
    stdin: impl std::io::Read,
) -> Result<ApplyProposal, ApplyCommandError> {
    load_proposal_with_limit(source, stdin, MAX_INPUT_BYTES)
}

fn read_file_bytes(path: &Path, max_bytes: u64) -> Result<Vec<u8>, ApplyCommandError> {
    let mut file = std::fs::File::open(path).map_err(|e| map_proposal_open_error(e, path))?;
    let metadata = file
        .metadata()
        .map_err(|e| map_proposal_open_error(e, path))?;
    if metadata.len() > max_bytes {
        return Err(ApplyCommandError::ProposalJsonMalformed(
            "input exceeds maximum size".to_string(),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| map_proposal_open_error(e, path))?;
    Ok(bytes)
}

fn map_proposal_open_error(e: std::io::Error, path: &Path) -> ApplyCommandError {
    match e.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
            ApplyCommandError::InputFileNotFound(path.to_path_buf())
        }
        _ => ApplyCommandError::InputIoError(e.to_string()),
    }
}

fn read_reader_bytes<R: std::io::Read>(
    mut reader: R,
    max_bytes: u64,
) -> Result<Vec<u8>, ApplyCommandError> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(max_bytes)
        .read_to_end(&mut bytes)
        .map_err(|e: std::io::Error| ApplyCommandError::InputIoError(e.to_string()))?;
    Ok(bytes)
}

fn parse_proposal_json(json: &str) -> Result<ApplyProposal, ApplyCommandError> {
    // Pass 1: validate raw JSON syntax
    serde_json::from_str::<serde_json::Value>(json)
        .map_err(|e| ApplyCommandError::ProposalJsonMalformed(e.to_string()))?;

    // Pass 2: deserialize into the typed struct.
    // Uses serde_json's typed `is_data()` classifier instead of fragile string matching
    // to distinguish schema errors (valid JSON, wrong shape) from malformed errors.
    serde_json::from_str::<ApplyProposal>(json).map_err(|e| {
        if e.is_data() {
            ApplyCommandError::ProposalSchemaInvalid {
                issues: vec![ValidationIssue::error(
                    ValidationCode::SCHEMA,
                    e.to_string(),
                    None,
                )],
            }
        } else {
            ApplyCommandError::ProposalJsonMalformed(e.to_string())
        }
    })
}

/// Internal load_current_document with configurable max_bytes for testing.
///
/// # Errors
/// Returns `ApplyCommandError` for any I/O, UTF-8, JSON, or schema failure.
pub(crate) fn load_current_document_with_limit(
    path: &Path,
    max_bytes: u64,
) -> Result<DiagramDocument, ApplyCommandError> {
    let mut file = std::fs::File::open(path).map_err(|e| map_document_open_error(e, path))?;
    let metadata = file
        .metadata()
        .map_err(|e| map_document_open_error(e, path))?;
    if metadata.len() > max_bytes {
        return Err(ApplyCommandError::DocumentJsonMalformed(
            "input exceeds maximum size".to_string(),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| map_document_open_error(e, path))?;
    let text = String::from_utf8(bytes).map_err(|_| ApplyCommandError::DocumentInvalidUtf8)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ApplyCommandError::DocumentEmpty);
    }
    parse_document_json(trimmed)
}

/// Reads and parses the current document from the given path.
///
/// # Errors
/// Returns `ApplyCommandError` for any I/O, UTF-8, JSON, or schema failure.
pub fn load_current_document(path: &Path) -> Result<DiagramDocument, ApplyCommandError> {
    load_current_document_with_limit(path, MAX_INPUT_BYTES)
}

fn map_document_open_error(e: std::io::Error, path: &Path) -> ApplyCommandError {
    match e.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
            ApplyCommandError::DocumentNotFound(path.to_path_buf())
        }
        _ => ApplyCommandError::DocumentIoError(e.to_string()),
    }
}

fn parse_document_json(json: &str) -> Result<DiagramDocument, ApplyCommandError> {
    // Pass 1: validate raw JSON syntax
    serde_json::from_str::<serde_json::Value>(json)
        .map_err(|e| ApplyCommandError::DocumentJsonMalformed(e.to_string()))?;

    // Pass 2: deserialize into the typed struct.
    // Uses serde_json's typed `is_data()` classifier instead of fragile string matching.
    serde_json::from_str::<DiagramDocument>(json).map_err(|e| {
        if e.is_data() {
            ApplyCommandError::DocumentSchemaInvalid(e.to_string())
        } else {
            ApplyCommandError::DocumentJsonMalformed(e.to_string())
        }
    })
}

// Top-level executor

/// Routes `load_proposal` errors: I/O failures → `Err`, schema rejections → emit + `Ok(None)`.
///
/// This is extracted from `execute_apply` to keep the orchestrator under 25 lines (Farley rule).
fn route_load_proposal_error<W: std::io::Write>(
    result: Result<ApplyProposal, ApplyCommandError>,
    writer: &mut W,
) -> Result<Option<ApplyProposal>, ApplyCommandError> {
    match result {
        Ok(proposal) => Ok(Some(proposal)),
        Err(e @ ApplyCommandError::InputFileNotFound(_)) => Err(e),
        Err(e @ ApplyCommandError::InputIoError(_)) => Err(e),
        Err(e @ ApplyCommandError::InputInvalidUtf8) => Err(e),
        Err(e @ ApplyCommandError::InputEmpty) => Err(e),
        Err(e @ ApplyCommandError::ProposalJsonMalformed(_)) => Err(e),
        Err(ApplyCommandError::ProposalSchemaInvalid { issues }) => {
            emit_status(
                ApplyOutcome::Rejected {
                    reason: RejectionReason::SchemaInvalid { issues },
                },
                writer,
            )?;
            Ok(None)
        }
        Err(ApplyCommandError::ProposalEmptyChanges) => {
            emit_status(
                ApplyOutcome::Rejected {
                    reason: RejectionReason::EmptyChanges,
                },
                writer,
            )?;
            Ok(None)
        }
        Err(ApplyCommandError::ProposalInvalidProposer) => {
            emit_status(
                ApplyOutcome::Rejected {
                    reason: RejectionReason::InvalidProposer,
                },
                writer,
            )?;
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

/// Top-level executor for the `seshat apply` subcommand.
/// Orchestrates: load proposal → load document → check revision → emit status.
///
/// I/O errors from `load_proposal` are returned as `Err`.
/// Schema-level issues (missing fields, empty changes, empty proposer) are converted
/// to rejections written to stdout and return `Ok(false)`.
///
/// # Returns
/// - `Ok(true)` — proposal was queued (status JSON on stdout)
/// - `Ok(false)` — proposal was rejected (status JSON on stdout)
/// - `Err(ApplyCommandError)` — I/O or output failure
///
/// # Errors
/// Returns `ApplyCommandError` for I/O failures or output write failures.
pub fn execute_apply<R: std::io::Read, W: std::io::Write>(
    cmd: &ApplyCommand,
    stdin: R,
    mut writer: W,
    doc_path: &Path,
) -> Result<bool, ApplyCommandError> {
    let proposal =
        match route_load_proposal_error(load_proposal(&cmd.input_source, stdin), &mut writer)? {
            Some(p) => p,
            None => return Ok(false),
        };

    let doc = load_current_document(doc_path)?;

    match crate::apply::calc::check_revision_match(&doc, &proposal) {
        Ok(()) => {
            emit_status(
                ApplyOutcome::Queued {
                    proposal_id: format!("prop-{}", crate::apply::calc::nanoid_like_id()),
                    change_count: proposal.changes.len(),
                    base_revision: proposal.base_revision.value(),
                },
                &mut writer,
            )?;
            Ok(true)
        }
        Err(details) => {
            emit_status(
                ApplyOutcome::Rejected {
                    reason: RejectionReason::StaleRevision {
                        expected: details.expected_revision,
                        current: details.current_revision,
                    },
                },
                &mut writer,
            )?;
            Ok(false)
        }
    }
}

/// Serializes an `ApplyOutcome` and writes it to the given writer.
/// Consumes the outcome, moving ownership of any contained data (e.g., validation issues).
///
/// # Errors
/// Returns `ApplyCommandError::OutputWriteFailure` if serialization or I/O fails.
fn emit_status<W: std::io::Write>(
    outcome: ApplyOutcome,
    writer: &mut W,
) -> Result<(), ApplyCommandError> {
    let status = build_apply_status(outcome);
    let json = serialize_apply_status(&status)?;
    writer
        .write_all(json.as_bytes())
        .map_err(|e| ApplyCommandError::OutputWriteFailure(e.to_string()))?;
    writer
        .flush()
        .map_err(|e| ApplyCommandError::OutputWriteFailure(e.to_string()))
}
