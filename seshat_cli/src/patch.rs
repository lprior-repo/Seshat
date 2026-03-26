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

        let diff_val = match serde_json::to_value(rich_diff) {
            Ok(v) => v,
            Err(_) => serde_json::json!({
                "status": "rejected",
                "reason": "Human Priority Block"
            }),
        };
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
