use crate::document::NodeId;
use crate::envelope::DomainOp;
use crate::projection::ops::node_bounds::{get_parent_containers, recompute_container_bounds};
use crate::projection::types::{DiagramProjection, ReplayError};

use super::add_move::{apply_node_add, apply_node_move};
use super::{build_projection, EdgeMap, NodeMap};

fn get_children(node_map: &NodeMap, parent_id: &NodeId) -> Vec<NodeId> {
    node_map
        .iter()
        .filter(|(_, node)| node.parent.as_ref() == Some(parent_id))
        .map(|(id, _)| id.clone())
        .collect()
}

fn unparent_children(node_map: &NodeMap, parent_id: &NodeId) -> NodeMap {
    let children = get_children(node_map, parent_id);
    children
        .iter()
        .fold(node_map.clone(), |acc: NodeMap, child_id| {
            acc.alter(
                |child_opt| {
                    child_opt.map(|mut child| {
                        child.parent = None;
                        child
                    })
                },
                child_id.clone(),
            )
        })
}

pub fn apply_node_delete(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());

    if !state.has_node(&node_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "node not found: {id}"
        )));
    }

    let parent_containers = get_parent_containers(&state.nodes, &node_id);
    let edges_to_remove: Vec<_> = state
        .edges
        .iter()
        .filter(|(_, edge)| edge.source == node_id || edge.target == node_id)
        .map(|(id, _)| id.clone())
        .collect();

    let new_edges: EdgeMap = edges_to_remove
        .into_iter()
        .fold(state.edges.clone(), |acc: EdgeMap, eid| acc.without(&eid));

    let mut new_nodes = unparent_children(&state.nodes, &node_id).without(&node_id);

    for container_id in parent_containers {
        if let Some(updated_container) = recompute_container_bounds(&new_nodes, &container_id) {
            new_nodes = new_nodes.update(container_id, updated_container);
        }
    }

    Ok(build_projection(state, new_nodes, new_edges))
}

pub fn apply_node_restore(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());

    if !state.has_node(&node_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "node not found for restore: {id}"
        )));
    }

    Ok(state)
}

pub fn apply_node_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    match op {
        DomainOp::NodeAdd {
            id,
            x,
            y,
            width,
            height,
            label,
        } => apply_node_add(state, id.as_str(), *x, *y, *width, *height, label),
        DomainOp::NodeMove { id, x, y } => apply_node_move(state, id.as_str(), *x, *y),
        DomainOp::NodeDelete { id } => apply_node_delete(state, id.as_str()),
        DomainOp::NodeRestore { id } => apply_node_restore(state, id.as_str()),
        _ => Err(ReplayError::InvalidEvent(format!(
            "not a node operation: {:?}",
            op.kind()
        ))),
    }
}
