use crate::document::NodeId;
use crate::envelope::DomainOp;
use crate::projection::ops::node_bounds::{get_parent_containers, recompute_container_bounds};
use crate::projection::types::{DiagramProjection, ReplayError};
use smallvec::SmallVec;

use super::add_move::{apply_node_add, apply_node_move};
use super::{build_projection, EdgeMap, NodeMap};

fn get_children(node_map: &NodeMap, parent_id: &NodeId) -> SmallVec<[NodeId; 8]> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::edge::{ArrowType, Edge, EdgeStyle};
    use crate::document::node::{LockState, Node, NodeKind};
    use crate::document::types::{EdgeId, NodeId, OrderedFloat};
    use crate::projection::types::DiagramProjection;

    fn test_node() -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "test".to_string(),
            font_size: None,
            font_weight: None,
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(10.0),
            height: OrderedFloat(10.0),
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

    fn test_edge(source: NodeId, target: NodeId) -> Edge {
        Edge {
            source,
            target,
            label: String::new(),
            style: EdgeStyle::Solid,
            arrow_type: ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.0),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    #[test]
    fn test_apply_node_delete_removes_node_and_connected_edges() {
        let node1_id = NodeId::new("n1".to_string());
        let node2_id = NodeId::new("n2".to_string());
        let edge_id = EdgeId::new("e1".to_string());

        let node1 = test_node();
        let mut node2 = test_node();
        node2.x = OrderedFloat(20.0);
        let edge = test_edge(node1_id.clone(), node2_id.clone());

        let mut state = DiagramProjection::default();
        state.nodes = state.nodes.update(node1_id.clone(), node1);
        state.nodes = state.nodes.update(node2_id.clone(), node2);
        state.edges = state.edges.update(edge_id.clone(), edge);

        let result = apply_node_delete(state, "n1").unwrap();
        assert!(!result.has_node(&node1_id));
        assert!(result.has_node(&node2_id));
        assert!(result.edges.is_empty());
    }

    #[test]
    fn test_apply_node_delete_unparents_children() {
        let parent_id = NodeId::new("p1".to_string());
        let child_id = NodeId::new("c1".to_string());

        let mut parent = test_node();
        parent.width = OrderedFloat(100.0);
        parent.height = OrderedFloat(100.0);

        let mut child = test_node();
        child.x = OrderedFloat(10.0);
        child.y = OrderedFloat(10.0);
        child.parent = Some(parent_id.clone());

        let mut state = DiagramProjection::default();
        state.nodes = state.nodes.update(parent_id.clone(), parent);
        state.nodes = state.nodes.update(child_id.clone(), child);

        let result = apply_node_delete(state, "p1").unwrap();
        assert!(!result.has_node(&parent_id));
        assert!(result.has_node(&child_id));
        assert_eq!(result.nodes.get(&child_id).unwrap().parent, None);
    }

    #[test]
    fn test_apply_node_delete_missing_node() {
        let state = DiagramProjection::default();
        let result = apply_node_delete(state, "missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_node_restore_success() {
        let node_id = NodeId::new("n1".to_string());
        let node = test_node();

        let mut state = DiagramProjection::default();
        state.nodes = state.nodes.update(node_id.clone(), node);

        let result = apply_node_restore(state.clone(), "n1").unwrap();
        assert!(result.has_node(&node_id));
    }

    #[test]
    fn test_apply_node_restore_missing() {
        let state = DiagramProjection::default();
        let result = apply_node_restore(state, "n1");
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_node_op_dispatch() {
        let mut state = DiagramProjection::default();
        let node_id = NodeId::new("n1".to_string());
        state.nodes = state.nodes.update(node_id.clone(), test_node());

        let op_move = DomainOp::NodeMove {
            id: NodeId::new("n1".to_string()),
            x: 10.0,
            y: 20.0,
        };
        let result_move = apply_node_op(state.clone(), &op_move).unwrap();
        assert_eq!(
            result_move.nodes.get(&node_id).unwrap().x,
            OrderedFloat(10.0)
        );

        let op_del = DomainOp::NodeDelete {
            id: NodeId::new("n1".to_string()),
        };
        let result_del = apply_node_op(state.clone(), &op_del).unwrap();
        assert!(!result_del.has_node(&node_id));

        let op_restore = DomainOp::NodeRestore {
            id: NodeId::new("n1".to_string()),
        };
        let result_restore = apply_node_op(state.clone(), &op_restore).unwrap();
        assert!(result_restore.has_node(&node_id));

        let op_invalid = DomainOp::EdgeDisconnect {
            id: EdgeId::new("e1".to_string()),
        };
        let result_invalid = apply_node_op(state.clone(), &op_invalid);
        assert!(result_invalid.is_err());
    }
}
