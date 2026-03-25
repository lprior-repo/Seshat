//! Implementation of the `seshat show` subcommand.
//!
//! # Architecture: Data → Calculations → Actions
//!
//! - **Data**: `ShowCommand`, `ShowSource`, `DiagramDocument` — inert, no I/O.
//! - **Calculations**: `map_show_subcommand`, `serialize_document` — pure functions.
//! - **Actions**: `load_document_from_path`, `load_document_from_reader`, `execute_show` — I/O boundary.

use std::io::Read as _;

#[cfg(test)]
mod proptests;
#[cfg(test)]
mod serialize_tests;
#[cfg(test)]
mod unit_tests;
#[cfg(kani)]
mod verification;

use diagram_models::document::DiagramDocument;

use crate::domain::{ShowCommand, ShowSource};
use crate::error::ShowError;

/// Maps the clap-parsed Show args into the domain `ShowCommand` type.
/// Pure mapping function — no I/O.
///
/// # Postconditions
/// - Returns `ShowCommand { source: ShowSource::File(path) }` when `file` is `Some`.
/// - Returns `ShowCommand { source: ShowSource::Stdin }` when `file` is `None`.
pub(crate) fn map_show_subcommand(file: Option<std::path::PathBuf>) -> ShowCommand {
    ShowCommand {
        source: file.map_or(ShowSource::Stdin, ShowSource::File),
    }
}

/// Serializes a `DiagramDocument` to a compact JSON string.
///
/// # Errors
/// - `ShowError::SerializationFailure` — `serde_json` internal error (unreachable in practice).
pub fn serialize_document(doc: &DiagramDocument) -> Result<String, ShowError> {
    serde_json::to_string(doc).map_err(|e| ShowError::SerializationFailure(e.to_string()))
}

/// Reads bytes from the filesystem and deserializes a `DiagramDocument`.
///
/// # Errors
/// - `ShowError::FileNotFound`    — path does not exist or permissions denied.
/// - `ShowError::IoError`         — other I/O failure during read.
/// - `ShowError::InvalidUtf8`     — bytes are not valid UTF-8.
/// - `ShowError::EmptyInput`      — file is zero bytes (or whitespace only).
/// - `ShowError::JsonDeserialize` — content is not valid JSON.
/// - `ShowError::InvalidDocument` — JSON is valid but does not match `DiagramDocument` schema.
pub fn load_document_from_path(path: &std::path::Path) -> Result<DiagramDocument, ShowError> {
    std::fs::File::open(path)
        .map_err(|e| map_open_error(&e, path))
        .and_then(load_document_from_reader)
}

/// Maps a `std::io::Error` from `File::open` into the correct `ShowError` variant.
///
/// `NotFound` and `PermissionDenied` map to `FileNotFound`.
/// All other I/O errors map to `IoError`.
fn map_open_error(e: &std::io::Error, path: &std::path::Path) -> ShowError {
    match e.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
            ShowError::FileNotFound(path.to_path_buf())
        }
        _ => ShowError::IoError(e.to_string()),
    }
}

/// Reads bytes from any `Read` implementor and deserializes a `DiagramDocument`.
///
/// # Errors
/// - `ShowError::IoError`         — I/O failure while reading.
/// - `ShowError::InvalidUtf8`     — bytes are not valid UTF-8.
/// - `ShowError::EmptyInput`      — reader yields zero bytes (or whitespace only).
/// - `ShowError::JsonDeserialize` — content is not valid JSON.
/// - `ShowError::InvalidDocument` — JSON valid but document schema mismatch.
pub fn load_document_from_reader<R: std::io::Read>(
    mut reader: R,
) -> Result<DiagramDocument, ShowError> {
    let bytes = read_bytes_from_reader(&mut reader)?;
    let text = decode_utf8(bytes)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ShowError::EmptyInput);
    }
    parse_document(trimmed)
}

/// Maximum bytes read from any reader to prevent infinite-stream hangs.
/// 64 MiB is sufficient for any realistic diagram document.
const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;

