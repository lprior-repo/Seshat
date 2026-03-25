//! End-to-end tests for the `seshat` binary — `apply` subcommand.
//!
//! These tests spawn the compiled `seshat` binary via `std::process::Command`
//! and validate exit codes, stdout bytes, and stderr output.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

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

/// Valid proposal JSON with the given revision and one change.
fn valid_proposal_json(revision: u64) -> String {
    format!(
        r#"{{
            "base_revision": {revision},
            "proposer": "test-agent",
            "proposed_at": 1700000000,
            "summary": "Test proposal",
            "changes": [
                {{"change_type": "move_node", "node_id": "n1", "new_x": 100.0, "new_y": 200.0}}
            ]
        }}"#
    )
}

/// Valid document JSON with the given revision.
fn valid_document_json(revision: u64) -> String {
    format!(
        r#"{{
            "version": 2,
            "revision": {revision},
            "document": {{"nodes": {{}}, "edges": {{}}}},
            "editor_state": {{"camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0}}
        }}"#
    )
}

// ---------------------------------------------------------------------------
// E2E Behavior 81: Full success
// ---------------------------------------------------------------------------

#[test]
fn e2e_apply_exits_zero_and_writes_queued_json_when_valid() {
    let proposal = write_to_temp_file(valid_proposal_json(1).as_bytes());
    let document = write_to_temp_file(valid_document_json(1).as_bytes());

    let output = seshat_bin()
        .arg("apply")
        .arg("--file")
        .arg(proposal.path())
        .arg("--doc")
        .arg(document.path())
        .output()
        .expect("seshat binary must be spawnable");

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "exit code must be 0 for valid apply; stderr: {stderr_str}"
    );

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout_str.contains(r#""status":"queued""#),
        "stdout must contain queued status, got: {stdout_str}"
    );
}

// ---------------------------------------------------------------------------
// E2E Behavior 82: Stale revision — rejection written to stdout, exit 1
// ---------------------------------------------------------------------------

#[test]
fn e2e_apply_exits_one_and_writes_rejected_json_when_revision_mismatch() {
    let proposal = write_to_temp_file(valid_proposal_json(3).as_bytes());
    let document = write_to_temp_file(valid_document_json(5).as_bytes());

    let output = seshat_bin()
        .arg("apply")
        .arg("--file")
        .arg(proposal.path())
        .arg("--doc")
        .arg(document.path())
        .output()
        .expect("seshat binary must be spawnable");

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(1),
        "exit code must be 1 for rejection (PR6/PV5); stderr: {stderr_str}"
    );

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout_str.contains(r#""status":"rejected""#),
        "stdout must contain rejected status, got: {stdout_str}"
    );
    assert!(
        stdout_str.contains(r#""reason":"Human Priority Block""#),
        "stdout must contain Human Priority Block reason, got: {stdout_str}"
    );
}

// ---------------------------------------------------------------------------
// E2E Behavior 84: Proposal file not found
// ---------------------------------------------------------------------------

#[test]
fn e2e_apply_exits_one_when_proposal_file_not_found() {
    let document = write_to_temp_file(valid_document_json(1).as_bytes());

    let output = seshat_bin()
        .arg("apply")
        .arg("--file")
        .arg("/nonexistent/proposal.json")
        .arg("--doc")
        .arg(document.path())
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(1),
        "exit code must be 1 for missing proposal; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("error:"),
        "stderr must contain error text, got: {stderr_str}"
    );
}

// ---------------------------------------------------------------------------
// E2E Behavior 86: Missing required arguments
// ---------------------------------------------------------------------------

#[test]
fn e2e_apply_exits_two_when_required_arguments_missing() {
    let output = seshat_bin()
        .arg("apply")
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(2),
        "exit code must be 2 for missing --doc; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("error") || stderr_str.contains("required"),
        "stderr must contain clap error text, got: {stderr_str}"
    );
}
