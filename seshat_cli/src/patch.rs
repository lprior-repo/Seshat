use crate::domain::{PatchCommand, PatchSource, PatchTarget};
use crate::error::PatchError;
use diagram_models::document::DiagramDocument;
use json_patch::{Patch, PatchOperation};
use std::io::{Read, Write};
use std::path::PathBuf;

#[must_use]
pub fn map_patch_subcommand(
    input: Option<PathBuf>,
    patch: PathBuf,
    output: Option<PathBuf>,
) -> PatchCommand {
    let input_source = input.map_or(PatchSource::Stdin, PatchSource::File);
    let output_target = output.map_or(PatchTarget::Stdout, PatchTarget::File);

    PatchCommand {
        input: input_source,
        patch,
        output: output_target,
    }
}

/// Executes the patch subcommand.
///
/// # Errors
/// Returns `PatchError` on failure.
#[allow(clippy::too_many_lines)]
pub fn execute_patch(
    cmd: &PatchCommand,
    mut stdin: impl Read,
    mut stdout: impl Write,
) -> Result<(), PatchError> {
    let input_str = match &cmd.input {
        PatchSource::File(path) => std::fs::read_to_string(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => PatchError::FileNotFound(path.clone()),
            std::io::ErrorKind::InvalidData => PatchError::InvalidUtf8,
            _ => PatchError::IoError(e.to_string()),
        })?,
        PatchSource::Stdin => {
            let mut buf = String::new();
            stdin.read_to_string(&mut buf).map_err(|e| match e.kind() {
                std::io::ErrorKind::InvalidData => PatchError::InvalidUtf8,
                _ => PatchError::IoError(e.to_string()),
            })?;
            buf
        }
    };

    if input_str.trim().is_empty() {
        return Err(PatchError::EmptyInput);
    }

    let mut doc: serde_json::Value =
        serde_json::from_str(&input_str).map_err(|e| PatchError::JsonDeserialize(e.to_string()))?;

    let patch_str = std::fs::read_to_string(&cmd.patch).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => PatchError::FileNotFound(cmd.patch.clone()),
        std::io::ErrorKind::InvalidData => PatchError::InvalidUtf8,
        _ => PatchError::IoError(e.to_string()),
    })?;

    if patch_str.trim().is_empty() {
        return Err(PatchError::EmptyInput);
    }

    let patch: Patch =
        serde_json::from_str(&patch_str).map_err(|e| PatchError::JsonDeserialize(e.to_string()))?;

    let mut expected_revision_opt = None;
    for op in &patch.0 {
        if let PatchOperation::Test(test_op) = op {
            if test_op.path == "/revision" {
                expected_revision_opt = test_op.value.as_u64();
                break;
            }
        }
    }

    let Some(expected_revision) = expected_revision_opt else {
        return Err(PatchError::MissingRevisionTest);
    };

    let actual_revision = doc
        .get("revision")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    if expected_revision != actual_revision {
        use crate::diff::build_rich_diff;

        let human_nodes = doc
            .get("document")
            .and_then(|d| d.get("nodes"))
            .and_then(|n| n.as_object())
            .cloned();
        let human_edges = doc
            .get("document")
            .and_then(|d| d.get("edges"))
            .and_then(|e| e.as_object())
            .cloned();

        // Try to apply patch without the revision test to get AI proposed state
        let mut ai_doc = doc.clone();
        let mut patch_no_rev = patch.clone();
        patch_no_rev.0.retain(|op| {
            if let PatchOperation::Test(test_op) = op {
                test_op.path != "/revision"
            } else {
                true
            }
        });

        // If it applies, we extract the nodes and edges
        let (ai_nodes, ai_edges) = match json_patch::patch(&mut ai_doc, &patch_no_rev) {
            Ok(()) => (
                ai_doc
                    .get("document")
                    .and_then(|d| d.get("nodes"))
                    .and_then(|n| n.as_object())
                    .cloned(),
                ai_doc
                    .get("document")
                    .and_then(|d| d.get("edges"))
                    .and_then(|e| e.as_object())
                    .cloned(),
            ),
            Err(_) => (None, None),
        };

        let rich_diff = build_rich_diff(
            expected_revision,
            actual_revision,
            human_nodes.as_ref(),
            human_edges.as_ref(),
            ai_nodes.as_ref(),
            ai_edges.as_ref(),
        );

        let diff_val = serde_json::to_value(rich_diff).unwrap_or_else(|_| {
            serde_json::json!({
                "status": "rejected",
                "reason": "Human Priority Block"
            })
        });
        let doc_str = match serde_json::to_string(&diff_val) {
            Ok(s) => s,
            Err(e) => return Err(PatchError::SerializationFailure(e.to_string())),
        };

        match &cmd.output {
            PatchTarget::File(output_path) => {
                std::fs::write(output_path, doc_str)
                    .map_err(|e| PatchError::IoError(e.to_string()))?;
            }
            PatchTarget::Stdout => {
                stdout
                    .write_all(doc_str.as_bytes())
                    .map_err(|e| PatchError::IoError(e.to_string()))?;
            }
        }
        return Ok(());
    }

    json_patch::patch(&mut doc, &patch).map_err(|e| PatchError::ApplyError(e.to_string()))?;

    // Increment revision on success if revision is an integer
    if let Some(rev) = doc.get_mut("revision").and_then(|r| r.as_u64()) {
        doc["revision"] = serde_json::json!(rev + 1);
    }

    // Validate that the document is a valid DiagramDocument
    let doc_str =
        serde_json::to_string(&doc).map_err(|e| PatchError::SerializationFailure(e.to_string()))?;

    let _validated_doc: DiagramDocument =
        serde_json::from_str(&doc_str).map_err(|e| PatchError::InvalidDocument(e.to_string()))?;

    // Now write the result
    match &cmd.output {
        PatchTarget::File(output_path) => {
            std::fs::write(output_path, doc_str).map_err(|e| PatchError::IoError(e.to_string()))?;
        }
        PatchTarget::Stdout => {
            stdout
                .write_all(doc_str.as_bytes())
                .map_err(|e| PatchError::IoError(e.to_string()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::tempdir;

    const VALID_INPUT_JSON: &str = r#"{
        "version": 2,
        "revision": 0,
        "document": { "nodes": {}, "edges": {} },
        "editor_state": { "camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0 }
    }"#;

    const VALID_PATCH_JSON: &str = r#"[
        {"op": "test", "path": "/revision", "value": 0},
        {"op": "replace", "path": "/revision", "value": 1}
    ]"#;

    #[test]
    fn map_patch_subcommand_all_paths() {
        let input = PathBuf::from("/tmp/input.json");
        let patch = PathBuf::from("/tmp/patch.json");
        let output = PathBuf::from("/tmp/output.json");
        let cmd = map_patch_subcommand(Some(input.clone()), patch.clone(), Some(output.clone()));
        assert_eq!(cmd.input, PatchSource::File(input));
        assert_eq!(cmd.patch, patch);
        assert_eq!(cmd.output, PatchTarget::File(output));
    }

    #[test]
    fn map_patch_subcommand_no_input_output() {
        let patch = PathBuf::from("/tmp/patch.json");
        let cmd = map_patch_subcommand(None, patch, None);
        assert_eq!(cmd.input, PatchSource::Stdin);
        assert_eq!(cmd.output, PatchTarget::Stdout);
    }

    #[test]
    fn execute_patch_missing_input_file() {
        let dir = tempdir().expect("tempdir");
        let missing_input = dir.path().join("nonexistent.json");
        let patch_path = dir.path().join("patch.json");
        std::fs::write(&patch_path, VALID_PATCH_JSON).expect("write patch");
        let cmd = PatchCommand {
            input: PatchSource::File(missing_input.clone()),
            patch: patch_path,
            output: PatchTarget::Stdout,
        };
        let err = execute_patch(&cmd, Cursor::new(""), Vec::new()).expect_err("should fail");
        assert!(matches!(err, PatchError::FileNotFound(p) if p == missing_input));
    }

    #[test]
    fn execute_patch_missing_patch_file() {
        let dir = tempdir().expect("tempdir");
        let input_path = dir.path().join("input.json");
        std::fs::write(&input_path, VALID_INPUT_JSON).expect("write input");
        let missing_patch = dir.path().join("nonexistent_patch.json");
        let cmd = PatchCommand {
            input: PatchSource::File(input_path),
            patch: missing_patch.clone(),
            output: PatchTarget::Stdout,
        };
        let err = execute_patch(&cmd, Cursor::new(""), Vec::new()).expect_err("should fail");
        assert!(matches!(err, PatchError::FileNotFound(p) if p == missing_patch));
    }

    #[test]
    fn execute_patch_empty_input() {
        let dir = tempdir().expect("tempdir");
        let patch_path = dir.path().join("patch.json");
        std::fs::write(&patch_path, VALID_PATCH_JSON).expect("write patch");
        let cmd = PatchCommand {
            input: PatchSource::Stdin,
            patch: patch_path,
            output: PatchTarget::Stdout,
        };
        let err = execute_patch(&cmd, Cursor::new(""), Vec::new()).expect_err("should fail");
        assert!(matches!(err, PatchError::EmptyInput));
    }

    #[test]
    fn execute_patch_empty_patch() {
        let dir = tempdir().expect("tempdir");
        let patch_path = dir.path().join("patch.json");
        std::fs::write(&patch_path, "").expect("write empty patch");
        let cmd = PatchCommand {
            input: PatchSource::Stdin,
            patch: patch_path,
            output: PatchTarget::Stdout,
        };
        let err = execute_patch(&cmd, Cursor::new(VALID_INPUT_JSON), Vec::new())
            .expect_err("should fail");
        assert!(matches!(err, PatchError::EmptyInput));
    }

    #[test]
    fn execute_patch_missing_revision_test() {
        let dir = tempdir().expect("tempdir");
        let patch_path = dir.path().join("patch.json");
        // Patch with no /revision test
        std::fs::write(
            &patch_path,
            r#"[{"op": "replace", "path": "/version", "value": 3}]"#,
        )
        .expect("write patch");
        let cmd = PatchCommand {
            input: PatchSource::Stdin,
            patch: patch_path,
            output: PatchTarget::Stdout,
        };
        let err = execute_patch(&cmd, Cursor::new(VALID_INPUT_JSON), Vec::new())
            .expect_err("should fail");
        assert!(matches!(err, PatchError::MissingRevisionTest));
    }

    #[test]
    fn execute_patch_revision_mismatch_outputs_diff() {
        let dir = tempdir().expect("tempdir");
        let patch_path = dir.path().join("patch.json");
        // Patch expects revision 5, but doc has revision 0
        std::fs::write(
            &patch_path,
            r#"[
                {"op": "test", "path": "/revision", "value": 5},
                {"op": "replace", "path": "/revision", "value": 6}
            ]"#,
        )
        .expect("write patch");
        let cmd = PatchCommand {
            input: PatchSource::Stdin,
            patch: patch_path,
            output: PatchTarget::Stdout,
        };
        let mut stdout_buf = Vec::new();
        execute_patch(&cmd, Cursor::new(VALID_INPUT_JSON), &mut stdout_buf)
            .expect("should succeed");
        let output_str = String::from_utf8(stdout_buf).expect("utf8");
        let parsed: serde_json::Value = serde_json::from_str(&output_str).expect("valid json");
        assert_eq!(parsed["status"], "rejected");
        assert_eq!(parsed["conflict_context"]["expected_revision"], 5);
        assert_eq!(parsed["conflict_context"]["actual_revision"], 0);
    }

    #[test]
    fn execute_patch_matching_revision_succeeds_and_increments() {
        let dir = tempdir().expect("tempdir");
        let patch_path = dir.path().join("patch.json");
        let output_path = dir.path().join("output.json");
        std::fs::write(&patch_path, VALID_PATCH_JSON).expect("write patch");
        let cmd = PatchCommand {
            input: PatchSource::Stdin,
            patch: patch_path,
            output: PatchTarget::File(output_path.clone()),
        };
        execute_patch(&cmd, Cursor::new(VALID_INPUT_JSON), Vec::new()).expect("should succeed");
        let result_str = std::fs::read_to_string(&output_path).expect("read output");
        let result: serde_json::Value = serde_json::from_str(&result_str).expect("valid json");
        // Revision should be incremented from 0 → 1 (patch set 1, then +1 = 2)
        // Wait: patch sets /revision to 1, then code does rev + 1 = 2
        assert_eq!(result["revision"], 2);
    }
}
