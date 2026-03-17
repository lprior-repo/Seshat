use anyhow::{anyhow, Result};
use std::path::Path;

use crate::cli::common::{emit_event, load_doc, CliEvent};
use crate::cli_persistence::{
    emit_stage_event, save_workspace_atomic, validate_safe_path, StageDetails,
};
use crate::mutation::pipeline::run_mutation;
use diagram_models::document::{DiagramDocument, NodeId};

pub fn handle(input: &str, patch: &str, output: &str) -> Result<()> {
    emit_stage_event(
        "patching",
        &StageDetails::new()
            .with_path(Path::new(input))
            .with_code("started"),
    );

    let current_doc = load_doc(input)?;

    let input_path = Path::new(input);
    let lkg_dir = input_path.parent().unwrap_or(Path::new(".")).join(".lkg");
    if let Err(e) = std::fs::create_dir_all(&lkg_dir) {
        emit_stage_event(
            "lkg_dir_create_failed",
            &StageDetails::new()
                .with_path(&lkg_dir)
                .with_code("lkg_dir_create_failed")
                .with_message(&e.to_string()),
        );
    }
    let lkg_filename = format!(
        "{}.lkg",
        input_path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default()
    );
    let lkg_path = lkg_dir.join(lkg_filename);

    if let Err(e) = save_workspace_atomic(&current_doc, &lkg_path) {
        emit_stage_event(
            "lkg_save_failed",
            &StageDetails::new()
                .with_path(&lkg_path)
                .with_code("lkg_save_failed")
                .with_message(&e.to_string()),
        );
    } else {
        emit_stage_event(
            "lkg_saved",
            &StageDetails::new()
                .with_path(&lkg_path)
                .with_code("success"),
        );
    }

    let patch_path = Path::new(patch);
    let patch_parent = patch_path.parent().filter(|p| !p.as_os_str().is_empty());
    let patch_base_dir = patch_parent.unwrap_or_else(|| Path::new("."));
    validate_safe_path(patch_path, patch_base_dir)
        .map_err(|e| anyhow!("Invalid patch path: {e}"))?;

    let patch_content =
        std::fs::read_to_string(patch).map_err(|e| anyhow!("Failed to read patch file: {e}"))?;
    let patch_ops: Vec<serde_json::Value> = serde_json::from_str(&patch_content)
        .map_err(|e| anyhow!("Failed to parse patch JSON: {e}"))?;

    let has_revision_test = patch_ops.first().is_some_and(|op| {
        op.get("op").and_then(|v| v.as_str()) == Some("test")
            && op.get("path").and_then(|v| v.as_str()) == Some("/revision")
    });
    if !has_revision_test {
        return Err(anyhow!(
            "patch must start with test operation for /revision"
        ));
    }

    let mut doc = current_doc.clone();
    for op in &patch_ops {
        let op_type = op.get("op").and_then(|v| v.as_str()).unwrap_or("");
        let path = op.get("path").and_then(|v| v.as_str()).unwrap_or("/");

        match op_type {
            "test" => {
                let expected = op.get("value");
                let actual = json_pointer_get(&doc, path);
                let test_passed = expected
                    .and_then(|e| actual.as_ref().map(|a| e == a))
                    .unwrap_or(false);
                if !test_passed {
                    let err_code = if path == "/revision" {
                        "stale_revision"
                    } else {
                        "command_error"
                    };

                    emit_event(&CliEvent::error(
                        String::from("patch"),
                        String::from(err_code),
                        format!("test failed at {path}: expected {expected:?} but got {actual:?}"),
                    ));

                    return Err(anyhow!(
                        "{err_code}: test failed at {path}: expected {expected:?} but got {actual:?}"
                    ));
                }
            }
            "replace" => {
                let value = op
                    .get("value")
                    .ok_or_else(|| anyhow!("replace operation missing value"))?;
                json_pointer_set(&mut doc, path, value.clone())?;
            }
            "add" => {
                let value = op
                    .get("value")
                    .ok_or_else(|| anyhow!("add operation missing value"))?;
                json_pointer_set(&mut doc, path, value.clone())?;
            }
            "remove" => {
                json_pointer_remove(&mut doc, path)?;
            }
            _ => {
                return Err(anyhow!("unsupported patch operation: {op_type}"));
            }
        }
    }

    let validated_doc = run_mutation(&doc, |d| Ok(d.clone()))
        .map_err(|err| anyhow!("Patch validation failed: {err}"))?;

    save_workspace_atomic(&validated_doc, Path::new(output))
        .map_err(|e| anyhow!("Failed to save patched document: {e}"))?;

    emit_stage_event(
        "patched",
        &StageDetails::new()
            .with_path(Path::new(output))
            .with_code("success"),
    );
    Ok(())
}

fn json_pointer_get(doc: &DiagramDocument, path: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["revision"] => Some(serde_json::json!(doc.revision.value())),
        ["document", "nodes", node_id, "label"] => doc
            .document
            .nodes
            .get(&NodeId::new(node_id.to_string()))
            .map(|n| serde_json::json!(n.label)),
        _ => None,
    }
}

fn json_pointer_set(doc: &mut DiagramDocument, path: &str, value: serde_json::Value) -> Result<()> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["revision"] => Err(anyhow!(
            "cannot write to /revision via patch: revision is computed from input document"
        )),
        ["document", "nodes", node_id, "label"] => {
            let node_id = NodeId::new(node_id.to_string());
            if let Some(node) = doc.document.nodes.get_mut(&node_id) {
                if let Some(label) = value.as_str() {
                    node.label = label.to_string();
                    Ok(())
                } else {
                    Err(anyhow!("label must be a string"))
                }
            } else {
                Err(anyhow!("node {node_id} not found"))
            }
        }
        _ => Err(anyhow!("unsupported path: {path}")),
    }
}

fn json_pointer_remove(_doc: &mut DiagramDocument, _path: &str) -> Result<()> {
    Err(anyhow!("remove operation not implemented"))
}
