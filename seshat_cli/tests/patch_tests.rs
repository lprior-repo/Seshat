#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use rstest::rstest;
use seshat_cli::domain::{PatchCommand, PatchSource, PatchTarget};
use seshat_cli::error::PatchError;
use seshat_cli::patch::{execute_patch, map_patch_subcommand};
use std::path::PathBuf;

#[test]
fn map_patch_subcommand_maps_correctly_with_files() {
    let input = Some(PathBuf::from("in.json"));
    let patch = PathBuf::from("patch.json");
    let output = Some(PathBuf::from("out.json"));

    let cmd = map_patch_subcommand(input, patch.clone(), output);

    assert_eq!(cmd.input, PatchSource::File(PathBuf::from("in.json")));
    assert_eq!(cmd.patch, patch);
    assert_eq!(cmd.output, PatchTarget::File(PathBuf::from("out.json")));
}

#[test]
fn map_patch_subcommand_maps_correctly_with_stdin_stdout() {
    let patch = PathBuf::from("patch.json");

    let cmd = map_patch_subcommand(None, patch.clone(), None);

    assert_eq!(cmd.input, PatchSource::Stdin);
    assert_eq!(cmd.patch, patch);
    assert_eq!(cmd.output, PatchTarget::Stdout);
}

#[rstest]
fn execute_patch_fails_if_patch_lacks_revision_test() {
    let dir = tempfile::tempdir().expect("tempdir creation");
    let input_path = dir.path().join("in.json");
    let patch_path = dir.path().join("patch.json");

    let valid_doc = r#"{
        "version": 2,
        "revision": 1,
        "document": {
            "nodes": {},
            "edges": {}
        }
    }"#;
    std::fs::write(&input_path, valid_doc).expect("write input file");

    let patch = r#"[
        { "op": "add", "path": "/nodes/0", "value": { "id": "n1", "kind": "default", "position": { "x": 0, "y": 0 } } }
    ]"#;
    std::fs::write(&patch_path, patch).expect("write patch file");

    let cmd = PatchCommand {
        input: PatchSource::File(input_path),
        patch: patch_path,
        output: PatchTarget::Stdout,
    };

    let mut out = Vec::new();
    let result = execute_patch(&cmd, std::io::empty(), &mut out);

    assert_eq!(result, Err(PatchError::MissingRevisionTest));
}

#[rstest]
fn execute_patch_fails_if_test_operation_fails() {
    let dir = tempfile::tempdir().expect("tempdir creation");
    let input_path = dir.path().join("in.json");
    let patch_path = dir.path().join("patch.json");

    let valid_doc = r#"{
        "version": 2,
        "revision": 1,
        "document": {
            "nodes": {},
            "edges": {}
        }
    }"#;
    std::fs::write(&input_path, valid_doc).expect("write input file");

    let patch = r#"[
        { "op": "test", "path": "/revision", "value": 2 },
        { "op": "add", "path": "/nodes/0", "value": { "id": "n1", "kind": "default", "position": { "x": 0, "y": 0 } } }
    ]"#;
    std::fs::write(&patch_path, patch).expect("write patch file");

    let cmd = PatchCommand {
        input: PatchSource::File(input_path),
        patch: patch_path,
        output: PatchTarget::Stdout,
    };

    let mut out = Vec::new();
    let result = execute_patch(&cmd, std::io::empty(), &mut out);

    assert!(result.is_ok());
    let diff: serde_json::Value = serde_json::from_slice(&out).expect("parse stdout json");
    assert_eq!(diff["status"], "rejected");
    assert_eq!(diff["conflict_context"]["expected_revision"], 2);
    assert_eq!(diff["conflict_context"]["actual_revision"], 1);
}

#[rstest]
fn execute_patch_fails_if_invalid_document_produced() {
    let dir = tempfile::tempdir().expect("tempdir creation");
    let input_path = dir.path().join("in.json");
    let patch_path = dir.path().join("patch.json");

    let valid_doc = r#"{
        "version": 2,
        "revision": 1,
        "document": {
            "nodes": {},
            "edges": {}
        }
    }"#;
    std::fs::write(&input_path, valid_doc).expect("write input file");

    // Removing required field "version"
    let patch = r#"[
        { "op": "test", "path": "/revision", "value": 1 },
        { "op": "remove", "path": "/version" }
    ]"#;
    std::fs::write(&patch_path, patch).expect("write patch file");

    let cmd = PatchCommand {
        input: PatchSource::File(input_path),
        patch: patch_path,
        output: PatchTarget::Stdout,
    };

    let mut out = Vec::new();
    let result = execute_patch(&cmd, std::io::empty(), &mut out);

    assert!(matches!(result, Err(PatchError::InvalidDocument(_))));
}

#[rstest]
fn execute_patch_succeeds_and_increments_revision() {
    let dir = tempfile::tempdir().expect("tempdir creation");
    let input_path = dir.path().join("in.json");
    let patch_path = dir.path().join("patch.json");
    let output_path = dir.path().join("out.json");

    let valid_doc = r#"{
        "version": 2,
        "revision": 1,
        "document": {
            "nodes": {},
            "edges": {}
        }
    }"#;
    std::fs::write(&input_path, valid_doc).expect("write input file");

    let patch = r#"[
        { "op": "test", "path": "/revision", "value": 1 }
    ]"#;
    std::fs::write(&patch_path, patch).expect("write patch file");

    let cmd = PatchCommand {
        input: PatchSource::File(input_path),
        patch: patch_path,
        output: PatchTarget::File(output_path.clone()),
    };

    let mut out = Vec::new();
    let result = execute_patch(&cmd, std::io::empty(), &mut out);
    assert_eq!(result, Ok(()));

    let output_str = std::fs::read_to_string(output_path).expect("read output file");
    let doc: serde_json::Value = serde_json::from_str(&output_str).expect("parse output json");
    assert_eq!(doc["revision"], 2);
}
