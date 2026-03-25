//! End-to-end tests for the `seshat` binary — `show` subcommand.
//!
//! These tests spawn the compiled `seshat` binary via `std::process::Command`
//! and validate exit codes, stdout bytes, and stderr output.
//!
//! RED PHASE: Tests will FAIL because the implementation is `unimplemented!()`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::redundant_clone)]

use std::io::Write;
use std::process::{Command, Stdio};

use diagram_models::document::DiagramDocument;
use seshat_cli::serialize_document;

fn seshat_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seshat"))
}

#[allow(dead_code)]
fn make_default_doc_json() -> String {
    serde_json::to_string(&DiagramDocument::default()).expect("default doc must serialize")
}

fn write_to_temp_file(contents: &[u8]) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("temp file creation must succeed");
    f.write_all(contents)
        .expect("write to temp file must succeed");
    f
}

// ---------------------------------------------------------------------------
// E2E Scenario 1: Full round-trip success
// ---------------------------------------------------------------------------

#[test]
fn e2e_show_exits_zero_and_writes_json_when_valid_file_provided() {
    let expected_doc = DiagramDocument::default();
    let json = serde_json::to_string(&expected_doc).expect("must serialize");
    let temp = write_to_temp_file(json.as_bytes());

    let output = seshat_bin()
        .arg("show")
        .arg("--file")
        .arg(temp.path())
        .arg("--json")
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(0),
        "exit code must be 0 for valid file; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output.stderr.is_empty(),
        "stderr must be empty on success, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout_str = String::from_utf8(output.stdout).expect("stdout must be valid UTF-8");
    assert!(
        stdout_str.ends_with('\n'),
        "stdout must end with newline, got: {stdout_str:?}"
    );

    let expected_json = serialize_document(&expected_doc).expect("expected doc must serialize");
    let expected_output = format!("{expected_json}\n");
    assert_eq!(
        stdout_str, expected_output,
        "stdout must equal expected JSON + newline"
    );
}

// ---------------------------------------------------------------------------
// E2E Scenario 2: Empty stdin returns exit code 1
// ---------------------------------------------------------------------------

#[test]
fn e2e_show_exits_one_and_writes_to_stderr_when_stdin_is_empty() {
    let output = seshat_bin()
        .arg("show")
        .arg("--json")
        .stdin(Stdio::null()) // /dev/null: empty stdin
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(1),
        "exit code must be 1 for empty stdin; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("error: show:"),
        "stderr must contain 'error: show:', got: {stderr_str:?}"
    );

    assert!(
        output.stdout.is_empty(),
        "stdout must be empty on error, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

// ---------------------------------------------------------------------------
// E2E Scenario 3: Missing --json flag exits with code 2
// ---------------------------------------------------------------------------

#[test]
fn e2e_show_exits_two_when_json_flag_is_absent() {
    let output = seshat_bin()
        .arg("show")
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(2),
        "exit code must be 2 for missing --json flag; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("error") || stderr_str.contains("required"),
        "stderr must contain clap error text, got: {stderr_str:?}"
    );

    assert!(
        output.stdout.is_empty(),
        "stdout must be empty on argument error, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
