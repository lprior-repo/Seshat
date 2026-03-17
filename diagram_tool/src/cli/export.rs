use anyhow::{anyhow, Context, Result};
use std::{fs::File, io::Write, path::Path};

use crate::cli::common::load_doc;
use crate::cli_persistence::{emit_stage_event, validate_safe_path, StageDetails};

pub fn handle(input: &str, format: &str, output: &str) -> Result<()> {
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

            let mut file = File::create(output).context("Failed to create PlantUML file")?;
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
    Ok(())
}
