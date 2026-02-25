#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::dag::validate_dag;
use crate::models::document::{DiagramDocument, NodeId, NodeKind};
use anyhow::{anyhow, bail, Result};
use im::HashSet;

/// Functional schema validation.
pub fn validate_schema(doc: &DiagramDocument) -> Result<()> {
    if doc.version != 2 {
        bail!("Document version must be 2, got {}", doc.version);
    }

    let nodes = &doc.document.nodes;
    let node_ids = nodes.keys().cloned().collect::<HashSet<NodeId>>();

    // 1. Validate Nodes
    nodes.iter().try_for_each(|(id, node)| {
        if let Some(parent_id) = &node.parent {
            if !node_ids.contains(parent_id) {
                bail!("Node {id} references non-existent parent {parent_id}");
            }
            if !nodes
                .get(parent_id)
                .is_some_and(|p| p.kind == NodeKind::Subgraph)
            {
                bail!("Node {id} parent {parent_id} is not a subgraph");
            }
        }
        Ok(())
    })?;

    // 2. Validate Edges
    doc.document.edges.iter().try_for_each(|(id, edge)| {
        if !node_ids.contains(&edge.source) {
            bail!("Edge {id:?} references non-existent source {}", edge.source);
        }
        if !node_ids.contains(&edge.target) {
            bail!("Edge {id:?} references non-existent target {}", edge.target);
        }
        Ok(())
    })?;

    // 3. Validate DAG property
    validate_dag(nodes, &doc.document.edges).map_err(|e| anyhow!("DAG Validation Failed: {e}"))?;

    Ok(())
}
