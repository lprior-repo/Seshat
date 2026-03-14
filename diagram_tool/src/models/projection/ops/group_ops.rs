//! Group operations for diagram projection
//!
//! This module provides functions for applying group/ungroup operations
//! to a diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use im::HashMap;
use std::collections::BTreeSet;

use crate::core::grouping::{
    calculate_edge_cleanup, calculate_ungroup, compute_padded_bounds, create_subgraph_node,
    find_lca, validate_selection,
};
use crate::models::document::{Node, NodeId, NodeKind, OrderedFloat};
use crate::models::envelope::DomainOp;

use crate::models::projection::types::{DiagramProjection, ReplayError};

/// Type alias for node map
type NodeMap = HashMap<NodeId, Node>;

/// Apply Group operation - creates a subgraph and assigns all specified nodes as children
pub fn apply_group(
    state: DiagramProjection,
    group_id: &NodeId,
    ids: &[NodeId],
) -> Result<DiagramProjection, ReplayError> {
    let mut nodes = state.nodes.clone();
    let selected_ids: im::HashSet<NodeId> = ids.iter().cloned().collect();
    let selected_strings: im::HashSet<String> =
        ids.iter().map(|id| id.as_str().to_string()).collect();

    validate_selection(&nodes, &selected_strings).map_err(
        |e: crate::core::grouping::GroupingError| ReplayError::InvariantViolation(e.to_string()),
    )?;

    let (padded_min_x, padded_min_y, width, height) = compute_padded_bounds(&nodes, &selected_ids)
        .map_err(|e: crate::core::grouping::GroupingError| {
            ReplayError::InvariantViolation(e.to_string())
        })?;

    // Q5: Z-Index Consistency
    let min_z = selected_ids
        .iter()
        .filter_map(|id| nodes.get(id).map(|n| n.z_index))
        .min()
        .unwrap_or(0);

    // Q6: Parent Assignment (LCA)
    let parent_id = find_lca(&nodes, &selected_ids);

    let group_node = create_subgraph_node(
        padded_min_x,
        padded_min_y,
        width,
        height,
        min_z - 1,
        parent_id,
    )
    .ok_or_else(|| ReplayError::InvariantViolation("subgraph too small".to_string()))?;

    // Insert group node
    nodes.insert(group_id.clone(), group_node);

    // Update children with parent reference
    for id in ids {
        if let Some(node) = nodes.get_mut(id) {
            node.parent = Some(group_id.clone());
        }
    }

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply Ungroup operation - removes the subgraph node and clears parent on all children
pub fn apply_ungroup(
    state: DiagramProjection,
    id: &NodeId,
) -> Result<DiagramProjection, ReplayError> {
    if !state.has_node(id) {
        return Err(ReplayError::InvariantViolation(format!(
            "subgraph not found: {}",
            id
        )));
    }

    let node = state
        .nodes
        .get(id)
        .ok_or_else(|| ReplayError::InvariantViolation("node not found".to_string()))?;
    if node.kind != NodeKind::Subgraph {
        return Err(ReplayError::InvariantViolation(format!(
            "node is not a subgraph: {}",
            id
        )));
    }

    let mut target_subgraphs = BTreeSet::new();
    target_subgraphs.insert(id.clone());

    let (new_nodes, _) = calculate_ungroup(&state.nodes, &target_subgraphs);
    let new_edges = calculate_edge_cleanup(&state.edges, &target_subgraphs);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: new_edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply a group operation to the projection
pub fn apply_group_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    match op {
        DomainOp::Group { id, ids } => apply_group(state, id, ids.as_slice()),
        DomainOp::Ungroup { id } => apply_ungroup(state, id),
        _ => Err(ReplayError::InvalidEvent(format!(
            "not a group operation: {:?}",
            op.kind()
        ))),
    }
}
