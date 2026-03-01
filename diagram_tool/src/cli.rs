#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::cli_persistence::{
    emit_stage_event, load_workspace_with_lkg, save_workspace_atomic, StageDetails,
};
use crate::export::png::export_png;
use crate::export::svg::generate_svg_string;
use crate::models::document::{DiagramDocument, NodeId, Revision};
use crate::mutation::ops::apply_layout;
use crate::mutation::pipeline::run_mutation;
use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Parser, Debug, Clone)]
#[command(name = "diagram_tool")]
#[command(version = "0.1.0")]
#[command(about = "Diagram Tool CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Render {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    Layout {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    Validate {
        #[arg(long)]
        input: String,
    },
    Patch {
        #[arg(long)]
        input: String,
        #[arg(long)]
        patch: String,
        #[arg(long)]
        output: String,
    },
}

pub fn run_cli(cli: &Cli) {
    if let Some(cmd) = &cli.command {
        emit_event(&CliEvent::start(command_name(cmd)));
        match execute_command(cmd) {
            Ok(()) => {
                emit_event(&CliEvent::finish(
                    command_name(cmd),
                    true,
                    String::from("ok"),
                ));
            }
            Err(err) => {
                emit_event(&CliEvent::error(
                    command_name(cmd),
                    error_code(&err),
                    err.to_string(),
                ));
                emit_event(&CliEvent::finish(
                    command_name(cmd),
                    false,
                    error_code(&err),
                ));
                std::process::exit(exit_code(&err));
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CliEvent {
    event: String,
    command: String,
    ok: bool,
    code: String,
    message: Option<String>,
}

impl CliEvent {
    pub fn start(command: String) -> Self {
        Self {
            event: String::from("start"),
            command,
            ok: true,
            code: String::from("start"),
            message: None,
        }
    }

    pub fn error(command: String, code: String, message: String) -> Self {
        Self {
            event: String::from("error"),
            command,
            ok: false,
            code,
            message: Some(message),
        }
    }

    pub fn finish(command: String, ok: bool, code: String) -> Self {
        Self {
            event: String::from("finish"),
            command,
            ok,
            code,
            message: None,
        }
    }
}

fn command_name(cmd: &Commands) -> String {
    match cmd {
        Commands::Render { .. } => String::from("render"),
        Commands::Layout { .. } => String::from("layout"),
        Commands::Validate { .. } => String::from("validate"),
        Commands::Patch { .. } => String::from("patch"),
    }
}

pub fn error_code(err: &anyhow::Error) -> String {
    let msg = err.to_string().to_lowercase();
    // Check more specific patterns before general ones
    if msg.contains("dag") || msg.contains("cycle") {
        String::from("dag_violation")
    } else if msg.contains("dangling") || msg.contains("edge-dangling") {
        String::from("dangling_reference")
    } else if msg.contains("stale_revision") {
        String::from("stale_revision")
    } else if msg.contains("schema") {
        String::from("schema_violation")
    } else if msg.contains("semantic") || msg.contains("semantic validation error") {
        String::from("semantic_error")
    } else if msg.contains("parse")
        || msg.contains("deserialize")
        || msg.contains("unknown variant")
        || msg.contains("failed to parse")
    {
        String::from("parse_error")
    } else {
        String::from("command_error")
    }
}

pub fn exit_code(err: &anyhow::Error) -> i32 {
    let code = error_code(err);
    match code.as_str() {
        "parse_error" | "command_error" => 2,
        _ => 1,
    }
}

fn emit_event(event: &CliEvent) {
    match serde_json::to_string(&event) {
        Ok(line) => println!("{line}"),
        Err(_) => {
            println!("{{\"event\":\"error\",\"ok\":false,\"code\":\"jsonl_encode_error\"}}");
        }
    }
}

fn execute_command(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Render { input, output } => {
            let doc = load_doc(input)?;
            if Path::new(output)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            {
                export_png(&doc, output)?;
            } else if Path::new(output)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
            {
                let svg = generate_svg_string(&doc);
                let mut file = File::create(output).context("Failed to create SVG file")?;
                file.write_all(svg.as_bytes())
                    .context("Failed to write SVG content")?;
            } else {
                return Err(anyhow!(
                    "unknown output format; expected .png or .svg extension"
                ));
            }
        }
        Commands::Layout { input, output } => {
            emit_stage_event(
                "validating",
                &StageDetails::new().with_path(Path::new(input)),
            );
            let doc = load_doc(input)?;
            let laid_out_doc = run_mutation(&doc, |current| Ok(apply_layout(current, 200.0)))
                .map_err(|err| anyhow!(err.to_string()))?;
            save_workspace_atomic(&laid_out_doc, Path::new(output))
                .map_err(|e| anyhow!("Failed to save workspace: {e}"))?;
        }
        Commands::Validate { input } => {
            emit_stage_event(
                "validating",
                &StageDetails::new().with_path(Path::new(input)),
            );
            let doc = load_doc(input)?;
            // Run full validation pipeline
            let issues = crate::models::validation::validate_document(&doc);
            if !issues.is_empty() {
                return Err(anyhow!(
                    "validation failed: {}",
                    issues
                        .iter()
                        .map(|i| format!("{}: {}", i.code, i.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
        }
        Commands::Patch {
            input,
            patch,
            output,
        } => {
            emit_stage_event(
                "patching",
                &StageDetails::new()
                    .with_path(Path::new(input))
                    .with_code("started"),
            );

            // Load the document
            let current_doc = load_doc(input)?;

            // Read and parse the patch file
            let patch_content = std::fs::read_to_string(patch)
                .map_err(|e| anyhow!("Failed to read patch file: {e}"))?;
            let patch_ops: Vec<serde_json::Value> = serde_json::from_str(&patch_content)
                .map_err(|e| anyhow!("Failed to parse patch JSON: {e}"))?;

            // Check that first operation is a test for /revision (optimistic locking)
            let has_revision_test = patch_ops.first().is_some_and(|op| {
                op.get("op").and_then(|v| v.as_str()) == Some("test")
                    && op.get("path").and_then(|v| v.as_str()) == Some("/revision")
            });
            if !has_revision_test {
                return Err(anyhow!(
                    "patch must start with test operation for /revision"
                ));
            }

            // Apply patch operations
            let mut doc = current_doc.clone();
            for op in &patch_ops {
                let op_type = op.get("op").and_then(|v| v.as_str()).unwrap_or("");
                let path = op.get("path").and_then(|v| v.as_str()).unwrap_or("/");

                match op_type {
                    "test" => {
                        // Test operation - verify value matches before proceeding
                        let expected = op.get("value");
                        let actual = json_pointer_get(&doc, path);
                        let test_passed = expected
                            .and_then(|e| actual.as_ref().map(|a| e == a))
                            .unwrap_or(false);
                        if !test_passed {
                            // Determine error code based on path
                            let err_code = if path == "/revision" {
                                "stale_revision"
                            } else {
                                "command_error"
                            };

                            // Create LKG before failing - save to .lkg subdirectory
                            let input_path = Path::new(input);
                            let lkg_dir =
                                input_path.parent().unwrap_or(Path::new(".")).join(".lkg");
                            std::fs::create_dir_all(&lkg_dir).ok();
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
                                    "lkg_saved",
                                    &StageDetails::new()
                                        .with_path(&lkg_path)
                                        .with_code("lkg_save_failed")
                                        .with_message(&e.to_string()),
                                );
                            }

                            emit_event(&CliEvent::error(
                                String::from("patch"),
                                String::from(err_code),
                                format!(
                                    "test failed at {path}: expected {expected:?} but got {actual:?}"
                                ),
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

            // Run validation pipeline
            let validated_doc = run_mutation(&doc, |d| Ok(d.clone()))
                .map_err(|err| anyhow!("Patch validation failed: {err}"))?;

            // Save the result
            save_workspace_atomic(&validated_doc, Path::new(output))
                .map_err(|e| anyhow!("Failed to save patched document: {e}"))?;

            emit_stage_event(
                "patched",
                &StageDetails::new()
                    .with_path(Path::new(output))
                    .with_code("success"),
            );
        }
    }
    Ok(())
}

fn load_doc(path: &str) -> Result<DiagramDocument> {
    load_workspace_with_lkg(Path::new(path))
        .map_err(|e| anyhow!("Failed to load document from {path}: {e}"))
}

/// Get a value from the document using a simple JSON Pointer path
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

/// Set a value in the document using a simple JSON Pointer path
fn json_pointer_set(doc: &mut DiagramDocument, path: &str, value: serde_json::Value) -> Result<()> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["revision"] => {
            if let Some(v) = value.as_u64() {
                doc.revision = Revision::new(v);
                Ok(())
            } else {
                Err(anyhow!("revision must be a number"))
            }
        }
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

/// Remove a value from the document using a simple JSON Pointer path
fn json_pointer_remove(_doc: &mut DiagramDocument, _path: &str) -> Result<()> {
    Err(anyhow!("remove operation not implemented"))
}
