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

#[cfg(feature = "async-db")]
use tokio::runtime::Runtime;

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
    // Database commands
    DbInit {
        #[arg(long)]
        path: String,
    },
    DbStatus {
        #[arg(long)]
        path: String,
    },
    DbRevision {
        #[arg(long)]
        path: String,
    },
    DbEvents {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = 0)]
        since: i64,
    },
    DbConflictDiff {
        #[arg(long)]
        path: String,
        #[arg(long)]
        assumed_revision: i64,
    },
    // Operation commands
    OpStart {
        #[arg(long)]
        path: String,
        #[arg(long)]
        operation_id: String,
        #[arg(long)]
        total_steps: u32,
        #[arg(long)]
        author_id: String,
        #[arg(long)]
        description: String,
    },
    OpStatus {
        #[arg(long)]
        path: String,
        #[arg(long)]
        operation_id: String,
    },
    OpList {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = String::from("in_progress"))]
        state: String,
    },
    OpComplete {
        #[arg(long)]
        path: String,
        #[arg(long)]
        operation_id: String,
    },
    OpFail {
        #[arg(long)]
        path: String,
        #[arg(long)]
        operation_id: String,
        #[arg(long)]
        error: String,
    },
    // Outbox commands
    OutboxList {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    OutboxAdd {
        #[arg(long)]
        path: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        side_effect_type: String,
        #[arg(long)]
        payload: String,
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
        Commands::DbInit { .. } => String::from("db_init"),
        Commands::DbStatus { .. } => String::from("db_status"),
        Commands::DbRevision { .. } => String::from("db_revision"),
        Commands::DbEvents { .. } => String::from("db_events"),
        Commands::DbConflictDiff { .. } => String::from("db_conflict_diff"),
        Commands::OpStart { .. } => String::from("op_start"),
        Commands::OpStatus { .. } => String::from("op_status"),
        Commands::OpList { .. } => String::from("op_list"),
        Commands::OpComplete { .. } => String::from("op_complete"),
        Commands::OpFail { .. } => String::from("op_fail"),
        Commands::OutboxList { .. } => String::from("outbox_list"),
        Commands::OutboxAdd { .. } => String::from("outbox_add"),
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
        Commands::Apply {
            input,
            subgraph,
            output,
        } => {
            emit_stage_event(
                "apply",
                &StageDetails::new()
                    .with_path(Path::new(input))
                    .with_code("started"),
            );

            let current_doc = load_doc(input)?;

            let subgraph_path = Path::new(subgraph);
            let subgraph_parent = subgraph_path.parent().filter(|p| !p.as_os_str().is_empty());
            let subgraph_base_dir = subgraph_parent.unwrap_or_else(|| Path::new("."));
            validate_safe_path(subgraph_path, subgraph_base_dir)
                .map_err(|e| anyhow!("Invalid subgraph path: {e}"))?;

            let subgraph_doc = load_doc(subgraph)?;

            let mut merged_doc = current_doc;

            for (node_id, node) in subgraph_doc.document.nodes {
                merged_doc.document.nodes.insert(node_id, node);
            }

            for (edge_id, edge) in subgraph_doc.document.edges {
                if merged_doc.document.nodes.contains_key(&edge.source)
                    && merged_doc.document.nodes.contains_key(&edge.target)
                {
                    merged_doc.document.edges.insert(edge_id, edge);
                }
            }

            let issues = crate::models::validation::validate_document(&merged_doc);
            if !issues.is_empty() {
                return Err(anyhow!(
                    "validation failed after apply: {}",
                    issues
                        .iter()
                        .map(|i| format!("{}: {}", i.code, i.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }

            merged_doc.revision = Revision::new(merged_doc.revision.value() + 1);

            save_workspace_atomic(&merged_doc, Path::new(output))
                .map_err(|e| anyhow!("Failed to save applied document: {e}"))?;

            emit_stage_event(
                "applied",
                &StageDetails::new()
                    .with_path(Path::new(output))
                    .with_code("success"),
            );
        }
        Commands::Export {
            input,
            format,
            output,
        } => {
            emit_stage_event(
                "export",
                &StageDetails::new()
                    .with_path(Path::new(input))
                    .with_code(format),
            );

            let doc = load_doc(input)?;

            let output_path = Path::new(output);
            let output_parent = output_path.parent().filter(|p| !p.as_os_str().is_empty());
            let output_base_dir = output_parent.unwrap_or_else(|| Path::new("."));
            validate_safe_path(output_path, output_base_dir)
                .map_err(|e| anyhow!("Invalid output path: {e}"))?;

            match format.to_lowercase().as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&doc)
                        .map_err(|e| anyhow!("Failed to serialize to JSON: {e}"))?;
                    let mut file = File::create(output).context("Failed to create output file")?;
                    file.write_all(json.as_bytes())
                        .context("Failed to write JSON content")?;
                }
                "dot" => {
                    let mut dot = String::from("digraph diagram {\n");
                    dot.push_str("  rankdir=LR;\n");
                    dot.push_str("  node [shape=box];\n");

                    for (node_id, node) in &doc.document.nodes {
                        let label = node.label.replace('"', "\\\"");
                        dot.push_str(&format!("  \"{node_id}\" [label=\"{label}\"];\n"));
                    }

                    for (_edge_id, edge) in &doc.document.edges {
                        dot.push_str(&format!("  \"{}\" -> \"{}\";\n", edge.source, edge.target));
                    }

                    dot.push_str("}\n");

                    let mut file = File::create(output).context("Failed to create DOT file")?;
                    file.write_all(dot.as_bytes())
                        .context("Failed to write DOT content")?;
                }
                "plantuml" => {
                    let mut plantuml = String::from("@startuml\n");

                    for (node_id, node) in &doc.document.nodes {
                        let label = node.label.replace('[', "(").replace(']', ")");
                        plantuml.push_str(&format!("card {node_id} as \"{label}\"\n"));
                    }

                    for (_edge_id, edge) in &doc.document.edges {
                        plantuml.push_str(&format!("{} --> {}\n", edge.source, edge.target));
                    }

                    plantuml.push_str("@enduml\n");

                    let mut file =
                        File::create(output).context("Failed to create PlantUML file")?;
                    file.write_all(plantuml.as_bytes())
                        .context("Failed to write PlantUML content")?;
                }
                _ => {
                    return Err(anyhow!(
                        "Unsupported format: {}. Supported formats: json, dot, plantuml",
                        format
                    ));
                }
            }

            emit_stage_event(
                "exported",
                &StageDetails::new()
                    .with_path(Path::new(output))
                    .with_code("success"),
            );
        }
        Commands::Import {
            input,
            format,
            output,
        } => {
            emit_stage_event(
                "import",
                &StageDetails::new()
                    .with_path(Path::new(input))
                    .with_code(format),
            );

            let input_path = Path::new(input);
            let input_parent = input_path.parent().filter(|p| !p.as_os_str().is_empty());
            let input_base_dir = input_parent.unwrap_or_else(|| Path::new("."));
            validate_safe_path(input_path, input_base_dir)
                .map_err(|e| anyhow!("Invalid input path: {e}"))?;

            let content = std::fs::read_to_string(input)
                .map_err(|e| anyhow!("Failed to read input file: {e}"))?;

            let doc = match format.to_lowercase().as_str() {
                "json" => {
                    let mut doc: DiagramDocument = serde_json::from_str(&content)
                        .map_err(|e| anyhow!("Failed to parse JSON: {e}"))?;
                    if doc.version != 2 {
                        return Err(anyhow!(
                            "Unsupported document version: {}. Only version 2 is supported.",
                            doc.version
                        ));
                    }
                    doc.revision = Revision::new(0);
                    doc
                }
                "dot" => {
                    let mut doc = DiagramDocument {
                        version: 2,
                        revision: Revision::new(0),
                        ..Default::default()
                    };

                    let node_re =
                        Regex::new(r#"^\s*"?(\w+)"?\s*\[.*label\s*=\s*"?([^"\]]+)"?\].*$"#)
                            .map_err(|e| anyhow!("Invalid node regex: {e}"))?;
                    let edge_re = Regex::new(r#"^\s*"?(\w+)"?\s*->\s*"?(\w+)"?\s*;?\s*$"#)
                        .map_err(|e| anyhow!("Invalid edge regex: {e}"))?;

                    for line in content.lines() {
                        if let Some(caps) = node_re.captures(line) {
                            let node_id = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                            let label = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                            let node = Node {
                                kind: NodeKind::Node,
                                icon: String::new(),
                                label: label.to_string(),
                                x: OrderedFloat(0.0),
                                y: OrderedFloat(0.0),
                                width: OrderedFloat(100.0),
                                height: OrderedFloat(50.0),
                                font_size: None,
                                font_weight: None,
                                locked: false,
                                parent: None,
                                dag_rank: None,
                                tags: im::Vector::new(),
                                metadata: HashMap::new(),
                                z_index: 0,
                                style: None,
                                collapsed: None,
                            };
                            doc.document
                                .nodes
                                .insert(NodeId::new(node_id.to_string()), node);
                        } else if let Some(caps) = edge_re.captures(line) {
                            let source = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                            let target = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                            if doc
                                .document
                                .nodes
                                .contains_key(&NodeId::new(source.to_string()))
                                && doc
                                    .document
                                    .nodes
                                    .contains_key(&NodeId::new(target.to_string()))
                            {
                                let edge = Edge {
                                    source: NodeId::new(source.to_string()),
                                    target: NodeId::new(target.to_string()),
                                    label: String::new(),
                                    style: EdgeStyle::Solid,
                                    arrow_type: ArrowType::Default,
                                    label_offset_t: OrderedFloat(0.5),
                                    color: None,
                                    thickness: OrderedFloat(2.0),
                                    directed: true,
                                    bend_points: im::Vector::new(),
                                    tags: im::Vector::new(),
                                    metadata: HashMap::new(),
                                    font_size: None,
                                };
                                doc.document
                                    .edges
                                    .insert(EdgeId::new(format!("{source}-{target}")), edge);
                            }
                        }
                    }

                    doc
                }
                "plantuml" => {
                    let mut doc = DiagramDocument {
                        version: 2,
                        revision: Revision::new(0),
                        ..Default::default()
                    };

                    let node_re = Regex::new(r#"(?:card|rectangle|node)\s+(\w+)\s+as\s+"([^"]+)""#)
                        .map_err(|e| anyhow!("Invalid plantuml node regex: {e}"))?;
                    let edge_re = Regex::new(r#"(\w+)\s*(--|->|<-|<--)\s*(\w+)"#)
                        .map_err(|e| anyhow!("Invalid plantuml edge regex: {e}"))?;

                    for line in content.lines() {
                        if let Some(caps) = node_re.captures(line) {
                            let node_id = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                            let label = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                            let node = Node {
                                kind: NodeKind::Node,
                                icon: String::new(),
                                label: label.to_string(),
                                x: OrderedFloat(0.0),
                                y: OrderedFloat(0.0),
                                width: OrderedFloat(100.0),
                                height: OrderedFloat(50.0),
                                font_size: None,
                                font_weight: None,
                                locked: false,
                                parent: None,
                                dag_rank: None,
                                tags: im::Vector::new(),
                                metadata: HashMap::new(),
                                z_index: 0,
                                style: None,
                                collapsed: None,
                            };
                            doc.document
                                .nodes
                                .insert(NodeId::new(node_id.to_string()), node);
                        } else if let Some(caps) = edge_re.captures(line) {
                            let source = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                            let target = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                            if doc
                                .document
                                .nodes
                                .contains_key(&NodeId::new(source.to_string()))
                                && doc
                                    .document
                                    .nodes
                                    .contains_key(&NodeId::new(target.to_string()))
                            {
                                let edge = Edge {
                                    source: NodeId::new(source.to_string()),
                                    target: NodeId::new(target.to_string()),
                                    label: String::new(),
                                    style: EdgeStyle::Solid,
                                    arrow_type: ArrowType::Default,
                                    label_offset_t: OrderedFloat(0.5),
                                    color: None,
                                    thickness: OrderedFloat(2.0),
                                    directed: true,
                                    bend_points: im::Vector::new(),
                                    tags: im::Vector::new(),
                                    metadata: HashMap::new(),
                                    font_size: None,
                                };
                                doc.document
                                    .edges
                                    .insert(EdgeId::new(format!("{source}-{target}")), edge);
                            }
                        }
                    }

                    doc
                }
                _ => {
                    return Err(anyhow!(
                        "Unsupported format: {}. Supported formats: json, dot, plantuml",
                        format
                    ));
                }
            };

            let issues = crate::models::validation::validate_document(&doc);
            if !issues.is_empty() {
                return Err(anyhow!(
                    "validation failed after import: {}",
                    issues
                        .iter()
                        .map(|i| format!("{}: {}", i.code, i.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }

            save_workspace_atomic(&doc, Path::new(output))
                .map_err(|e| anyhow!("Failed to save imported document: {e}"))?;

            emit_stage_event(
                "imported",
                &StageDetails::new()
                    .with_path(Path::new(output))
                    .with_code("success"),
            );
        }
        // Database commands
        Commands::DbInit { path } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            runtime.block_on(async {
                use crate::store_durable::bootstrap_durable_store;
                use crate::store_durable::DurableConfig;
                bootstrap_durable_store(db_path, DurableConfig::default())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))
            })?;
            println!("Database initialized at {}", path);
        }
        Commands::DbStatus { path } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            let status = runtime.block_on(async {
                use crate::store_async::bootstrap_async_store;
                use crate::store_async::read_store_pragmas_async;
                let bootstrap = bootstrap_async_store(db_path)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                let pragmas = read_store_pragmas_async(&bootstrap.pool)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                Ok::<_, anyhow::Error>(pragmas)
            })?;
            println!(
                "{}",
                serde_json::json!({
                    "path": path,
                    "schema_version": 2,
                    "journal_mode": status.journal_mode,
                    "synchronous": status.synchronous,
                    "wal_autocheckpoint": status.wal_autocheckpoint,
                    "foreign_keys": status.foreign_keys,
                    "busy_timeout": status.busy_timeout,
                })
            );
        }
        Commands::DbRevision { path } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            let revision = runtime.block_on(async {
                use crate::store_async::{bootstrap_async_store, fetch_latest_revision};
                let bootstrap = bootstrap_async_store(db_path)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                fetch_latest_revision(&bootstrap.pool)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))
            })?;
            println!("{}", revision);
        }
        Commands::DbEvents { path, since } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            let events = runtime.block_on(async {
                use crate::store_async::{bootstrap_async_store, fetch_events_since};
                let bootstrap = bootstrap_async_store(db_path)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                fetch_events_since(&bootstrap.pool, *since)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))
            })?;
            for event in events {
                println!(
                    "{}",
                    serde_json::json!({
                        "op_id": event.op_id,
                        "revision": event.revision,
                        "timestamp": event.timestamp,
                    })
                );
            }
        }
        Commands::DbConflictDiff {
            path,
            assumed_revision,
        } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            let diff = runtime.block_on(async {
                use crate::store_async::create_async_pool;
                use crate::store_durable::generate_conflict_diff;
                let pool = create_async_pool(db_path)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                generate_conflict_diff(&pool, *assumed_revision)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))
            })?;
            println!(
                "{}",
                serde_json::json!({
                    "assumed_revision": diff.assumed_revision,
                    "actual_revision": diff.actual_revision,
                    "changes_count": diff.changes.len(),
                    "first_change_timestamp": diff.first_change_timestamp,
                    "first_change_author": diff.first_change_author,
                })
            );
        }
        // Operation commands
        Commands::OpStart {
            path,
            operation_id,
            total_steps,
            author_id,
            description,
        } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            runtime.block_on(async {
                use crate::store_durable::{
                    bootstrap_durable_store, start_operation, DurableConfig,
                };
                let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| anyhow!(e.to_string()))?
                    .as_secs() as i64;
                start_operation(
                    &bootstrap.pool,
                    operation_id.clone(),
                    *total_steps,
                    author_id.clone(),
                    description.clone(),
                    timestamp,
                )
                .await
                .map_err(|e| anyhow!(e.to_string()))
            })?;
            println!("Operation {} started", operation_id);
        }
        Commands::OpStatus { path, operation_id } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            let op = runtime.block_on(async {
                use crate::store_durable::{bootstrap_durable_store, get_operation, DurableConfig};
                let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                get_operation(&bootstrap.pool, operation_id)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))
            })?;
            println!(
                "{}",
                serde_json::json!({
                    "operation_id": op.operation_id,
                    "state": op.state.as_str(),
                    "current_step": op.current_step,
                    "total_steps": op.total_steps,
                    "started_at": op.started_at,
                    "completed_at": op.completed_at,
                    "final_revision": op.final_revision,
                    "error_message": op.error_message,
                })
            );
        }
        Commands::OpList { path, state } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            let ops = runtime.block_on(async {
                use crate::store::types::OperationState;
                use crate::store_durable::{
                    bootstrap_durable_store, get_operations_by_state, DurableConfig,
                };
                let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                let op_state = OperationState::from_str(state)
                    .ok_or_else(|| anyhow!("Invalid state: {}", state))?;
                get_operations_by_state(&bootstrap.pool, op_state)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))
            })?;
            for op in ops {
                println!(
                    "{}",
                    serde_json::json!({
                        "operation_id": op.operation_id,
                        "state": op.state.as_str(),
                        "current_step": op.current_step,
                        "total_steps": op.total_steps,
                    })
                );
            }
        }
        Commands::OpComplete { path, operation_id } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            runtime.block_on(async {
                use crate::store::types::OperationState;
                use crate::store_async::fetch_latest_revision;
                use crate::store_durable::{
                    bootstrap_durable_store, update_operation_state, DurableConfig,
                };
                let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                let revision = fetch_latest_revision(&bootstrap.pool)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                update_operation_state(
                    &bootstrap.pool,
                    operation_id,
                    OperationState::Completed,
                    None,
                    Some(revision),
                    None,
                )
                .await
                .map_err(|e| anyhow!(e.to_string()))
            })?;
            println!("Operation {} completed", operation_id);
        }
        Commands::OpFail {
            path,
            operation_id,
            error,
        } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            runtime.block_on(async {
                use crate::store::types::OperationState;
                use crate::store_durable::{
                    bootstrap_durable_store, update_operation_state, DurableConfig,
                };
                let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                update_operation_state(
                    &bootstrap.pool,
                    operation_id,
                    OperationState::Failed,
                    None,
                    None,
                    Some(error.clone()),
                )
                .await
                .map_err(|e| anyhow!(e.to_string()))
            })?;
            println!("Operation {} failed: {}", operation_id, error);
        }
        // Outbox commands
        Commands::OutboxList { path, limit } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            let entries = runtime.block_on(async {
                use crate::store_durable::{
                    bootstrap_durable_store, get_pending_outbox, DurableConfig,
                };
                let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                get_pending_outbox(&bootstrap.pool, *limit)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))
            })?;
            for entry in entries {
                println!(
                    "{}",
                    serde_json::json!({
                        "id": entry.id,
                        "side_effect_type": entry.side_effect_type.as_str(),
                        "status": entry.status.as_str(),
                        "retry_count": entry.retry_count,
                        "max_retries": entry.max_retries,
                        "created_at": entry.created_at,
                    })
                );
            }
        }
        Commands::OutboxAdd {
            path,
            id,
            side_effect_type,
            payload,
        } => {
            let db_path = Path::new(path);
            let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
            runtime.block_on(async {
                use crate::store::types::SideEffectType;
                use crate::store_async::fetch_latest_revision;
                use crate::store_durable::{
                    add_outbox_entry, bootstrap_durable_store, DurableConfig,
                };
                let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                let revision = fetch_latest_revision(&bootstrap.pool)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                let se_type = SideEffectType::from_str(side_effect_type)
                    .ok_or_else(|| anyhow!("Invalid side effect type: {}", side_effect_type))?;
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| anyhow!(e.to_string()))?
                    .as_secs() as i64;
                add_outbox_entry(
                    &bootstrap.pool,
                    id.clone(),
                    se_type,
                    payload.clone(),
                    revision,
                    3,
                    timestamp,
                )
                .await
                .map_err(|e| anyhow!(e.to_string()))
            })?;
            println!("Outbox entry {} added", id);
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
