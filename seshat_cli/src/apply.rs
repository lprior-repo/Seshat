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
