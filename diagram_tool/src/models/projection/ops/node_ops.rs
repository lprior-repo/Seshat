//! Node operations for diagram projection
//!
//! This module provides functions for applying node-related operations
//! to a diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use im::HashMap;

use crate::models::document::{Edge, EdgeId, Node, NodeId, NodeKind, OrderedFloat};
use crate::models::envelope::DomainOp;

use crate::models::projection::types::{DiagramProjection, ReplayError};

/// Type alias for edge map
type EdgeMap = HashMap<EdgeId, Edge>;

/// Apply `NodeAdd` operation
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

    // Check for duplicate node ID
    if state.has_node(&node_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "duplicate node ID: {id}"
        )));
    }

    let new_nodes = state
        .nodes
        .update(node_id, create_default_node(x, y, width, height, label));

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Create a default node with specified geometry and label
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
        locked: false,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

/// Apply `NodeMove` operation
pub fn apply_node_move(
    state: DiagramProjection,
    id: &str,
    x: f64,
    y: f64,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());

    // Check node exists
    let node = state
        .nodes
        .get(&node_id)
        .ok_or_else(|| ReplayError::InvariantViolation(format!("node not found: {id}")))?
        .clone();

    // Create updated node with new position
    let updated_node = Node {
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        ..node
    };

    let new_nodes = state.nodes.update(node_id, updated_node);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `NodeDelete` operation
pub fn apply_node_delete(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());

    // Check node exists
    if !state.has_node(&node_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "node not found: {id}"
        )));
    }

    // Also remove edges connected to this node
    let edges_to_remove: Vec<EdgeId> = state
        .edges
        .iter()
        .filter(|(_, edge)| edge.source == node_id || edge.target == node_id)
        .map(|(id, _)| id.clone())
        .collect();

    // Use without() which returns HashMap, not Option
    let new_edges: EdgeMap = edges_to_remove
        .into_iter()
        .fold(state.edges.clone(), |acc: EdgeMap, eid| acc.without(&eid));

    let new_nodes = state.nodes.without(&node_id);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: new_edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `NodeRestore` operation
///
/// Note: This is currently a no-op because the projection model doesn't track deleted nodes.
/// A proper implementation would require maintaining a "deleted nodes" set to restore from.
/// Currently returns an error if the node doesn't exist (can't restore what was never added),
/// and returns the state unchanged if the node already exists.
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

/// Apply a node operation to the projection
///
/// This is the contract-specified entry point for applying node operations.
/// It dispatches to the appropriate handler based on the operation type.
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
        } => apply_node_add(state, id, *x, *y, *width, *height, label),
        DomainOp::NodeMove { id, x, y } => apply_node_move(state, id, *x, *y),
        DomainOp::NodeDelete { id } => apply_node_delete(state, id),
        DomainOp::NodeRestore { id } => apply_node_restore(state, id),
        _ => Err(ReplayError::InvalidEvent(format!(
            "not a node operation: {:?}",
            op.kind()
        ))),
    }
}
