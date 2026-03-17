use anyhow::{anyhow, Result};
use std::path::Path;

use crate::cli::common::load_doc;
use crate::cli_persistence::{
    emit_stage_event, save_workspace_atomic, validate_safe_path, StageDetails,
};
use diagram_models::document::Revision;

pub fn handle(input: &str, subgraph: &str, output: &str) -> Result<()> {
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

    let issues = diagram_models::validation::validate_document(&merged_doc);
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
    Ok(())
}
