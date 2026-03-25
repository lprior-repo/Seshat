//! End-to-end tests for the `seshat` binary — `patch` subcommand.
//!
//! These tests spawn the compiled `seshat` binary via `std::process::Command`
//! and validate exit codes, stdout bytes, and stderr output.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use diagram_models::document::DiagramDocument;
use std::io::Write;
use std::process::Command;

fn seshat_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seshat"))
}

fn write_to_temp_file(contents: &[u8]) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("temp file creation must succeed");
    f.write_all(contents)
        .expect("write to temp file must succeed");
    f
}

#[test]
fn e2e_patch_exits_zero_and_increments_revision_on_success() {
    let expected_doc = DiagramDocument::default();
    let json = serde_json::to_string(&expected_doc).expect("must serialize");
    let input_temp = write_to_temp_file(json.as_bytes());

    let patch_json = r#"[
        { "op": "test", "path": "/revision", "value": 0 }
    ]"#;
    let patch_temp = write_to_temp_file(patch_json.as_bytes());

    let output = seshat_bin()
        .arg("patch")
        .arg("--input")
        .arg(input_temp.path())
        .arg("--patch")
        .arg(patch_temp.path())
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(0),
        "exit code must be 0 for valid patch; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output.stderr.is_empty(),
        "stderr must be empty on success, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout_str = String::from_utf8(output.stdout).expect("stdout must be valid UTF-8");
    let result_doc: serde_json::Value =
        serde_json::from_str(&stdout_str).expect("output must be valid JSON");

    assert_eq!(
        result_doc["revision"], 1,
        "revision must be incremented to 1"
    );
}

#[test]
fn e2e_patch_exits_one_on_missing_revision_test() {
    let expected_doc = DiagramDocument::default();
    let json = serde_json::to_string(&expected_doc).expect("must serialize");
    let input_temp = write_to_temp_file(json.as_bytes());

    let patch_json = r#"[
        { "op": "add", "path": "/document/nodes/n1", "value": { "id": "n1", "kind": "default", "position": { "x": 0.0, "y": 0.0 } } }
    ]"#;
    let patch_temp = write_to_temp_file(patch_json.as_bytes());

    let output = seshat_bin()
        .arg("patch")
        .arg("--input")
        .arg(input_temp.path())
        .arg("--patch")
        .arg(patch_temp.path())
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(1),
        "exit code must be 1 when revision test is missing; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("error: patch: patch must include a test for /revision"),
        "stderr must contain proper error message, got: {stderr_str:?}"
    );
}

#[test]
fn e2e_patch_exits_two_on_missing_patch_argument() {
    let output = seshat_bin()
        .arg("patch")
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(2),
        "exit code must be 2 for missing required arguments; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("error") || stderr_str.contains("required"),
        "stderr must contain clap error text, got: {stderr_str:?}"
    );
}
