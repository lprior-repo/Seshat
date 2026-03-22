//! Group operations for diagram projection
//!
//! This module provides functions for applying group/ungroup operations
//! to a diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use im::HashMap;
use std::collections::BTreeSet;

use crate::document::{Node, NodeId, NodeKind, OrderedFloat};
use crate::envelope::DomainOp;
use crate::grouping::{
    calculate_edge_cleanup, calculate_ungroup, compute_padded_bounds, create_subgraph_node,
    find_lca, validate_selection,
};

use crate::projection::types::{DiagramProjection, ReplayError};

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
        |e: crate::grouping::GroupingError| ReplayError::InvariantViolation(e.to_string()),
    )?;

    let (padded_min_x, padded_min_y, width, height) = compute_padded_bounds(&nodes, &selected_ids)
        .map_err(|e: crate::grouping::GroupingError| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{LockState, NodeKind};

    fn test_node(x: f64, y: f64, w: f64, h: f64, kind: NodeKind) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: "test".to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn test_apply_group_success() {
        let mut state = DiagramProjection::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());

        state
            .nodes
            .insert(n1.clone(), test_node(0.0, 0.0, 10.0, 10.0, NodeKind::Node));
        state
            .nodes
            .insert(n2.clone(), test_node(20.0, 0.0, 10.0, 10.0, NodeKind::Node));

        let g1 = NodeId::new("g1".to_string());
        let result = apply_group(state, &g1, &[n1.clone(), n2.clone()]).unwrap();

        assert!(result.nodes.contains_key(&g1));
        let g_node = result.nodes.get(&g1).unwrap();
        assert_eq!(g_node.kind, NodeKind::Subgraph);

        assert_eq!(result.nodes.get(&n1).unwrap().parent, Some(g1.clone()));
        assert_eq!(result.nodes.get(&n2).unwrap().parent, Some(g1.clone()));
    }

    #[test]
    fn test_apply_group_invalid_selection() {
        let state = DiagramProjection::default();
        let g1 = NodeId::new("g1".to_string());
        let result = apply_group(state, &g1, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_ungroup_success() {
        let mut state = DiagramProjection::default();
        let g1 = NodeId::new("g1".to_string());
        let n1 = NodeId::new("n1".to_string());

        let mut child = test_node(10.0, 10.0, 10.0, 10.0, NodeKind::Node);
        child.parent = Some(g1.clone());

        state.nodes.insert(
            g1.clone(),
            test_node(0.0, 0.0, 100.0, 100.0, NodeKind::Subgraph),
        );
        state.nodes.insert(n1.clone(), child);

        let result = apply_ungroup(state, &g1).unwrap();

        assert!(!result.nodes.contains_key(&g1));
        assert_eq!(result.nodes.get(&n1).unwrap().parent, None);
    }

    #[test]
    fn test_apply_ungroup_not_found() {
        let state = DiagramProjection::default();
        let result = apply_ungroup(state, &NodeId::new("missing".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_ungroup_not_subgraph() {
        let mut state = DiagramProjection::default();
        let n1 = NodeId::new("n1".to_string());
        state
            .nodes
            .insert(n1.clone(), test_node(0.0, 0.0, 10.0, 10.0, NodeKind::Node));

        let result = apply_ungroup(state, &n1);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_group_op_dispatch() {
        let state = DiagramProjection::default();
        let g1 = NodeId::new("g1".to_string());
        let op_group = DomainOp::Group {
            id: g1.clone(),
            ids: vec![],
        };
        let result = apply_group_op(state.clone(), &op_group);
        assert!(result.is_err()); // empty selection err

        let op_ungroup = DomainOp::Ungroup { id: g1.clone() };
        let result_ungroup = apply_group_op(state.clone(), &op_ungroup);
        assert!(result_ungroup.is_err()); // missing node err

        let op_invalid = DomainOp::NodeDelete { id: g1.clone() };
        let result_invalid = apply_group_op(state, &op_invalid);
        assert!(result_invalid.is_err());
    }
}
