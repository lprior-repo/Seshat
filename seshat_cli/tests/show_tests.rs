//! Integration tests for the `seshat show` subcommand.
//!
//! These tests exercise the public API of `seshat_cli` through real filesystem
//! fixtures, injected readers/writers, and the full parse_args → execute_show pipeline.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::similar_names)]
#![allow(clippy::doc_markdown)]

use std::io::Cursor;
use std::path::PathBuf;

use seshat_cli::error::ShowError;
use seshat_cli::{
    execute_show, load_document_from_path, load_document_from_reader, parse_args,
    serialize_document, ShowCommand, ShowSource,
};

use diagram_models::document::DiagramDocument;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn make_default_doc() -> DiagramDocument {
    DiagramDocument::default()
}

fn make_default_doc_json() -> String {
    serde_json::to_string(&make_default_doc()).expect("default doc must serialize in test helper")
}

fn make_doc_with_version(version: u32) -> DiagramDocument {
    DiagramDocument {
        version,
        ..DiagramDocument::default()
    }
}

fn write_to_temp_file(contents: &[u8]) -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().expect("temp file creation must succeed");
    f.write_all(contents)
        .expect("write to temp file must succeed");
    f
}

/// A writer where write() and write_all() always return a BrokenPipe error.
struct FailingWriter;

impl std::io::Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "broken",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "broken",
        ))
    }
}

/// A writer where write() succeeds but flush() always fails with BrokenPipe.
struct FlushFailingWriter {
    buf: Vec<u8>,
}

impl FlushFailingWriter {
    const fn new() -> Self {
        Self { buf: Vec::new() }
    }
}

impl std::io::Write for FlushFailingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "broken",
        ))
    }
}

/// A writer that tracks all bytes written to it (for INV-1).
struct TrackingWriter {
    pub bytes: Vec<u8>,
}

impl TrackingWriter {
    const fn new() -> Self {
        Self { bytes: Vec::new() }
    }
}

impl std::io::Write for TrackingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// parse_args integration tests (B-01, B-02, B-03)
// ---------------------------------------------------------------------------

#[test]
fn parse_args_returns_show_file_command_when_file_and_json_flags_provided() {
    use seshat_cli::{Cli, Subcommand};
    use std::ffi::OsString;

    let args: Vec<OsString> = vec![
        "seshat".into(),
        "show".into(),
        "--file".into(),
        "/tmp/diagram.json".into(),
        "--json".into(),
    ];
    let result = parse_args(args.into_iter());
    assert_eq!(
        result,
        Ok(Cli::Run(Subcommand::Show(ShowCommand {
            source: ShowSource::File(PathBuf::from("/tmp/diagram.json")),
        })))
    );
}

#[test]
fn parse_args_returns_show_stdin_command_when_only_json_flag_provided() {
    use seshat_cli::{Cli, Subcommand};
    use std::ffi::OsString;

    let args: Vec<OsString> = vec!["seshat".into(), "show".into(), "--json".into()];
    let result = parse_args(args.into_iter());
    assert_eq!(
        result,
        Ok(Cli::Run(Subcommand::Show(ShowCommand {
            source: ShowSource::Stdin,
        })))
    );
}

