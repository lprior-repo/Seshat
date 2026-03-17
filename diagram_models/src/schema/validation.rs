use crate::dag::validate_dag;
use crate::document::{DiagramDocument, NodeId, NodeKind};
use anyhow::{anyhow, bail, Result};
use im::HashSet;

/// Functional schema validation.
///
/// # Errors
///
/// Returns an error if document version is not 2, nodes have invalid properties,
/// edges reference missing nodes, or the graph contains cycles.
pub fn validate_schema(doc: &DiagramDocument) -> Result<()> {
    if doc.version != 2 {
        bail!("Document version must be 2, got {}", doc.version);
    }

    let nodes = &doc.document.nodes;
    let node_ids = nodes.keys().cloned().collect::<HashSet<NodeId>>();

    // 1. Validate Nodes
    nodes.iter().try_for_each(|(id, node)| {
        if !node.x.0.is_finite() {
            bail!("Node {id} has non-finite x: {}", node.x.0);
        }
        if !node.y.0.is_finite() {
            bail!("Node {id} has non-finite y: {}", node.y.0);
        }
        if node.width.0 < 0.0 || !node.width.0.is_finite() {
            bail!("Node {id} has invalid width: {}", node.width.0);
        }
        if node.height.0 < 0.0 || !node.height.0.is_finite() {
            bail!("Node {id} has invalid height: {}", node.height.0);
        }
        if !node.width.0.is_finite() {
            bail!("Node {id} has non-finite width: {}", node.width.0);
        }
        if !node.height.0.is_finite() {
            bail!("Node {id} has non-finite height: {}", node.height.0);
        }
        if !node.x.0.is_finite() {
            bail!("Node {id} has non-finite x coordinate: {}", node.x.0);
        }
        if !node.y.0.is_finite() {
            bail!("Node {id} has non-finite y coordinate: {}", node.y.0);
        }
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

    // 1b. Check for circular parent chains using functional recursion
    for (id, _) in nodes.iter() {
        let has_cycle = check_parent_cycle(nodes, id, &HashSet::new());
        if has_cycle {
            bail!("Circular parent chain detected involving node {id}");
        }
    }

    // 2. Validate Edges and DAG
    validate_edges_and_dag(doc)?;

    Ok(())
}

#[allow(clippy::redundant_clone)]
fn check_parent_cycle(
    nodes: &im::HashMap<NodeId, crate::document::Node>,
    current: &NodeId,
    visited: &im::HashSet<NodeId>,
) -> bool {
    if visited.contains(current) {
        return true;
    }
    let mut next_visited = visited.clone();
    next_visited.insert(current.clone());

    nodes
        .get(current)
        .and_then(|n| n.parent.as_ref())
        .is_some_and(|parent| check_parent_cycle(nodes, parent, &next_visited))
}

/// Validate edges and DAG after parent chain validation
fn validate_edges_and_dag(doc: &DiagramDocument) -> Result<()> {
    let nodes = &doc.document.nodes;
    let node_ids = nodes.keys().cloned().collect::<HashSet<NodeId>>();

    // 2. Validate Edges
    doc.document.edges.iter().try_for_each(|(id, edge)| {
        if !node_ids.contains(&edge.source) {
            bail!("Edge {id:?} references non-existent source {}", edge.source);
        }
        if !node_ids.contains(&edge.target) {
            bail!("Edge {id:?} references non-existent target {}", edge.target);
        }
        if !edge.label_offset_t.0.is_finite() {
            bail!(
                "Edge {id:?} has non-finite label_offset_t: {}",
                edge.label_offset_t.0
            );
        }
        if edge.label_offset_t.0 < 0.0 || edge.label_offset_t.0 > 1.0 {
            bail!(
                "Edge {id:?} has label_offset_t {} outside valid range [0, 1]",
                edge.label_offset_t.0
            );
        }
        if !edge.thickness.0.is_finite() {
            bail!("Edge {id:?} has non-finite thickness: {}", edge.thickness.0);
        }
        if let Some(ref color) = edge.color {
            if !is_valid_hex_color(color) {
                bail!("Edge {id:?} has invalid color format: {color}");
            }
        }
        if let Some(ref font_size) = edge.font_size {
            if !font_size.0.is_finite() {
                bail!("Edge {id:?} has non-finite font_size: {}", font_size.0);
            }
        }
        Ok(())
    })?;

    // 3. Validate Editor State
    let es = &doc.editor_state;
    if !es.camera_x.0.is_finite() {
        bail!("Editor state has non-finite camera_x: {}", es.camera_x.0);
    }
    if !es.camera_y.0.is_finite() {
        bail!("Editor state has non-finite camera_y: {}", es.camera_y.0);
    }
    if !es.zoom.0.is_finite() {
        bail!("Editor state has non-finite zoom: {}", es.zoom.0);
    }

    // 4. Validate DAG property
    validate_dag(nodes, &doc.document.edges).map_err(|e| anyhow!("DAG Validation Failed: {e}"))?;

    Ok(())
}

fn is_valid_hex_color(color: &str) -> bool {
    color.starts_with('#')
        && match color.len() {
            4 => {
                // #RGB
                color[1..].chars().all(|c| c.is_ascii_hexdigit())
            }
            7 => {
                // #RRGGBB
                color[1..].chars().all(|c| c.is_ascii_hexdigit())
            }
            5 => {
                // #RGBA
                color[1..].chars().all(|c| c.is_ascii_hexdigit())
            }
            9 => {
                // #RRGGBBAA
                color[1..].chars().all(|c| c.is_ascii_hexdigit())
            }
            _ => false,
        }
}
