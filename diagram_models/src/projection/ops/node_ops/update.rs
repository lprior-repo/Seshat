use crate::document::{Node, NodeId, NodeStyle};
use crate::projection::types::{DiagramProjection, ReplayError};

use super::build_projection;

pub fn apply_update_label(
    state: DiagramProjection,
    target_id: &str,
    new_label: &str,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(target_id.to_string());

    let node = state
        .nodes
        .get(&node_id)
        .ok_or_else(|| ReplayError::InvariantViolation(format!("node not found: {target_id}")))?
        .clone();

    let updated_node = Node {
        label: new_label.to_string(),
        ..node
    };

    let new_nodes = state.nodes.update(node_id, updated_node);
    let existing_edges = state.edges.clone();
    Ok(build_projection(state, new_nodes, existing_edges))
}

pub fn apply_update_node_style(
    state: DiagramProjection,
    id: &str,
    style: NodeStyle,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());

    let node = state
        .nodes
        .get(&node_id)
        .ok_or_else(|| ReplayError::InvariantViolation(format!("node not found: {id}")))?
        .clone();

    let updated_node = Node {
        style: Some(style),
        ..node
    };

    let new_nodes = state.nodes.update(node_id, updated_node);
    let existing_edges = state.edges.clone();
    Ok(build_projection(state, new_nodes, existing_edges))
}