#[test]
fn parse_args_returns_clap_error_when_show_invoked_without_json_flag() {
    use seshat_cli::error::{Error, ParseError};
    use std::ffi::OsString;

    let args: Vec<OsString> = vec!["seshat".into(), "show".into()];
    let result = parse_args(args.into_iter());
    match result {
        Err(Error::ArgumentParse(ParseError::Clap(msg))) => {
            assert!(
                msg.contains("required") || msg.contains("--json") || msg.contains("json"),
                "error message must mention missing --json flag, got: {msg:?}"
            );
        }
        other => panic!("expected Err(ArgumentParse(Clap(_))), got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// B-04: empty file path results in FileNotFound or IoError
// ---------------------------------------------------------------------------

#[test]
fn execute_show_returns_file_not_found_or_io_error_when_file_path_is_empty_string() {
    let cmd = ShowCommand {
        source: ShowSource::File(PathBuf::from("")),
    };
    let reader = Cursor::new(vec![]);
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    match result {
        Err(ShowError::FileNotFound(_) | ShowError::IoError(_)) => {}
        other => panic!("expected FileNotFound or IoError for empty path, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// load_document_from_path integration tests
// ---------------------------------------------------------------------------

#[test]
fn load_document_from_path_returns_document_when_valid_json_file_provided() {
    let expected_doc = make_default_doc();
    let json = make_default_doc_json();
    let temp = write_to_temp_file(json.as_bytes());
    let result = load_document_from_path(temp.path());
    assert_eq!(result, Ok(expected_doc));
}

#[test]
fn load_document_from_path_returns_file_not_found_when_path_does_not_exist() {
    let nonexistent = PathBuf::from("/tmp/seshat-gkc-test-nonexistent-xxxxxxxxxx.json");
    let result = load_document_from_path(&nonexistent);
    assert_eq!(result, Err(ShowError::FileNotFound(nonexistent)));
}

#[cfg(unix)]
#[test]
fn load_document_from_path_returns_file_not_found_when_read_permission_denied() {
    use std::os::unix::fs::PermissionsExt;
    let json = make_default_doc_json();
    let temp = write_to_temp_file(json.as_bytes());
    let path = temp.path().to_path_buf();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000))
        .expect("setting permissions must succeed in test");
    let result = load_document_from_path(&path);
    // Restore permissions so tempfile cleanup works
    let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));
    assert_eq!(result, Err(ShowError::FileNotFound(path)));
}

#[test]
fn load_document_from_path_returns_io_error_when_path_is_directory() {
    let dir = tempfile::tempdir().expect("temp dir creation must succeed");
    let result = load_document_from_path(dir.path());
    match result {
        Err(ShowError::IoError(msg)) => {
            assert!(
                msg.contains("Is a directory") || msg.contains("21") || msg.contains("directory"),
                "IoError message must mention directory issue, got: {msg:?}"
            );
        }
        other => panic!("expected Err(ShowError::IoError(_)) for directory path, got: {other:?}"),
    }
}

#[test]
fn load_document_from_path_returns_empty_input_when_file_is_zero_bytes() {
    let temp = write_to_temp_file(b"");
    let result = load_document_from_path(temp.path());
    assert_eq!(result, Err(ShowError::EmptyInput));
}

#[test]
fn load_document_from_path_returns_empty_input_when_file_contains_only_whitespace() {
    let temp = write_to_temp_file(b"\n\n  \t\n");
    let result = load_document_from_path(temp.path());
    assert_eq!(result, Err(ShowError::EmptyInput));
}

#[test]
fn load_document_from_path_returns_empty_input_when_file_contains_single_space() {
    let temp = write_to_temp_file(b" ");
    let result = load_document_from_path(temp.path());
    assert_eq!(result, Err(ShowError::EmptyInput));
}

#[test]
fn load_document_from_path_returns_invalid_utf8_when_file_contains_binary_bytes() {
    let temp = write_to_temp_file(&[0xFF, 0xFE, 0x80, 0x00]);
    let result = load_document_from_path(temp.path());
    assert_eq!(result, Err(ShowError::InvalidUtf8));
}

#[test]
fn load_document_from_path_returns_json_deserialize_when_file_contains_malformed_json() {
    let temp = write_to_temp_file(b"{\"version\": 2, \"broken\": [}");
    let result = load_document_from_path(temp.path());
    match result {
        Err(ShowError::JsonDeserialize(msg)) => {
            assert!(
                msg.contains("expected") || msg.contains("line") || msg.contains("column"),
                "JsonDeserialize message must contain position info, got: {msg:?}"
            );
        }
        other => {
            panic!("expected Err(ShowError::JsonDeserialize(_)) for malformed JSON, got: {other:?}")
        }
    }
}

#[test]
fn load_document_from_path_returns_json_deserialize_when_file_contains_truncated_json() {
    let temp = write_to_temp_file(b"{\"version\": 2, \"revision\": 0, \"document\": {\"nodes\":");
    let result = load_document_from_path(temp.path());
    match result {
        Err(ShowError::JsonDeserialize(msg)) => {
            assert!(
                msg.contains("EOF")
                    || msg.contains("expected")
                    || msg.contains("line")
                    || msg.contains("eof"),
                "JsonDeserialize message must indicate truncated input, got: {msg:?}"
            );
        }
        other => {
            panic!("expected Err(ShowError::JsonDeserialize(_)) for truncated JSON, got: {other:?}")
        }
    }
}

#[test]
fn load_document_from_path_returns_invalid_document_when_json_contains_unknown_fields() {
    // Build a valid DiagramDocument JSON then inject an unknown field
    let doc = make_default_doc();
    let valid_json = serde_json::to_string(&doc).expect("must serialize");
    // Insert unknown field: strip trailing "}" and add unknown field
    let with_unknown = format!(
        "{},\"unknown_field\":true}}",
        &valid_json[..valid_json.len() - 1]
    );
    let temp = write_to_temp_file(with_unknown.as_bytes());
    let result = load_document_from_path(temp.path());
    match result {
        Err(ShowError::InvalidDocument(msg)) => {
            assert!(
                msg.contains("unknown field"),
                "InvalidDocument message must contain 'unknown field', got: {msg:?}"
            );
        }
        other => panic!(
            "expected Err(ShowError::InvalidDocument(_)) for JSON with unknown fields, got: {other:?}"
        ),
    }
}

#[test]
fn load_document_from_path_returns_document_when_document_has_zero_nodes_and_edges() {
    let expected_doc = make_default_doc();
    let json = serde_json::to_string(&expected_doc).expect("must serialize");
    let temp = write_to_temp_file(json.as_bytes());
    let result = load_document_from_path(temp.path());
    assert_eq!(result, Ok(expected_doc));
}

#[test]
fn load_document_from_path_returns_document_when_version_is_u32_max() {
    let doc = make_doc_with_version(u32::MAX);
    let json = serde_json::to_string(&doc).expect("must serialize");
    let temp = write_to_temp_file(json.as_bytes());
    let result = load_document_from_path(temp.path());
    match result {
        Ok(loaded) => assert_eq!(
            loaded.version,
            u32::MAX,
            "loaded document must have version == u32::MAX"
        ),
        Err(e) => panic!("expected Ok(doc) for u32::MAX version, got: {e:?}"),
    }
}

#[test]
fn load_document_from_path_returns_ok_when_edges_reference_nonexistent_node_ids() {
    // Build a document JSON with an orphaned edge (source/target not in nodes map)
    let orphan_json = r#"{
        "version": 0,
        "revision": 0,
        "document": {
            "nodes": {},
            "edges": {
                "e1": {
                    "source": "node-missing",
                    "target": "node-also-missing",
                    "label": "",
                    "style": "solid",
                    "arrowType": "default",
                    "label_offset_t": 0.5,
                    "thickness": 1.5,
                    "directed": true
                }
            }
        }
    }"#;
    let temp = write_to_temp_file(orphan_json.as_bytes());
    let result = load_document_from_path(temp.path());
    assert!(
        result.is_ok(),
        "show must not validate graph integrity — orphaned edge reference must be accepted: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// load_document_from_reader integration tests
// ---------------------------------------------------------------------------

#[test]
fn load_document_from_reader_returns_document_when_cursor_contains_valid_json() {
    let expected_doc = make_default_doc();
    let json = make_default_doc_json();
    let reader = Cursor::new(json.into_bytes());
    let result = load_document_from_reader(reader);
    assert_eq!(result, Ok(expected_doc));
}

#[test]
fn load_document_from_reader_returns_empty_input_when_reader_yields_zero_bytes() {
    let reader = Cursor::new(vec![]);
    let result = load_document_from_reader(reader);
    assert_eq!(result, Err(ShowError::EmptyInput));
}

#[test]
fn load_document_from_reader_returns_invalid_utf8_when_reader_yields_non_utf8_bytes() {
    let reader = Cursor::new(vec![0xFF_u8, 0xFE, 0x80]);
    let result = load_document_from_reader(reader);
    assert_eq!(result, Err(ShowError::InvalidUtf8));
}

#[test]
fn load_document_from_reader_returns_json_deserialize_when_reader_yields_plain_text() {
    let reader = Cursor::new(b"this is not json at all".to_vec());
    let result = load_document_from_reader(reader);
    match result {
        Err(ShowError::JsonDeserialize(msg)) => {
            assert!(
                msg.contains("expected") || msg.contains("line") || msg.contains("column"),
                "JsonDeserialize message must contain position info, got: {msg:?}"
            );
        }
        other => {
            panic!("expected Err(ShowError::JsonDeserialize(_)) for plain text, got: {other:?}")
        }
    }
}

#[test]
fn load_document_from_reader_returns_empty_input_when_reader_yields_only_whitespace() {
    let reader = Cursor::new(b"   \n\t\n".to_vec());
    let result = load_document_from_reader(reader);
    assert_eq!(result, Err(ShowError::EmptyInput));
}

#[test]
fn load_document_from_reader_returns_invalid_document_when_json_has_unknown_fields() {
    let doc = make_default_doc();
    let valid_json = serde_json::to_string(&doc).expect("must serialize");
    let with_unknown = format!(
        "{},\"unknown_field\":true}}",
        &valid_json[..valid_json.len() - 1]
    );
    let reader = Cursor::new(with_unknown.into_bytes());
    let result = load_document_from_reader(reader);
    match result {
        Err(ShowError::InvalidDocument(msg)) => {
            assert!(
                msg.contains("unknown field"),
                "InvalidDocument message must contain 'unknown field', got: {msg:?}"
            );
        }
        other => panic!(
            "expected Err(ShowError::InvalidDocument(_)) for JSON with unknown fields, got: {other:?}"
        ),
    }
}

#[test]
fn load_document_from_reader_returns_document_when_version_is_zero() {
    let doc = make_doc_with_version(0);
    let json = serde_json::to_string(&doc).expect("must serialize");
    let reader = Cursor::new(json.into_bytes());
    let result = load_document_from_reader(reader);
    match result {
        Ok(loaded) => assert_eq!(loaded.version, 0u32, "loaded doc must have version == 0"),
        Err(e) => panic!("expected Ok(doc) for version 0, got: {e:?}"),
    }
}

// ---------------------------------------------------------------------------
// execute_show integration tests
// ---------------------------------------------------------------------------

#[test]
fn execute_show_writes_json_followed_by_newline_when_source_is_valid_file() {
    let expected_doc = make_default_doc();
    let json = serde_json::to_string(&expected_doc).expect("must serialize");
    let temp = write_to_temp_file(json.as_bytes());

    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(vec![]);
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    assert!(result.is_ok(), "execute_show must succeed: {result:?}");

    let content = String::from_utf8(writer.clone());
    assert!(content.is_ok(), "output must be valid UTF-8: {content:?}");
    let content = content.expect("output must be valid UTF-8");
    assert!(
        content.ends_with('\n'),
        "output must end with exactly one newline, got: {content:?}"
    );
    let json_part = &content[..content.len() - 1];
    let expected_json = serialize_document(&expected_doc);
    assert!(
        expected_json.is_ok(),
        "expected doc must serialize: {expected_json:?}"
    );
    assert_eq!(
        json_part,
        expected_json.expect("serialize expected document").as_str()
    );
}

#[test]
fn execute_show_writes_json_followed_by_newline_when_source_is_valid_stdin() {
    let doc = make_default_doc();
    let json = serde_json::to_string(&doc).expect("must serialize");

    let cmd = ShowCommand {
        source: ShowSource::Stdin,
    };
    let reader = Cursor::new(json.as_bytes().to_vec());
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    assert!(result.is_ok(), "execute_show must succeed: {result:?}");

    let expected_bytes: Vec<u8> = json.as_bytes().to_vec();
    // The writer must contain the serialized JSON + newline
    // The serialized form may differ in whitespace, so parse and re-serialize
    let expected_json = serialize_document(&doc).expect("doc must serialize");
    let mut expected_output = expected_json.into_bytes();
    expected_output.push(b'\n');
    assert_eq!(
        writer, expected_output,
        "writer bytes must equal JSON bytes + newline"
    );
    // Suppress unused variable warning
    drop(expected_bytes);
}

#[test]
fn execute_show_returns_stdout_write_failure_when_writer_returns_error() {
    let doc = make_default_doc();
    let json = serde_json::to_string(&doc).expect("must serialize");
    let temp = write_to_temp_file(json.as_bytes());

    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(vec![]);
    let result = execute_show(&cmd, reader, FailingWriter, serialize_document);
    match result {
        Err(ShowError::StdoutWriteFailure(msg)) => {
            assert!(
                msg.contains("broken"),
                "StdoutWriteFailure message must contain 'broken', got: {msg:?}"
            );
        }
        other => panic!("expected Err(ShowError::StdoutWriteFailure(_)), got: {other:?}"),
    }
}

#[test]
fn execute_show_returns_stdout_write_failure_when_flush_returns_error() {
    let doc = make_default_doc();
    let json = serde_json::to_string(&doc).expect("must serialize");
    let temp = write_to_temp_file(json.as_bytes());

    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(vec![]);
    let result = execute_show(&cmd, reader, FlushFailingWriter::new(), serialize_document);
    match result {
        Err(ShowError::StdoutWriteFailure(msg)) => {
            assert!(
                msg.contains("broken"),
                "StdoutWriteFailure message (flush) must contain 'broken', got: {msg:?}"
            );
        }
        other => panic!(
            "expected Err(ShowError::StdoutWriteFailure(_)) for flush failure, got: {other:?}"
        ),
    }
}

// B-28a: FileNotFound propagated
#[test]
fn execute_show_returns_file_not_found_when_file_source_path_does_not_exist() {
    let nonexistent = PathBuf::from("/tmp/seshat-gkc-nonexistent-xxxxxxxxx.json");
    let cmd = ShowCommand {
        source: ShowSource::File(nonexistent.clone()),
    };
    let reader = Cursor::new(vec![]);
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    assert_eq!(result, Err(ShowError::FileNotFound(nonexistent)));
}

// B-28b: EmptyInput propagated (file)
#[test]
fn execute_show_returns_empty_input_when_file_source_is_empty() {
    let temp = write_to_temp_file(b"");
    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(vec![]);
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    assert_eq!(result, Err(ShowError::EmptyInput));
}

// B-28c: EmptyInput propagated (stdin)
#[test]
fn execute_show_returns_empty_input_when_stdin_source_is_empty() {
    let cmd = ShowCommand {
        source: ShowSource::Stdin,
    };
    let reader = Cursor::new(vec![]);
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    assert_eq!(result, Err(ShowError::EmptyInput));
}

// B-28d: InvalidUtf8 propagated
#[test]
fn execute_show_returns_invalid_utf8_when_file_contains_non_utf8_bytes() {
    let temp = write_to_temp_file(&[0xFF_u8, 0x80]);
    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(vec![]);
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    assert_eq!(result, Err(ShowError::InvalidUtf8));
}

// B-28e: JsonDeserialize propagated
#[test]
fn execute_show_returns_json_deserialize_when_file_contains_invalid_json() {
    let temp = write_to_temp_file(b"not json");
    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(vec![]);
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    match result {
        Err(ShowError::JsonDeserialize(msg)) => {
            assert!(
                msg.contains("expected") || msg.contains("line") || msg.contains("column"),
                "JsonDeserialize message must contain position info, got: {msg:?}"
            );
        }
        other => panic!("expected Err(ShowError::JsonDeserialize(_)), got: {other:?}"),
    }
}

// B-28f: InvalidDocument propagated
#[test]
fn execute_show_returns_invalid_document_when_file_json_has_unknown_fields() {
    let doc = make_default_doc();
    let valid_json = serde_json::to_string(&doc).expect("must serialize");
    let with_unknown = format!(
        "{},\"unknown_field\":true}}",
        &valid_json[..valid_json.len() - 1]
    );
    let temp = write_to_temp_file(with_unknown.as_bytes());
    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(vec![]);
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    match result {
        Err(ShowError::InvalidDocument(msg)) => {
            assert!(
                msg.contains("unknown field"),
                "InvalidDocument message must contain 'unknown field', got: {msg:?}"
            );
        }
        other => panic!("expected Err(ShowError::InvalidDocument(_)), got: {other:?}"),
    }
}

// B-28g: SerializationFailure propagated through execute_show (injection via serialize_fn param)
#[test]
fn execute_show_propagates_serialization_failure_from_serialize_document() {
    let doc = make_default_doc();
    let json = serde_json::to_string(&doc).expect("must serialize");

    let cmd = ShowCommand {
        source: ShowSource::Stdin,
    };
    let reader = Cursor::new(json.into_bytes());
    let mut writer = Vec::<u8>::new();

    // Inject a serialize_fn that always fails
    let failing_serialize = |_: &DiagramDocument| -> Result<String, ShowError> {
        Err(ShowError::SerializationFailure("injected".to_string()))
    };

    let result = execute_show(&cmd, reader, &mut writer, failing_serialize);
    match result {
        Err(ShowError::SerializationFailure(msg)) => {
            assert!(
                msg.contains("injected"),
                "SerializationFailure message must contain 'injected', got: {msg:?}"
            );
        }
        other => panic!("expected Err(ShowError::SerializationFailure(_)), got: {other:?}"),
    }
}

// EC-15: file source takes precedence over stdin
#[test]
fn execute_show_uses_file_source_and_ignores_stdin_when_file_path_is_provided() {
    let doc_a = make_doc_with_version(1);
    let doc_b = make_doc_with_version(2);

    let json_a = serde_json::to_string(&doc_a).expect("must serialize");
    let json_b = serde_json::to_string(&doc_b).expect("must serialize");

    let temp = write_to_temp_file(json_a.as_bytes());
    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(json_b.into_bytes()); // doc_b in stdin — must be ignored
    let mut writer = Vec::<u8>::new();
    let result = execute_show(&cmd, reader, &mut writer, serialize_document);
    assert!(result.is_ok(), "execute_show must succeed: {result:?}");

    let content = String::from_utf8(writer).expect("output must be valid UTF-8");
    let loaded: DiagramDocument = serde_json::from_str(content.trim_end_matches('\n'))
        .expect("output must be valid DiagramDocument JSON");
    assert_eq!(
        loaded.version, 1,
        "output must be from file (version 1), not stdin (version 2)"
    );
}

// B-INV1: execute_show writes JSON only to injected writer
#[test]
fn execute_show_writes_json_only_to_injected_writer_when_source_is_valid() {
    let expected_doc = make_default_doc();
    let json = serde_json::to_string(&expected_doc).expect("must serialize");
    let temp = write_to_temp_file(json.as_bytes());

    let cmd = ShowCommand {
        source: ShowSource::File(temp.path().to_path_buf()),
    };
    let reader = Cursor::new(vec![]);
    let mut tracking_writer = TrackingWriter::new();
    let result = execute_show(&cmd, reader, &mut tracking_writer, serialize_document);
    assert!(result.is_ok(), "execute_show must succeed: {result:?}");

    assert!(
        !tracking_writer.bytes.is_empty(),
        "tracking writer must have received bytes"
    );
    // Last byte must be newline
    assert_eq!(
        tracking_writer.bytes.last(),
        Some(&b'\n'),
        "output must end with newline"
    );
    // The bytes minus trailing newline must be valid DiagramDocument JSON
    let json_bytes = &tracking_writer.bytes[..tracking_writer.bytes.len() - 1];
    let parsed = serde_json::from_slice::<DiagramDocument>(json_bytes);
    assert!(
        parsed.is_ok(),
        "bytes in tracking writer must parse as DiagramDocument: {parsed:?}"
    );
}

// ---------------------------------------------------------------------------
// Type-mismatch tests: wrong-type field → JsonDeserialize (NOT InvalidDocument)
// Kills the surviving mutant: `e.is_data() && e.to_string().contains("unknown field")`
// When version is a string instead of u32, serde produces is_data()==true but
// the message does NOT contain "unknown field". The correct error is JsonDeserialize.
// ---------------------------------------------------------------------------

#[test]
fn load_document_from_reader_returns_json_deserialize_when_field_has_wrong_type() {
    // version is "not_a_number" (string) instead of u32 — valid JSON, wrong type
    let json = r#"{"version": "not_a_number", "nodes": {}, "edges": {}, "metadata": {}}"#;
    let reader = std::io::Cursor::new(json.as_bytes());
    let result = load_document_from_reader(reader);
    assert!(
        matches!(result, Err(ShowError::JsonDeserialize(_))),
        "expected JsonDeserialize for wrong-type field, got: {result:?}"
    );
}

#[test]
fn load_document_from_path_returns_json_deserialize_when_field_has_wrong_type() {
    // version is "not_a_number" (string) instead of u32 — valid JSON, wrong type
    let json = r#"{"version": "not_a_number", "nodes": {}, "edges": {}, "metadata": {}}"#;
    let mut temp = tempfile::NamedTempFile::new().expect("temp file");
    std::io::Write::write_all(&mut temp, json.as_bytes()).expect("write");
    let result = load_document_from_path(temp.path());
    assert!(
        matches!(result, Err(ShowError::JsonDeserialize(_))),
        "expected JsonDeserialize for wrong-type field, got: {result:?}"
    );
}

#[test]
fn load_document_from_reader_returns_error_for_stream_exceeding_size_limit() {
    // Verifies that an extremely large stream (simulated via repeated null bytes)
    // does NOT hang indefinitely and returns a graceful error.
    // Uses 100 MiB of null bytes, well above the 64 MiB MAX_INPUT_BYTES limit.
    let large_input = vec![0u8; 100 * 1024 * 1024];
    let reader = std::io::Cursor::new(large_input);
    // Must complete (not hang) and return an error (null bytes are not valid UTF-8/JSON)
    let result = load_document_from_reader(reader);
    assert!(
        result.is_err(),
        "expected error for oversized/invalid stream, got: {result:?}"
    );
    // Must not be a hang — if we reach here, the size limit worked
}
