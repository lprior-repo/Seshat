use crate::domain::{ApplyCommand, ApplySource};
use crate::error::ApplyError;
use diagram_models::document::DiagramDocument;
use std::io::{Read, Write};
use std::path::PathBuf;

#[must_use]
pub fn map_apply_subcommand(input: Option<PathBuf>, proposal: PathBuf) -> ApplyCommand {
    let document_source = input.map_or(ApplySource::Stdin, ApplySource::File);
    ApplyCommand {
        document: document_source,
        proposal,
    }
}

/// Executes the apply subcommand.
///
/// # Errors
/// Returns `ApplyError` on failure.
pub fn execute_apply(
    cmd: &ApplyCommand,
    mut stdin: impl Read,
    mut stdout: impl Write,
) -> Result<(), ApplyError> {
    let doc_str = match &cmd.document {
        ApplySource::File(path) => std::fs::read_to_string(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ApplyError::FileNotFound(path.clone()),
            std::io::ErrorKind::InvalidData => ApplyError::InvalidUtf8,
            _ => ApplyError::IoError(e.to_string()),
        })?,
        ApplySource::Stdin => {
            let mut buf = String::new();
            stdin.read_to_string(&mut buf).map_err(|e| match e.kind() {
                std::io::ErrorKind::InvalidData => ApplyError::InvalidUtf8,
                _ => ApplyError::IoError(e.to_string()),
            })?;
            buf
        }
    };

    if doc_str.trim().is_empty() {
        return Err(ApplyError::EmptyInput);
    }

    let document: DiagramDocument =
        serde_json::from_str(&doc_str).map_err(|e| ApplyError::InvalidDocument(e.to_string()))?;

    let proposal_str = std::fs::read_to_string(&cmd.proposal).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ApplyError::FileNotFound(cmd.proposal.clone()),
        std::io::ErrorKind::InvalidData => ApplyError::InvalidUtf8,
        _ => ApplyError::IoError(e.to_string()),
    })?;

    if proposal_str.trim().is_empty() {
        return Err(ApplyError::EmptyInput);
    }

    let proposal: serde_json::Value = serde_json::from_str(&proposal_str)
        .map_err(|e| ApplyError::JsonDeserialize(e.to_string()))?;

    let base_revision = proposal
        .get("base_revision")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            ApplyError::InvalidProposal("Missing or invalid base_revision".to_string())
        })?;

    let current_revision = document.revision.value();

    let result = if base_revision == current_revision {
        serde_json::json!({
            "status": "queued",
            "base_revision": base_revision
        })
    } else {
        use crate::diff::build_rich_diff;

        let human_nodes = document.document.nodes.clone();
        let human_edges = document.document.edges.clone();

        let ai_nodes = proposal
            .get("document")
            .and_then(|d| d.get("nodes"))
            .and_then(|n| n.as_object());
        let ai_edges = proposal
            .get("document")
            .and_then(|d| d.get("edges"))
            .and_then(|e| e.as_object());

        let human_nodes_json = match serde_json::to_value(&human_nodes) {
            Ok(v) => v,
            Err(_) => serde_json::json!({}),
        };
        let human_nodes_obj = human_nodes_json.as_object();

        let human_edges_json = match serde_json::to_value(&human_edges) {
            Ok(v) => v,
            Err(_) => serde_json::json!({}),
        };
        let human_edges_obj = human_edges_json.as_object();

        let rich_diff = build_rich_diff(
            base_revision,
            current_revision,
            human_nodes_obj,
            human_edges_obj,
            ai_nodes,
            ai_edges,
        );

        match serde_json::to_value(rich_diff) {
            Ok(v) => v,
            Err(_) => serde_json::json!({
                "status": "rejected",
                "reason": "Human Priority Block"
            }),
        }
    };

    let result_json = serde_json::to_string_pretty(&result)
        .map_err(|e| ApplyError::SerializationFailure(e.to_string()))?;

    stdout
        .write_all(result_json.as_bytes())
        .map_err(|e| ApplyError::IoError(e.to_string()))?;
    stdout
        .write_all(b"\n")
        .map_err(|e| ApplyError::IoError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::tempdir;

    const VALID_DOC_JSON: &str = r#"{
        "version": 2,
        "revision": 0,
        "document": { "nodes": {}, "edges": {} },
        "editor_state": { "camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0 }
    }"#;

    #[test]
    fn map_apply_subcommand_with_input_returns_file_source() {
        let input = PathBuf::from("/tmp/input.json");
        let proposal = PathBuf::from("/tmp/proposal.json");
        let cmd = map_apply_subcommand(Some(input.clone()), proposal.clone());
        assert_eq!(cmd.document, ApplySource::File(input));
        assert_eq!(cmd.proposal, proposal);
    }

    #[test]
    fn map_apply_subcommand_no_input_returns_stdin_source() {
        let proposal = PathBuf::from("/tmp/proposal.json");
        let cmd = map_apply_subcommand(None, proposal.clone());
        assert_eq!(cmd.document, ApplySource::Stdin);
        assert_eq!(cmd.proposal, proposal);
    }

    #[test]
    fn execute_apply_missing_input_file() {
        let dir = tempdir().expect("tempdir");
        let missing_input = dir.path().join("nonexistent.json");
        let proposal_path = dir.path().join("proposal.json");
        std::fs::write(
            &proposal_path,
            r#"{ "base_revision": 0, "document": { "nodes": {}, "edges": {} } }"#,
        )
        .expect("write proposal");
        let cmd = ApplyCommand {
            document: ApplySource::File(missing_input.clone()),
            proposal: proposal_path,
        };
        let err = execute_apply(&cmd, Cursor::new(""), Vec::new()).expect_err("should fail");
        assert!(matches!(err, ApplyError::FileNotFound(p) if p == missing_input));
    }

    #[test]
    fn execute_apply_missing_proposal_file() {
        let dir = tempdir().expect("tempdir");
        let input_path = dir.path().join("input.json");
        std::fs::write(&input_path, VALID_DOC_JSON).expect("write input");
        let missing_proposal = dir.path().join("nonexistent_proposal.json");
        let cmd = ApplyCommand {
            document: ApplySource::File(input_path),
            proposal: missing_proposal.clone(),
        };
        let err = execute_apply(&cmd, Cursor::new(""), Vec::new()).expect_err("should fail");
        assert!(matches!(err, ApplyError::FileNotFound(p) if p == missing_proposal));
    }

    #[test]
    fn execute_apply_empty_input() {
        let dir = tempdir().expect("tempdir");
        let proposal_path = dir.path().join("proposal.json");
        std::fs::write(
            &proposal_path,
            r#"{ "base_revision": 0, "document": { "nodes": {}, "edges": {} } }"#,
        )
        .expect("write proposal");
        let cmd = ApplyCommand {
            document: ApplySource::Stdin,
            proposal: proposal_path,
        };
        let err = execute_apply(&cmd, Cursor::new(""), Vec::new()).expect_err("should fail");
        assert!(matches!(err, ApplyError::EmptyInput));
    }

    #[test]
    fn execute_apply_empty_proposal() {
        let dir = tempdir().expect("tempdir");
        let proposal_path = dir.path().join("proposal.json");
        std::fs::write(&proposal_path, "").expect("write empty proposal");
        let cmd = ApplyCommand {
            document: ApplySource::Stdin,
            proposal: proposal_path,
        };
        let err =
            execute_apply(&cmd, Cursor::new(VALID_DOC_JSON), Vec::new()).expect_err("should fail");
        assert!(matches!(err, ApplyError::EmptyInput));
    }

    #[test]
    fn execute_apply_proposal_missing_base_revision() {
        let dir = tempdir().expect("tempdir");
        let proposal_path = dir.path().join("proposal.json");
        std::fs::write(
            &proposal_path,
            r#"{ "document": { "nodes": {}, "edges": {} } }"#,
        )
        .expect("write proposal");
        let cmd = ApplyCommand {
            document: ApplySource::Stdin,
            proposal: proposal_path,
        };
        let err =
            execute_apply(&cmd, Cursor::new(VALID_DOC_JSON), Vec::new()).expect_err("should fail");
        assert!(matches!(err, ApplyError::InvalidProposal(_)));
    }

    #[test]
    fn execute_apply_matching_revision_outputs_queued() {
        let dir = tempdir().expect("tempdir");
        let proposal_path = dir.path().join("proposal.json");
        std::fs::write(
            &proposal_path,
            r#"{ "base_revision": 0, "document": { "nodes": {}, "edges": {} } }"#,
        )
        .expect("write proposal");
        let cmd = ApplyCommand {
            document: ApplySource::Stdin,
            proposal: proposal_path,
        };
        let mut stdout_buf = Vec::new();
        execute_apply(&cmd, Cursor::new(VALID_DOC_JSON), &mut stdout_buf).expect("should succeed");
        let output_str = String::from_utf8(stdout_buf).expect("utf8");
        let parsed: serde_json::Value =
            serde_json::from_str(output_str.trim()).expect("valid json");
        assert_eq!(parsed["status"], "queued");
        assert_eq!(parsed["base_revision"], 0);
    }

    #[test]
    fn execute_apply_mismatched_revision_outputs_rejected_with_diff() {
        let dir = tempdir().expect("tempdir");
        let proposal_path = dir.path().join("proposal.json");
        // Proposal expects revision 5 but doc is at 0
        std::fs::write(
            &proposal_path,
            r#"{ "base_revision": 5, "document": { "nodes": {}, "edges": {} } }"#,
        )
        .expect("write proposal");
        let cmd = ApplyCommand {
            document: ApplySource::Stdin,
            proposal: proposal_path,
        };
        let mut stdout_buf = Vec::new();
        execute_apply(&cmd, Cursor::new(VALID_DOC_JSON), &mut stdout_buf).expect("should succeed");
        let output_str = String::from_utf8(stdout_buf).expect("utf8");
        let parsed: serde_json::Value =
            serde_json::from_str(output_str.trim()).expect("valid json");
        assert_eq!(parsed["status"], "rejected");
        assert_eq!(parsed["conflict_context"]["expected_revision"], 5);
        assert_eq!(parsed["conflict_context"]["actual_revision"], 0);
    }
}
