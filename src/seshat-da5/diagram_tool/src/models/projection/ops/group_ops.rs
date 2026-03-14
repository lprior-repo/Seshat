//! Group operations for diagram projection
//!
//! This module provides functions for applying group/ungroup operations
//! to a diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use im::HashMap;
use uuid::Uuid;

use crate::models::document::{Node, NodeId, NodeKind, OrderedFloat, MAX_SUBGRAPH_NESTING_DEPTH};
use crate::models::envelope::DomainOp;

use crate::models::projection::types::{DiagramProjection, ReplayError};

/// Type alias for node map
type NodeMap = HashMap<NodeId, Node>;

/// Count nesting depth of a node in the graph
fn count_nesting_depth(nodes: &NodeMap, parent: Option<&NodeId>) -> usize {
    parent.and_then(|pid| nodes.get(pid)).map_or(0, |node| {
        1 + count_nesting_depth(nodes, node.parent.as_ref())
    })
}

/// Check if grouping these nodes would exceed max nesting depth
fn check_nesting_depth(nodes: &NodeMap, ids: &[NodeId]) -> Result<(), ReplayError> {
    for id in ids {
        if let Some(node) = nodes.get(id) {
            let depth = count_nesting_depth(nodes, node.parent.as_ref());
            if depth >= MAX_SUBGRAPH_NESTING_DEPTH {
                return Err(ReplayError::NestedSubgraphLimitExceeded(
                    MAX_SUBGRAPH_NESTING_DEPTH,
                ));
            }
        }
    }
    Ok(())
}

/// Apply Group operation - creates a subgraph and assigns all specified nodes as children
pub fn apply_group(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    let valid_ids = validate_group_ids(&state, ids)?;
    check_nesting_depth(&state.nodes, &valid_ids)?;
    let (min_x, min_y, max_x, max_y) = compute_bounding_box(&state, &valid_ids)?;
    let (group_node, group_id) = create_group_node(min_x, min_y, max_x, max_y)?;
    let new_nodes = add_group_and_update_children(state.nodes, &group_node, &group_id, &valid_ids);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Validate IDs for grouping operation
fn validate_group_ids(
    state: &DiagramProjection,
    ids: &[String],
) -> Result<Vec<NodeId>, ReplayError> {
    if ids.is_empty() {
        return Err(ReplayError::NoNodesSpecified);
    }

    let node_ids: Vec<NodeId> = ids.iter().map(|s| NodeId::new(s.clone())).collect();

    let valid_ids: Vec<NodeId> = node_ids
        .iter()
        .filter(|id| state.has_node(id))
        .cloned()
        .collect();

    if valid_ids.len() < 2 {
        let invalid_ids = ids.join(", ");
        return Err(ReplayError::AllNodesInvalid(invalid_ids));
    }

    Ok(valid_ids)
}

/// Compute bounding box for a set of nodes
fn compute_bounding_box(
    state: &DiagramProjection,
    valid_ids: &[NodeId],
) -> Result<(f64, f64, f64, f64), ReplayError> {
    let bounds = valid_ids
        .iter()
        .filter_map(|id| state.nodes.get(id))
        .map(|n| (n.x.0, n.y.0, n.x.0 + n.width.0, n.y.0 + n.height.0))
        .fold(init_bounds(), extend_bounds);
    validate_bounds_finite(bounds)
}

/// Initialize bounds to infinity values
fn init_bounds() -> (f64, f64, f64, f64) {
    (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    )
}

/// Extend bounds with new coordinates
fn extend_bounds(
    (min_x, min_y, max_x, max_y): (f64, f64, f64, f64),
    (x, y, right, bottom): (f64, f64, f64, f64),
) -> (f64, f64, f64, f64) {
    (
        min_x.min(x),
        min_y.min(y),
        max_x.max(right),
        max_y.max(bottom),
    )
}

/// Validate bounding box coordinates are finite
fn validate_bounds_finite(
    bounds: (f64, f64, f64, f64),
) -> Result<(f64, f64, f64, f64), ReplayError> {
    let (min_x, min_y, max_x, max_y) = bounds;
    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return Err(ReplayError::InvariantViolation(
            "invalid node coordinates for grouping".to_string(),
        ));
    }
    Ok(bounds)
}

