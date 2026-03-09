//! Z-order operations for diagram projection
//!
//! Functions for applying z-order (layering) domain operations.

use std::collections::BTreeSet;

use crate::models::document::NodeId;
use crate::models::projection::types::{DiagramProjection, ReplayError};

/// Validates that ids are not empty and at least some exist in state
fn validate_selected_ids(
    state: &DiagramProjection,
    ids: &[String],
) -> Result<BTreeSet<NodeId>, ReplayError> {
    if ids.is_empty() {
        return Err(ReplayError::NoNodesSpecified);
    }

    let selected: BTreeSet<NodeId> = ids
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .filter(|id| state.has_node(id))
        .collect();

    if selected.is_empty() {
        let invalid_ids = ids.join(", ");
        return Err(ReplayError::AllNodesInvalid(invalid_ids));
    }

    Ok(selected)
}

/// Sort node IDs by their z-index
fn sort_nodes_by_z_index(state: &DiagramProjection) -> Vec<NodeId> {
    let mut node_ids: Vec<NodeId> = state.nodes.keys().cloned().collect();
    node_ids.sort_by(|a, b| {
        let z_a = state.nodes.get(a).map_or(0, |n| n.z_index);
        let z_b = state.nodes.get(b).map_or(0, |n| n.z_index);
        z_a.cmp(&z_b)
    });
    node_ids
}

/// Reassign z-indices to nodes based on their order in the list
fn reassign_z_indices(
    state: DiagramProjection,
    ordered_ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError> {
    let min_z = ordered_ids
        .iter()
        .filter_map(|id| state.nodes.get(id).map(|n| n.z_index))
        .min()
        .unwrap_or(0);

    let max_idx = ordered_ids.len().saturating_sub(1);
    let _ = i64::try_from(max_idx).map_err(|_| ReplayError::ZIndexOverflow)?;

    let new_nodes = ordered_ids
        .iter()
        .enumerate()
        .fold(state.nodes, |acc, (idx, id)| {
            let Some(node) = acc.get(id) else {
                return acc;
            };
            let new_z = min_z.saturating_add(idx as i64);
            let mut new_node = node.clone();
            new_node.z_index = new_z;
            acc.update(id.clone(), new_node)
        });

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `BringForward` operation (z-order)
pub fn apply_bring_forward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    let selected = validate_selected_ids(&state, ids)?;
    let mut node_ids = sort_nodes_by_z_index(&state);

    for idx in (0..node_ids.len() - 1).rev() {
        let current_selected = selected.contains(&node_ids[idx]);
        let next_selected = selected.contains(&node_ids[idx + 1]);
        if current_selected && !next_selected {
            node_ids.swap(idx, idx + 1);
        }
    }

    reassign_z_indices(state, &node_ids)
}

/// Apply `SendBackward` operation (z-order)
pub fn apply_send_backward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    let selected = validate_selected_ids(&state, ids)?;
    let mut node_ids = sort_nodes_by_z_index(&state);

    for idx in 1..node_ids.len() {
        let current_selected = selected.contains(&node_ids[idx]);
        let previous_selected = selected.contains(&node_ids[idx - 1]);
        if current_selected && !previous_selected {
            node_ids.swap(idx - 1, idx);
        }
    }

    reassign_z_indices(state, &node_ids)
}

/// Apply `BringToFront` operation (z-order)
pub fn apply_bring_to_front(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    let selected = validate_selected_ids(&state, ids)?;
    let sorted_node_ids = sort_nodes_by_z_index(&state);

    let reordered: Vec<NodeId> = sorted_node_ids
        .iter()
        .filter(|id| !selected.contains(*id))
        .cloned()
        .chain(
            sorted_node_ids
                .iter()
                .filter(|id| selected.contains(*id))
                .cloned(),
        )
        .collect();

    reassign_z_indices(state, &reordered)
}

/// Apply `SendToBack` operation (z-order)
pub fn apply_send_to_back(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    let selected = validate_selected_ids(&state, ids)?;
    let sorted_node_ids = sort_nodes_by_z_index(&state);

    let reordered: Vec<NodeId> = sorted_node_ids
        .iter()
        .filter(|id| selected.contains(*id))
        .cloned()
        .chain(
            sorted_node_ids
                .iter()
                .filter(|id| !selected.contains(*id))
                .cloned(),
        )
        .collect();

    reassign_z_indices(state, &reordered)
}
