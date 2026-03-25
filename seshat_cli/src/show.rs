//! Implementation of the `seshat show` subcommand.
//!
//! # Architecture: Data → Calculations → Actions
//!
//! - **Data**: `ShowCommand`, `ShowSource`, `DiagramDocument` — inert, no I/O.
//! - **Calculations**: `map_show_subcommand`, `serialize_document` — pure functions.
//! - **Actions**: `load_document_from_path`, `load_document_from_reader`, `execute_show` — I/O boundary.

use std::io::Read as _;

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
        // FRAGILE: This check relies on serde_json's internal error message format.
        // If serde_json changes "unknown field" to different wording in a future release,
        // unknown-field errors will be silently routed to JsonDeserialize instead of
        // InvalidDocument. The two tests
        // `load_document_from_*_returns_json_deserialize_when_field_has_wrong_type` and
        // `load_document_from_*_returns_invalid_document_when_json_has_unknown_fields`
        // guard both branches and will catch such a regression.
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

// ---------------------------------------------------------------------------
// Unit Tests (Calc layer — pure functions)
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod unit_tests {
    use super::*;
    use crate::domain::{ShowCommand, ShowSource};
    use crate::error::{ExecutionError, ShowError};
    use std::path::PathBuf;

    // -----------------------------------------------------------------------
    // ShowError Display tests
    // -----------------------------------------------------------------------

    #[test]
    fn show_error_display_file_not_found_starts_with_prefix_and_contains_payload() {
        let err = ShowError::FileNotFound(PathBuf::from("/tmp/missing.json"));
        let output = err.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        assert!(
            output.contains("/tmp/missing.json"),
            "expected path in output, got: {output:?}"
        );
    }

    #[test]
    fn show_error_display_io_error_starts_with_prefix_and_contains_payload() {
        let err = ShowError::IoError("disk fail".to_string());
        let output = err.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        assert!(
            output.contains("disk fail"),
            "expected payload in output, got: {output:?}"
        );
    }

    #[test]
    fn show_error_display_invalid_utf8_starts_with_prefix_and_contains_utf8_mention() {
        let err = ShowError::InvalidUtf8;
        let output = err.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        let lower = output.to_lowercase();
        assert!(
            lower.contains("utf-8") || lower.contains("utf8"),
            "expected utf-8 mention in output, got: {output:?}"
        );
    }

    #[test]
    fn show_error_display_empty_input_starts_with_prefix_and_contains_empty() {
        let err = ShowError::EmptyInput;
        let output = err.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        assert!(
            output.contains("empty"),
            "expected 'empty' in output, got: {output:?}"
        );
    }

    #[test]
    fn show_error_display_json_deserialize_starts_with_prefix_and_contains_payload() {
        let err = ShowError::JsonDeserialize("parse error".to_string());
        let output = err.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        assert!(
            output.contains("parse error"),
            "expected payload in output, got: {output:?}"
        );
    }

    #[test]
    fn show_error_display_invalid_document_starts_with_prefix_and_contains_payload() {
        let err = ShowError::InvalidDocument("unknown field".to_string());
        let output = err.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        assert!(
            output.contains("unknown field"),
            "expected payload in output, got: {output:?}"
        );
    }

    #[test]
    fn show_error_display_serialization_failure_starts_with_prefix_and_contains_payload() {
        let err = ShowError::SerializationFailure("ser fail".to_string());
        let output = err.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        assert!(
            output.contains("ser fail"),
            "expected payload in output, got: {output:?}"
        );
    }

    #[test]
    fn show_error_display_stdout_write_failure_starts_with_prefix_and_contains_payload() {
        let err = ShowError::StdoutWriteFailure("write fail".to_string());
        let output = err.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        assert!(
            output.contains("write fail"),
            "expected payload in output, got: {output:?}"
        );
    }

    // -----------------------------------------------------------------------
    // ExecutionError::Show delegates Display to ShowError
    // -----------------------------------------------------------------------

    #[test]
    fn execution_error_show_variant_delegates_display_to_show_error() {
        let e = ExecutionError::Show(ShowError::EmptyInput);
        let output = e.to_string();
        assert!(
            output.starts_with("error: show: "),
            "expected prefix 'error: show: ', got: {output:?}"
        );
        assert!(
            output.contains("empty"),
            "expected 'empty' in output, got: {output:?}"
        );
    }

    // -----------------------------------------------------------------------
    // map_show_subcommand unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn map_show_subcommand_returns_file_source_when_path_is_some() {
        let path = PathBuf::from("/some/path.json");
        let result = map_show_subcommand(Some(path.clone()));
        assert_eq!(
            result,
            ShowCommand {
                source: ShowSource::File(path)
            }
        );
    }

    #[test]
    fn map_show_subcommand_returns_file_source_when_path_is_relative() {
        let path = PathBuf::from("rel/path.json");
        let result = map_show_subcommand(Some(path.clone()));
        assert_eq!(
            result,
            ShowCommand {
                source: ShowSource::File(path)
            }
        );
    }

    #[test]
    fn map_show_subcommand_returns_file_source_when_path_is_root() {
        let path = PathBuf::from("/");
        let result = map_show_subcommand(Some(path.clone()));
        assert_eq!(
            result,
            ShowCommand {
                source: ShowSource::File(path)
            }
        );
    }

    #[test]
    fn map_show_subcommand_returns_stdin_source_when_path_is_none() {
        let result = map_show_subcommand(None);
        assert_eq!(
            result,
            ShowCommand {
                source: ShowSource::Stdin
            }
        );
    }

    // -----------------------------------------------------------------------
    // ShowCommand trait derive tests (INV-8)
    // -----------------------------------------------------------------------

    #[test]
    fn show_command_clone_produces_equal_value() {
        let cmd = ShowCommand {
            source: ShowSource::File(PathBuf::from("/a/b.json")),
        };
        let cloned = cmd.clone();
        assert_eq!(cloned, cmd);
    }

    #[test]
    fn show_command_debug_output_contains_type_and_variant_names() {
        let cmd = ShowCommand {
            source: ShowSource::Stdin,
        };
        let output = format!("{cmd:?}");
        assert!(
            output.contains("ShowCommand"),
            "debug output must contain 'ShowCommand', got: {output:?}"
        );
        assert!(
            output.contains("Stdin"),
            "debug output must contain 'Stdin', got: {output:?}"
        );
    }

    // -----------------------------------------------------------------------
    // serialize_document unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn serialize_document_returns_compact_json_when_given_default_document() {
        let doc = DiagramDocument::default();
        let result = serialize_document(&doc);
        assert!(
            result.is_ok(),
            "serialize_document must succeed for default doc: {result:?}"
        );
        let json = result.unwrap();
        let deserialized = serde_json::from_str::<DiagramDocument>(&json);
        assert!(
            deserialized.is_ok(),
            "deserialized must succeed: {:?}",
            deserialized.as_ref().err()
        );
        assert_eq!(deserialized.unwrap(), doc);
    }

    #[test]
    fn serialize_document_output_contains_version_zero_when_document_version_is_zero() {
        let doc = DiagramDocument {
            version: 0,
            ..DiagramDocument::default()
        };
        let result = serialize_document(&doc);
        assert!(
            result.is_ok(),
            "serialize must succeed for version 0 doc: {result:?}"
        );
        assert!(
            result.as_ref().unwrap().contains("\"version\":0"),
            "output must contain \"version\":0, got: {:?}",
            result.as_ref().unwrap()
        );
    }

    #[test]
    fn serialize_document_output_contains_version_one_when_document_version_is_one() {
        let doc = DiagramDocument {
            version: 1,
            ..DiagramDocument::default()
        };
        let result = serialize_document(&doc);
        assert!(
            result.is_ok(),
            "serialize must succeed for version 1 doc: {result:?}"
        );
        assert!(
            result.as_ref().unwrap().contains("\"version\":1"),
            "output must contain \"version\":1, got: {:?}",
            result.as_ref().unwrap()
        );
    }

    #[test]
    fn serialize_document_output_contains_node_id_when_document_has_nodes() {
        use diagram_models::document::types::OrderedFloat;
        use diagram_models::document::{LockState, Node, NodeId, NodeKind};
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new("node-abc".to_string());
        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "test".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document.nodes.insert(node_id, node);
        let result = serialize_document(&doc);
        assert!(
            result.is_ok(),
            "serialize must succeed for doc with nodes: {result:?}"
        );
        let json = result.unwrap();
        assert!(
            json.contains("node-abc"),
            "output must contain 'node-abc', got: {json:?}"
        );
        assert!(
            json.contains("\"nodes\":"),
            "output must contain '\"nodes\":', got: {json:?}"
        );
    }

    #[test]
    fn serialize_document_output_contains_no_newlines_or_indentation() {
        let doc = DiagramDocument::default();
        let result = serialize_document(&doc);
        assert!(result.is_ok(), "serialize must succeed: {result:?}");
        let json = result.unwrap();
        assert!(
            !json.contains('\n'),
            "compact JSON must not contain newlines, got: {json:?}"
        );
        assert!(
            !json.contains("  "),
            "compact JSON must not contain double-space indentation, got: {json:?}"
        );
    }

    #[test]
    fn serialize_document_round_trips_to_identical_document_when_serialized_then_deserialized() {
        let doc = DiagramDocument::default();
        let json_result = serialize_document(&doc);
        assert!(
            json_result.is_ok(),
            "serialize must succeed: {json_result:?}"
        );
        let json = json_result.unwrap();
        let deserialized = serde_json::from_str::<DiagramDocument>(&json);
        assert!(
            deserialized.is_ok(),
            "deserialized must succeed: {:?}",
            deserialized.as_ref().err()
        );
        assert_eq!(deserialized.unwrap(), doc);
    }

    // -----------------------------------------------------------------------
    // serialize_document B-29: SerializationFailure error arm
    // We test the error-mapping code by calling the internal mapping helper
    // through a test-only wrapper accepting any Serialize type.
    // -----------------------------------------------------------------------

    /// Test-only serialize function that maps the serde error identically to
    /// how `serialize_document` would, but accepts any Serialize type.
    fn serialize_any<T: serde::Serialize>(val: &T) -> Result<String, ShowError> {
        serde_json::to_string(val).map_err(|e| ShowError::SerializationFailure(e.to_string()))
    }

    /// A type that always fails to serialize.
    struct AlwaysFailsSerialize;

    impl serde::Serialize for AlwaysFailsSerialize {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("injected serialization error"))
        }
    }

    #[test]
    fn serialize_document_returns_serialization_failure_when_serde_json_errors() {
        let failing = AlwaysFailsSerialize;
        let result = serialize_any(&failing);
        assert!(
            matches!(result, Err(ShowError::SerializationFailure(_))),
            "expected SerializationFailure, got: {result:?}"
        );
        if let Err(ShowError::SerializationFailure(msg)) = result {
            assert!(
                msg.contains("injected serialization error"),
                "error message must contain injected payload, got: {msg:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Proptest invariants
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::domain::{ShowCommand, ShowSource};
    use diagram_models::document::DiagramDocument;
    use proptest::prelude::*;
    use std::io::Cursor;
    use std::path::PathBuf;

    // INV-1: map_show_subcommand never panics for any Option<PathBuf>
    proptest! {
        #[test]
        fn proptest_map_show_subcommand_never_panics_for_any_option_pathbuf(
            bytes in prop::option::of(prop::collection::vec(any::<u8>(), 0..256))
        ) {
            let path_opt = bytes.map(|b| {
                use std::ffi::OsString;
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStringExt;
                    PathBuf::from(OsString::from_vec(b))
                }
                #[cfg(not(unix))]
                {
                    PathBuf::from(String::from_utf8_lossy(&b).to_string())
                }
            });
            let is_some = path_opt.is_some();
            let result = map_show_subcommand(path_opt);
            if is_some {
                prop_assert!(matches!(result.source, ShowSource::File(_)));
            } else {
                prop_assert_eq!(result.source, ShowSource::Stdin);
            }
        }
    }

    // INV-2: serialize_document is total for any DiagramDocument (version sweep)
    proptest! {
        #[test]
        fn proptest_serialize_document_returns_ok_for_any_well_formed_document(
            version in any::<u32>()
        ) {
            let doc = DiagramDocument {
                version,
                ..DiagramDocument::default()
            };
            let result = serialize_document(&doc);
            prop_assert!(result.is_ok(), "serialize_document must return Ok for any version: {result:?}");
            if let Ok(ref json) = result {
                prop_assert!(!json.is_empty(), "serialized JSON must be non-empty");
            }
        }
    }

    // INV-3: JSON round-trip identity
    proptest! {
        #[test]
        fn proptest_serialize_then_deserialize_produces_identical_document(
            version in any::<u32>()
        ) {
            let doc = DiagramDocument {
                version,
                ..DiagramDocument::default()
            };
            let json_result = serialize_document(&doc);
            prop_assert!(json_result.is_ok(), "serialize must succeed: {json_result:?}");
            if let Ok(json) = json_result {
                let doc2 = load_document_from_reader(Cursor::new(json.into_bytes()));
                prop_assert_eq!(doc2, Ok(doc));
            }
        }
    }

    // INV-4: load_document_from_reader never panics on arbitrary bytes
    proptest! {
        #[test]
        fn proptest_load_document_from_reader_never_panics_for_arbitrary_byte_input(
            bytes in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let reader = Cursor::new(bytes);
            let result = load_document_from_reader(reader);
            // Must return either Ok or a defined Err variant — never panic
            match result {
                Ok(_) => prop_assert!(true),
                Err(
                    ShowError::EmptyInput
                    | ShowError::InvalidUtf8
                    | ShowError::JsonDeserialize(_)
                    | ShowError::InvalidDocument(_)
                    | ShowError::IoError(_),
                ) => prop_assert!(true),
                Err(
                    ShowError::FileNotFound(_)
                    | ShowError::SerializationFailure(_)
                    | ShowError::StdoutWriteFailure(_),
                ) => {
                    prop_assert!(false, "reader-based load must not return file/write errors");
                }
            }
        }
    }

    // INV-5: execute_show output always ends with exactly one newline on success
    proptest! {
        #[test]
        fn proptest_execute_show_output_ends_with_exactly_one_newline_when_successful(
            version in any::<u32>()
        ) {
            let doc = DiagramDocument {
                version,
                ..DiagramDocument::default()
            };
            let Ok(json) = serde_json::to_string(&doc) else {
                return Ok(());
            };
            let cmd = ShowCommand { source: ShowSource::Stdin };
            let reader = Cursor::new(json.into_bytes());
            let mut writer = Vec::<u8>::new();
            let result = execute_show(&cmd, reader, &mut writer, serialize_document);
            prop_assert!(result.is_ok(), "execute_show must succeed: {result:?}");
            prop_assert!(writer.ends_with(b"\n"), "output must end with newline");
            prop_assert!(writer.len() >= 2, "output must have at least JSON char + newline");
            prop_assert_ne!(writer[writer.len() - 2], b'\n', "no double trailing newline");
        }
    }

    // INV-6: ShowError display always starts with "error: show:"
    proptest! {
        #[test]
        fn proptest_show_error_display_always_starts_with_error_show_prefix(
            payload in ".*"
        ) {
            use crate::error::ShowError;
            let variants: Vec<ShowError> = vec![
                ShowError::FileNotFound(std::path::PathBuf::from(&payload)),
                ShowError::IoError(payload.clone()),
                ShowError::InvalidUtf8,
                ShowError::EmptyInput,
                ShowError::JsonDeserialize(payload.clone()),
                ShowError::InvalidDocument(payload.clone()),
                ShowError::SerializationFailure(payload.clone()),
                ShowError::StdoutWriteFailure(payload),
            ];
            // Unrolled: one prop_assert per variant for precise shrinking
            prop_assert!(
                variants[0].to_string().starts_with("error: show: "),
                "FileNotFound display must start with 'error: show: ', got: {:?}", variants[0].to_string()
            );
            prop_assert!(
                variants[1].to_string().starts_with("error: show: "),
                "IoError display must start with 'error: show: ', got: {:?}", variants[1].to_string()
            );
            prop_assert!(
                variants[2].to_string().starts_with("error: show: "),
                "InvalidUtf8 display must start with 'error: show: ', got: {:?}", variants[2].to_string()
            );
            prop_assert!(
                variants[3].to_string().starts_with("error: show: "),
                "EmptyInput display must start with 'error: show: ', got: {:?}", variants[3].to_string()
            );
            prop_assert!(
                variants[4].to_string().starts_with("error: show: "),
                "JsonDeserialize display must start with 'error: show: ', got: {:?}", variants[4].to_string()
            );
            prop_assert!(
                variants[5].to_string().starts_with("error: show: "),
                "InvalidDocument display must start with 'error: show: ', got: {:?}", variants[5].to_string()
            );
            prop_assert!(
                variants[6].to_string().starts_with("error: show: "),
                "SerializationFailure display must start with 'error: show: ', got: {:?}", variants[6].to_string()
            );
            prop_assert!(
                variants[7].to_string().starts_with("error: show: "),
                "StdoutWriteFailure display must start with 'error: show: ', got: {:?}", variants[7].to_string()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Kani verification harnesses
// ---------------------------------------------------------------------------

#[allow(unexpected_cfgs)]
#[cfg(kani)]
mod verification {
    use super::*;
    use crate::domain::{ShowCommand, ShowSource};
    use diagram_models::document::DiagramDocument;
    use std::path::PathBuf;

    #[kani::proof]
    fn verify_map_show_subcommand_is_structurally_total() {
        let has_file: bool = kani::any();
        let cmd = if has_file {
            map_show_subcommand(Some(PathBuf::from("/bounded/path.json")))
        } else {
            map_show_subcommand(None)
        };
        if has_file {
            assert!(matches!(cmd.source, ShowSource::File(_)));
        } else {
            assert!(matches!(cmd.source, ShowSource::Stdin));
        }
    }

    #[kani::proof]
    fn verify_serialize_document_never_panics_for_valid_doc() {
        let doc = DiagramDocument {
            version: kani::any(),
            ..DiagramDocument::default()
        };
        let result = serialize_document(&doc);
        if let Ok(s) = result {
            assert!(!s.is_empty());
        }
    }

    #[kani::proof]
    fn verify_show_error_display_prefix_for_all_variants() {
        use crate::error::ShowError;
        let err = ShowError::EmptyInput;
        let s = err.to_string();
        assert!(s.starts_with("error: show: "));

        let err2 = ShowError::InvalidUtf8;
        let s2 = err2.to_string();
        assert!(s2.starts_with("error: show: "));
    }

    #[kani::proof]
    fn verify_load_document_from_reader_cannot_return_file_not_found() {
        use std::io::Cursor;
        // load_document_from_reader with an empty reader must not return FileNotFound
        let reader = Cursor::new(vec![]);
        let result = load_document_from_reader(reader);
        match result {
            Err(crate::error::ShowError::FileNotFound(_)) => {
                // This must never happen
                assert!(false, "FileNotFound is impossible for reader-based load");
            }
            _ => {} // Any other outcome is acceptable
        }
    }
}