/// Create a group node with calculated bounds
fn create_group_node(
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> Result<(Node, NodeId), ReplayError> {
    let group_id = NodeId::new(format!("group-{}", Uuid::new_v4()));
    let node = build_group_node(min_x, min_y, max_x, max_y);
    Ok((node, group_id))
}

/// Build group node fields
fn build_group_node(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Node {
    let padding = 24.0;
    Node {
        kind: NodeKind::Subgraph,
        icon: String::new(),
        label: "Group".to_string(),
        x: OrderedFloat(min_x - padding),
        y: OrderedFloat(min_y - padding),
        width: OrderedFloat((max_x - min_x) + (padding * 2.0)),
        height: OrderedFloat((max_y - min_y) + (padding * 2.0)),
        font_size: None,
        font_weight: None,
        locked: true,
        parent: None,
        dag_rank: None,
        tags: im::vector![],
        metadata: HashMap::new(),
        z_index: -1,
        style: Some(crate::models::document::NodeStyle::Box),
        collapsed: Some(false),
    }
}

/// Add group node and update children with parent reference
fn add_group_and_update_children(
    nodes: NodeMap,
    group_node: &Node,
    group_id: &NodeId,
    child_ids: &[NodeId],
) -> NodeMap {
    // Insert group node
    let new_nodes = nodes.update(group_id.clone(), group_node.clone());

    // Update children with parent reference
    child_ids.iter().fold(new_nodes, |acc, id| {
        if let Some(node) = acc.get(id) {
            let mut updated_node = node.clone();
            updated_node.parent = Some(group_id.clone());
            acc.update(id.clone(), updated_node)
        } else {
            acc
        }
    })
}

/// Apply Ungroup operation - removes the subgraph node and clears parent on all children
pub fn apply_ungroup(state: DiagramProjection, id: &str) -> Result<DiagramProjection, ReplayError> {
    let subgraph_id = NodeId::new(id.to_string());
    validate_subgraph_exists(&state, &subgraph_id, id)?;
    let children = find_child_nodes(&state, &subgraph_id);
    let new_nodes = unparent_children_and_remove_group(&state, &subgraph_id, &children);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Validate that a subgraph exists and is actually a subgraph
fn validate_subgraph_exists(
    state: &DiagramProjection,
    subgraph_id: &NodeId,
    id: &str,
) -> Result<(), ReplayError> {
    if !state.has_node(subgraph_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "subgraph not found: {}",
            id
        )));
    }

    let subgraph = state.nodes.get(subgraph_id).cloned();
    match subgraph {
        Some(s) if s.kind == NodeKind::Subgraph => Ok(()),
        _ => Err(ReplayError::InvariantViolation(format!(
            "node is not a subgraph: {}",
            id
        ))),
    }
}

/// Find all child nodes of a given parent subgraph
fn find_child_nodes(state: &DiagramProjection, subgraph_id: &NodeId) -> Vec<NodeId> {
    state
        .nodes
        .iter()
        .filter(|(_, node)| node.parent.as_ref() == Some(subgraph_id))
        .map(|(id, _)| id.clone())
        .collect()
}

/// Unparent all children and remove the group node
fn unparent_children_and_remove_group(
    state: &DiagramProjection,
    subgraph_id: &NodeId,
    children: &[NodeId],
) -> NodeMap {
    children
        .iter()
        .fold(state.nodes.clone(), |acc: NodeMap, child_id| {
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
        .without(subgraph_id)
}

/// Apply a group operation to the projection
pub fn apply_group_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    match op {
        DomainOp::Group { ids } => apply_group(state, ids),
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
    use crate::models::document::{Node, NodeId, NodeKind, MAX_SUBGRAPH_NESTING_DEPTH};

    #[test]
    fn test_count_nesting_depth_recursive() {
        let mut nodes = HashMap::new();
        let s1_id = NodeId::new("s1".to_string());
        let s2_id = NodeId::new("s2".to_string());
        let n1_id = NodeId::new("n1".to_string());

        let mut s1 = Node {
            kind: NodeKind::Subgraph,
            icon: String::new(),
            label: "S1".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
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
        };
        let mut s2 = s1.clone();
        s2.parent = Some(s1_id.clone());
        let mut n1 = s1.clone();
        n1.kind = NodeKind::Node;
        n1.parent = Some(s2_id.clone());

        nodes.insert(s1_id.clone(), s1);
        nodes.insert(s2_id.clone(), s2);
        nodes.insert(n1_id.clone(), n1);

        assert_eq!(count_nesting_depth(&nodes, None), 0);
        assert_eq!(count_nesting_depth(&nodes, Some(&s1_id)), 1);
        assert_eq!(count_nesting_depth(&nodes, Some(&s2_id)), 2);
    }

    #[test]
    fn test_apply_group_depth_limit() {
        let mut state = DiagramProjection::empty();

        // Setup S1 > S2 > S3 > S4 > S5
        let mut last_parent = None;
        for i in 1..=MAX_SUBGRAPH_NESTING_DEPTH {
            let id = format!("s{}", i);
            let mut node = Node {
                kind: NodeKind::Subgraph,
                icon: String::new(),
                label: format!("S{}", i),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(100.0),
                height: OrderedFloat(100.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: last_parent.clone(),
                dag_rank: None,
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            let node_id = NodeId::new(id.clone());
            state.nodes.insert(node_id.clone(), node);
            last_parent = Some(node_id);
        }

        // Add nodes at depth 5
        let n1_id = NodeId::new("n1".to_string());
        let n2_id = NodeId::new("n2".to_string());
        let n_template = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "N".to_string(),
            x: OrderedFloat(10.0),
            y: OrderedFloat(10.0),
            width: OrderedFloat(10.0),
            height: OrderedFloat(10.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: last_parent,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        state.nodes.insert(n1_id.clone(), n_template.clone());
        state.nodes.insert(n2_id.clone(), n_template);

        let result = apply_group(state, &["n1".to_string(), "n2".to_string()]);
        assert!(result.is_err());
        match result {
            Err(ReplayError::NestedSubgraphLimitExceeded(limit)) => {
                assert_eq!(limit, MAX_SUBGRAPH_NESTING_DEPTH);
            }
            _ => panic!("Expected NestedSubgraphLimitExceeded, got {:?}", result),
        }
    }
}
