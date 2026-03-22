#[cfg(test)]
mod tests {
    use crate::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};
    use crate::envelope::DomainOp;
    use crate::projection::ops::z_order::{
        apply_bring_forward, apply_bring_to_front, apply_send_backward, apply_send_to_back,
        apply_z_order_op,
    };
    use crate::projection::types::{DiagramProjection, ReplayError};
    use im::HashMap;

    fn create_test_node(id: &str, z_index: i64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index,
            style: None,
            collapsed: None,
        }
    }

    fn build_projection(nodes: Vec<(&str, i64)>) -> DiagramProjection {
        let mut nodes_map = HashMap::new();
        for (id, z) in nodes {
            nodes_map.insert(NodeId::new(id.to_string()), create_test_node(id, z));
        }

        DiagramProjection {
            version: 0,
            revision: 0,
            nodes: nodes_map,
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: Default::default(),
        }
    }

    #[test]
    fn given_empty_ids_when_z_order_op_then_returns_no_nodes_specified_error() {
        let proj = build_projection(vec![("n1", 0)]);
        let result = apply_bring_forward(proj, &[]);
        assert!(matches!(result, Err(ReplayError::NoNodesSpecified)));
    }

    #[test]
    fn given_invalid_ids_when_z_order_op_then_returns_all_nodes_invalid_error() {
        let proj = build_projection(vec![("n1", 0)]);
        let result = apply_bring_forward(proj, &[NodeId::new("missing".to_string())]);
        assert!(matches!(result, Err(ReplayError::AllNodesInvalid(_))));
    }

    #[test]
    fn given_nodes_when_bring_forward_then_z_index_is_incremented() {
        let proj = build_projection(vec![("n1", 0), ("n2", 1), ("n3", 2)]);
        // Bring n2 forward. Order should become n1 (0), n3 (1), n2 (2)
        let ids = vec![NodeId::new("n2".to_string())];

        let result = apply_bring_forward(proj, &ids).unwrap();

        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .z_index,
            0
        );
        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n3".to_string()))
                .unwrap()
                .z_index,
            1
        );
        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n2".to_string()))
                .unwrap()
                .z_index,
            2
        );
    }

    #[test]
    fn given_nodes_when_send_backward_then_z_index_is_decremented() {
        let proj = build_projection(vec![("n1", 0), ("n2", 1), ("n3", 2)]);
        // Send n2 backward. Order should become n2 (0), n1 (1), n3 (2)
        let ids = vec![NodeId::new("n2".to_string())];

        let result = apply_send_backward(proj, &ids).unwrap();

        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n2".to_string()))
                .unwrap()
                .z_index,
            0
        );
        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .z_index,
            1
        );
        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n3".to_string()))
                .unwrap()
                .z_index,
            2
        );
    }

    #[test]
    fn given_nodes_when_bring_to_front_then_z_index_is_highest() {
        let proj = build_projection(vec![("n1", 0), ("n2", 1), ("n3", 2)]);
        // Bring n1 to front. Order should become n2 (0), n3 (1), n1 (2)
        let ids = vec![NodeId::new("n1".to_string())];

        let result = apply_bring_to_front(proj, &ids).unwrap();

        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n2".to_string()))
                .unwrap()
                .z_index,
            0
        );
        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n3".to_string()))
                .unwrap()
                .z_index,
            1
        );
        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .z_index,
            2
        );
    }

    #[test]
    fn given_nodes_when_send_to_back_then_z_index_is_lowest() {
        let proj = build_projection(vec![("n1", 0), ("n2", 1), ("n3", 2)]);
        // Send n3 to back. Order should become n3 (0), n1 (1), n2 (2)
        let ids = vec![NodeId::new("n3".to_string())];

        let result = apply_send_to_back(proj, &ids).unwrap();

        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n3".to_string()))
                .unwrap()
                .z_index,
            0
        );
        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .z_index,
            1
        );
        assert_eq!(
            result
                .nodes
                .get(&NodeId::new("n2".to_string()))
                .unwrap()
                .z_index,
            2
        );
    }

    #[test]
    fn given_wrong_op_when_apply_z_order_op_then_returns_error() {
        let proj = build_projection(vec![("n1", 0)]);
        let op = DomainOp::NodeDelete {
            id: NodeId::new("n1".to_string()),
        };

        let result = apply_z_order_op(proj, &op);
        assert!(matches!(result, Err(ReplayError::InvalidEvent(_))));
    }
}