/// Reads all bytes from a reader into a `Vec<u8>`.
///
/// Reads at most `MAX_INPUT_BYTES` bytes. If the stream exceeds this limit
/// the read is truncated and subsequent parsing will fail gracefully.
///
/// # Errors
/// - `ShowError::IoError` if reading fails.
fn read_bytes_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Vec<u8>, ShowError> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(MAX_INPUT_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|e| ShowError::IoError(e.to_string()))?;
    Ok(bytes)
}

/// Converts raw bytes to a UTF-8 `String`.
///
/// # Errors
/// - `ShowError::InvalidUtf8` if the bytes are not valid UTF-8.
fn decode_utf8(bytes: Vec<u8>) -> Result<String, ShowError> {
    String::from_utf8(bytes).map_err(|_| ShowError::InvalidUtf8)
}

/// Parses a trimmed JSON string into a `DiagramDocument`.
///
/// Uses a two-pass strategy:
/// 1. Parse as raw `serde_json::Value` to detect syntax/EOF errors → `JsonDeserialize`.
/// 2. Parse as `DiagramDocument`; data errors (unknown fields, schema mismatches)
///    containing "unknown field" → `InvalidDocument`; all others → `JsonDeserialize`.
///
/// # Errors
/// - `ShowError::InvalidDocument` — JSON valid but unknown fields or structural mismatch.
/// - `ShowError::JsonDeserialize` — malformed JSON syntax or type mismatch.
fn parse_document(json: &str) -> Result<DiagramDocument, ShowError> {
    // Pass 1: validate raw JSON syntax first.
    serde_json::from_str::<serde_json::Value>(json)
        .map_err(|e| ShowError::JsonDeserialize(e.to_string()))?;

    // Pass 2: deserialize into the typed struct.
    serde_json::from_str::<DiagramDocument>(json).map_err(|e| {
        if e.is_data() && e.to_string().contains("unknown field") {
            ShowError::InvalidDocument(e.to_string())
        } else {
            ShowError::JsonDeserialize(e.to_string())
        }
    })
}

/// Top-level executor for the `show` subcommand.
/// Orchestrates: load → serialize → write.
///
/// Accepts an injectable writer and an injectable `serialize_fn` for deterministic testing.
///
/// # Postconditions
/// - On `Ok(())`: the JSON representation of the `DiagramDocument` followed by `\n` has been
///   written to `writer`.
///
/// # Errors
/// - Any `ShowError` variant may be returned.
pub fn execute_show<W: std::io::Write>(
    cmd: &ShowCommand,
    stdin_reader: impl std::io::Read,
    mut writer: W,
    serialize_fn: impl Fn(&DiagramDocument) -> Result<String, ShowError>,
) -> Result<(), ShowError> {
    let doc = load_document(cmd, stdin_reader)?;
    let json = serialize_fn(&doc)?;
    write_output(&mut writer, &json)
}

/// Dispatches loading to the correct source (file vs. stdin).
fn load_document(
    cmd: &ShowCommand,
    stdin_reader: impl std::io::Read,
) -> Result<DiagramDocument, ShowError> {
    match &cmd.source {
        ShowSource::File(path) => load_document_from_path(path),
        ShowSource::Stdin => load_document_from_reader(stdin_reader),
    }
}

/// Writes the JSON string followed by a single newline to `writer`, then flushes.
///
/// # Errors
/// - `ShowError::StdoutWriteFailure` if `write_all` or `flush` fails.
fn write_output<W: std::io::Write>(writer: &mut W, json: &str) -> Result<(), ShowError> {
    let mut output = Vec::with_capacity(json.len() + 1);
    output.extend_from_slice(json.as_bytes());
    output.push(b'\n');
    writer
        .write_all(&output)
        .map_err(|e| ShowError::StdoutWriteFailure(e.to_string()))?;
    writer
        .flush()
        .map_err(|e| ShowError::StdoutWriteFailure(e.to_string()))
}
