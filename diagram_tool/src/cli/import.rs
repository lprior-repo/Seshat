use anyhow::{anyhow, Result};
use im::HashMap;
use regex_lite::Regex;
use std::path::Path;

use crate::cli_persistence::{
    emit_stage_event, save_workspace_atomic, validate_safe_path, StageDetails,
};
use diagram_models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind,
    OrderedFloat, Revision,
};

pub fn handle(input: &str, format: &str, output: &str) -> Result<()> {
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

    let content =
        std::fs::read_to_string(input).map_err(|e| anyhow!("Failed to read input file: {e}"))?;

    let doc = match format.to_lowercase().as_str() {
        "json" => {
            let mut doc: DiagramDocument =
                serde_json::from_str(&content).map_err(|e| anyhow!("Failed to parse JSON: {e}"))?;
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

            let node_re = Regex::new(r#"^\s*"?(\w+)"?\s*\[.*label\s*=\s*"?([^"\]]+)"?\].*$"#)
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
                        lock_state: LockState::Unlocked,
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
                            source_port: None,
                            target_port: None,
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
                        lock_state: LockState::Unlocked,
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
                            source_port: None,
                            target_port: None,
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

    let issues = diagram_models::validation::validate_document(&doc);
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
    Ok(())
}
