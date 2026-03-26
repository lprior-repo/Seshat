//! Z-order operations for diagram projection
//!
//! This module provides functions for applying z-order operations
//! (bring forward, send backward, bring to front, send to back)
//! to a diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::BTreeSet;

use crate::document::{Node, NodeId};
use crate::envelope::DomainOp;
use crate::z_order::{apply_z_order_reorder, ZOrderOp};
use im::HashMap;

use crate::projection::types::{DiagramProjection, ReplayError};

/// Type alias for node map
type NodeMap = HashMap<NodeId, Node>;

/// Validate input IDs and return selected node IDs
///
/// # Errors
/// Returns `ReplayError::NoNodesSpecified` if ids is empty
/// Returns `ReplayError::AllNodesInvalid` if no valid nodes found
fn validate_and_collect_selected(
    state: &DiagramProjection,
    ids: &[NodeId],
) -> Result<BTreeSet<NodeId>, ReplayError> {
    if ids.is_empty() {
        return Err(ReplayError::NoNodesSpecified);
    }

    let selected: BTreeSet<NodeId> = ids
        .iter()
        .filter(|id| state.has_node(id))
        .cloned()
        .collect();

    if selected.is_empty() {
        let invalid_ids = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ReplayError::AllNodesInvalid(invalid_ids));
    }

    Ok(selected)
}

/// Sort node IDs by their z-index
fn sort_nodes_by_z(state: &DiagramProjection) -> Vec<NodeId> {
    let mut node_ids: Vec<NodeId> = state.nodes.keys().cloned().collect();
    node_ids.sort_by(|a, b| {
        let z_a = state.nodes.get(a).map_or(0, |n| n.z_index);
        let z_b = state.nodes.get(b).map_or(0, |n| n.z_index);
        z_a.cmp(&z_b)
    });
    node_ids
}

/// Calculate min z-index from sorted node IDs
fn calculate_min_z(node_ids: &[NodeId], nodes: &NodeMap) -> i64 {
    node_ids
        .iter()
        .filter_map(|id| nodes.get(id).map(|n| n.z_index))
        .min()
        .unwrap_or(0)
}

/// Validate z-index won't overflow
fn validate_z_bounds(node_count: usize) -> Result<(), ReplayError> {
    let max_idx = node_count.saturating_sub(1);
    i64::try_from(max_idx).map_err(|_| ReplayError::ZIndexOverflow)?;
    Ok(())
}

/// Apply new z-values to nodes based on sorted order
fn apply_z_values(nodes: NodeMap, node_ids: &[NodeId], min_z: i64) -> NodeMap {
    node_ids
        .iter()
        .enumerate()
        .fold(nodes, |acc: NodeMap, (idx, id)| {
            let Some(node) = acc.get(id) else {
                return acc;
            };
            let new_z = min_z.saturating_add(idx as i64);
            let mut new_node = node.clone();
            new_node.z_index = new_z;
            acc.update(id.clone(), new_node)
        })
}

/// Build result projection with new nodes
fn build_result(state: DiagramProjection, new_nodes: NodeMap) -> DiagramProjection {
    DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    }
}

/// Apply z-ordering using a custom reorder function
fn apply_z_order(
    state: DiagramProjection,
    ids: &[NodeId],
    op: ZOrderOp,
) -> Result<DiagramProjection, ReplayError> {
    let selected = validate_and_collect_selected(&state, ids)?;
    let mut node_ids = sort_nodes_by_z(&state);
    apply_z_order_reorder(&mut node_ids, &selected, op);

    let min_z = calculate_min_z(&node_ids, &state.nodes);
    validate_z_bounds(node_ids.len())?;

    let new_nodes = apply_z_values(state.nodes.clone(), &node_ids, min_z);
    Ok(build_result(state, new_nodes))
}

/// Apply `BringForward` operation (z-order)
pub fn apply_bring_forward(
    state: DiagramProjection,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError> {
    apply_z_order(state, ids, ZOrderOp::BringForward)
}

/// Apply `SendBackward` operation (z-order)
pub fn apply_send_backward(
    state: DiagramProjection,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError> {
    apply_z_order(state, ids, ZOrderOp::SendBackward)
}

/// Apply `BringToFront` operation (z-order)
pub fn apply_bring_to_front(
    state: DiagramProjection,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError> {
    apply_z_order(state, ids, ZOrderOp::BringToFront)
}

/// Apply `SendToBack` operation (z-order)
pub fn apply_send_to_back(
    state: DiagramProjection,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError> {
    apply_z_order(state, ids, ZOrderOp::SendToBack)
}

/// Apply a z-order operation to the projection
pub fn apply_z_order_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    match op {
        DomainOp::BringForward { ids } => apply_bring_forward(state, ids),
        DomainOp::SendBackward { ids } => apply_send_backward(state, ids),
        DomainOp::BringToFront { ids } => apply_bring_to_front(state, ids),
        DomainOp::SendToBack { ids } => apply_send_to_back(state, ids.as_slice()),
        _ => Err(ReplayError::InvalidEvent(format!(
            "not a z-order operation: {:?}",
            op.kind()
        ))),
    }
}

#[cfg(test)]
#[path = "z_order_tests.rs"]
mod tests;
