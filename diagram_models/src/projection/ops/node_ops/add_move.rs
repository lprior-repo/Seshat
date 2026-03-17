use im::HashMap;

use crate::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};
use crate::projection::ops::node_bounds::propagate_bounds_to_ancestors;
use crate::projection::types::{DiagramProjection, ReplayError};

use super::build_projection;

pub fn apply_node_add(
    state: DiagramProjection,
    id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    label: &str,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());
    let node_id_for_propagation = node_id.clone();

    if state.has_node(&node_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "duplicate node ID: {id}"
        )));
    }

    let new_nodes = state
        .nodes
        .update(node_id, create_default_node(x, y, width, height, label));
    let new_nodes = propagate_bounds_to_ancestors(new_nodes, &node_id_for_propagation);
    let existing_edges = state.edges.clone();

    Ok(build_projection(state, new_nodes, existing_edges))
}

pub fn create_default_node(x: f64, y: f64, width: f64, height: f64, label: &str) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: label.to_string(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(width),
        height: OrderedFloat(height),
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
    }
}

pub fn apply_node_move(
    state: DiagramProjection,
    id: &str,
    x: f64,
    y: f64,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());
    let node_id_for_propagation = node_id.clone();

    let node = state
        .nodes
        .get(&node_id)
        .ok_or_else(|| ReplayError::InvariantViolation(format!("node not found: {id}")))?
        .clone();

    let updated_node = Node {
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        ..node
    };

    let new_nodes = state.nodes.update(node_id, updated_node);
    let new_nodes = propagate_bounds_to_ancestors(new_nodes, &node_id_for_propagation);
    let existing_edges = state.edges.clone();

    Ok(build_projection(state, new_nodes, existing_edges))
}
