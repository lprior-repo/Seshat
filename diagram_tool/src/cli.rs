#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::{fs::File, io::Write, path::Path};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use im::HashMap;
use regex_lite::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    cli_persistence::{
        emit_stage_event, load_workspace_with_lkg, save_workspace_atomic, validate_safe_path,
        StageDetails,
    },
    export::{png::export_png, svg::generate_svg_string},
    models::document::{
        ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, OrderedFloat,
        Revision,
    },
    mutation::{ops::apply_layout, pipeline::run_mutation},
};

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
    GenerateScene {
        #[arg(long, default_value_t = 3000)]
        nodes: u32,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long)]
        output: String,
    },
    Apply {
        #[arg(long)]
        input: String,
        #[arg(long)]
        subgraph: String,
        #[arg(long)]
        output: String,
    },
    Export {
        #[arg(long)]
        input: String,
        #[arg(long)]
        format: String,
        #[arg(long)]
        output: String,
    },
    Import {
        #[arg(long)]
        input: String,
        #[arg(long)]
        format: String,
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
        Commands::GenerateScene { .. } => String::from("generate_scene"),
        Commands::Apply { .. } => String::from("apply"),
        Commands::Export { .. } => String::from("export"),
        Commands::Import { .. } => String::from("import"),
    }
}

pub fn error_code(err: &anyhow::Error) -> String {
    let msg = err.to_string().to_lowercase();
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

            let output_path = Path::new(output);
            let output_parent = output_path.parent().filter(|p| !p.as_os_str().is_empty());
            let output_base_dir = output_parent.unwrap_or_else(|| Path::new("."));
            validate_safe_path(output_path, output_base_dir)
                .map_err(|e| anyhow!("Invalid output path: {e}"))?;

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

            let patch_content = std::fs::read_to_string(patch)
                .map_err(|e| anyhow!("Failed to read patch file: {e}"))?;
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
        }
        Commands::GenerateScene {
            nodes,
            seed,
            output,
        } => {
            emit_stage_event(
                "generating_scene",
                &StageDetails::new()
                    .with_path(Path::new(output))
                    .with_code("started"),
            );

            let doc = crate::perf::generate_test_scene(*nodes, *seed);

            save_workspace_atomic(&doc, Path::new(output))
                .map_err(|e| anyhow!("Failed to save generated scene: {e}"))?;

            emit_stage_event(
                "generated_scene",
                &StageDetails::new()
                    .with_path(Path::new(output))
                    .with_code("success"),
            );
        }
        Commands::Apply { .. } => {
            return Err(anyhow!("apply command not yet implemented"));
        }
        Commands::Export { .. } => {
            return Err(anyhow!("export command not yet implemented"));
        }
        Commands::Import { .. } => {
            return Err(anyhow!("import command not yet implemented"));
        }
    }
    Ok(())
}

fn load_doc(path: &str) -> Result<DiagramDocument> {
    load_workspace_with_lkg(Path::new(path))
        .map_err(|e| anyhow!("Failed to load document from {path}: {e}"))
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
