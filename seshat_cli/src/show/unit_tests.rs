//! Unit tests for the `show` module (Calc layer — pure functions).

#![allow(clippy::unwrap_used)]

use crate::domain::{ShowCommand, ShowSource};
use crate::error::{ExecutionError, ShowError};
use std::path::PathBuf;

use super::*;

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
