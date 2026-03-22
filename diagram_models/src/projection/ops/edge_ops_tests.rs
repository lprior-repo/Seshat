#[cfg(test)]
mod tests {
    use crate::document::{
        ArrowType, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use crate::envelope::DomainOp;
    use crate::projection::ops::edge_ops::{
        apply_edge_connect, apply_edge_connect_checked, apply_edge_disconnect,
        apply_edge_disconnect_checked, apply_edge_op, apply_update_edge_label,
        apply_update_edge_style, create_default_edge, verify_edge_tolerance,
    };
    use crate::projection::types::{DiagramProjection, ReplayError};
    use im::HashMap;

    fn create_test_node(id: &str) -> Node {
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
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn default_projection_with_nodes(node_ids: &[&str]) -> DiagramProjection {
        let mut nodes = HashMap::new();
        for id in node_ids {
            nodes.insert(NodeId::new(id.to_string()), create_test_node(id));
        }
        DiagramProjection {
            version: 0,
            revision: 0,
            nodes,
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: Default::default(),
        }
    }

    #[test]
    fn given_existing_edge_when_update_label_then_label_updated() {
        let mut state = default_projection_with_nodes(&["n1", "n2"]);
        state.edges.insert(
            EdgeId::new("e1".to_string()),
            create_default_edge(NodeId::new("n1".to_string()), NodeId::new("n2".to_string())),
        );

        let new_state = apply_update_edge_label(state, "e1", "new_label").unwrap();
        assert_eq!(
            new_state
                .edges
                .get(&EdgeId::new("e1".to_string()))
                .unwrap()
                .label,
            "new_label"
        );
    }

    #[test]
    fn given_missing_edge_when_update_label_then_returns_error() {
        let state = default_projection_with_nodes(&["n1", "n2"]);
        let result = apply_update_edge_label(state, "missing", "new_label");
        assert!(matches!(result, Err(ReplayError::InvariantViolation(_))));
    }

    #[test]
    fn given_existing_edge_when_update_style_then_style_updated() {
        let mut state = default_projection_with_nodes(&["n1", "n2"]);
        state.edges.insert(
            EdgeId::new("e1".to_string()),
            create_default_edge(NodeId::new("n1".to_string()), NodeId::new("n2".to_string())),
        );

        let new_state = apply_update_edge_style(state, "e1", EdgeStyle::Dashed).unwrap();
        assert_eq!(
            new_state
                .edges
                .get(&EdgeId::new("e1".to_string()))
                .unwrap()
                .style,
            EdgeStyle::Dashed
        );
    }

    #[test]
    fn given_valid_nodes_when_apply_connect_invariant_then_edge_created() {
        let state = default_projection_with_nodes(&["n1", "n2"]);
        let new_state = apply_edge_connect(state, "e1", "n1", "n2").unwrap();
        assert!(new_state.edges.contains_key(&EdgeId::new("e1".to_string())));
    }

    #[test]
    fn given_missing_source_when_apply_connect_invariant_then_returns_invariant_error() {
        let state = default_projection_with_nodes(&["n2"]);
        let result = apply_edge_connect(state, "e1", "missing", "n2");
        assert!(matches!(result, Err(ReplayError::InvariantViolation(_))));
    }

    #[test]
    fn given_duplicate_edge_when_apply_connect_checked_then_returns_duplicate_error() {
        let mut state = default_projection_with_nodes(&["n1", "n2"]);
        state.edges.insert(
            EdgeId::new("e1".to_string()),
            create_default_edge(NodeId::new("n1".to_string()), NodeId::new("n2".to_string())),
        );
        let result = apply_edge_connect_checked(state, "e1", "n1", "n2");
        assert!(matches!(result, Err(ReplayError::DuplicateEdge(_))));
    }

    #[test]
    fn given_missing_target_when_apply_connect_checked_then_returns_policy_error() {
        let state = default_projection_with_nodes(&["n1"]);
        let result = apply_edge_connect_checked(state, "e1", "n1", "missing");
        assert!(matches!(result, Err(ReplayError::PolicyViolation(_))));
    }

    #[test]
    fn given_existing_edge_when_apply_disconnect_invariant_then_edge_removed() {
        let mut state = default_projection_with_nodes(&["n1", "n2"]);
        state.edges.insert(
            EdgeId::new("e1".to_string()),
            create_default_edge(NodeId::new("n1".to_string()), NodeId::new("n2".to_string())),
        );
        let new_state = apply_edge_disconnect(state, "e1").unwrap();
        assert!(!new_state.edges.contains_key(&EdgeId::new("e1".to_string())));
    }

    #[test]
    fn given_missing_edge_when_apply_disconnect_invariant_then_returns_invariant_error() {
        let state = default_projection_with_nodes(&[]);
        let result = apply_edge_disconnect(state, "missing");
        assert!(matches!(result, Err(ReplayError::InvariantViolation(_))));
    }

    #[test]
    fn given_missing_edge_when_apply_disconnect_checked_then_returns_edge_not_found_error() {
        let state = default_projection_with_nodes(&[]);
        let result = apply_edge_disconnect_checked(state, "missing");
        assert!(matches!(result, Err(ReplayError::EdgeNotFound(_))));
    }

    #[test]
    fn given_edge_connect_op_when_apply_edge_op_then_edge_created() {
        let state = default_projection_with_nodes(&["n1", "n2"]);
        let op = DomainOp::EdgeConnect {
            id: EdgeId::new("e1".to_string()),
            source: NodeId::new("n1".to_string()),
            target: NodeId::new("n2".to_string()),
        };
        let new_state = apply_edge_op(state, &op).unwrap();
        assert!(new_state.edges.contains_key(&EdgeId::new("e1".to_string())));
    }

    #[test]
    fn given_edge_disconnect_op_when_apply_edge_op_then_edge_removed() {
        let mut state = default_projection_with_nodes(&["n1", "n2"]);
        state.edges.insert(
            EdgeId::new("e1".to_string()),
            create_default_edge(NodeId::new("n1".to_string()), NodeId::new("n2".to_string())),
        );
        let op = DomainOp::EdgeDisconnect {
            id: EdgeId::new("e1".to_string()),
        };
        let new_state = apply_edge_op(state, &op).unwrap();
        assert!(!new_state.edges.contains_key(&EdgeId::new("e1".to_string())));
    }

    #[test]
    fn given_non_edge_op_when_apply_edge_op_then_returns_invalid_event() {
        let state = default_projection_with_nodes(&[]);
        let op = DomainOp::NodeDelete {
            id: NodeId::new("n1".to_string()),
        };
        let result = apply_edge_op(state, &op);
        assert!(matches!(result, Err(ReplayError::InvalidEvent(_))));
    }

    #[test]
    fn given_valid_state_when_verify_edge_tolerance_then_returns_ok() {
        let mut state = default_projection_with_nodes(&["n1", "n2"]);
        state.edges.insert(
            EdgeId::new("e1".to_string()),
            create_default_edge(NodeId::new("n1".to_string()), NodeId::new("n2".to_string())),
        );
        let result = verify_edge_tolerance(&state);
        assert!(matches!(result, Ok(())));
    }

    #[test]
    fn given_edge_with_missing_source_when_verify_edge_tolerance_then_returns_policy_violation() {
        let mut state = default_projection_with_nodes(&["n2"]);
        state.edges.insert(
            EdgeId::new("e1".to_string()),
            create_default_edge(
                NodeId::new("missing".to_string()),
                NodeId::new("n2".to_string()),
            ),
        );
        let result = verify_edge_tolerance(&state);
        assert!(matches!(result, Err(ReplayError::PolicyViolation(_))));
    }

    #[test]
    fn given_edge_with_nan_thickness_when_verify_edge_tolerance_then_returns_invariant_violation() {
        let mut state = default_projection_with_nodes(&["n1", "n2"]);
        let mut edge =
            create_default_edge(NodeId::new("n1".to_string()), NodeId::new("n2".to_string()));
        edge.thickness = OrderedFloat(f64::NAN);
        state.edges.insert(EdgeId::new("e1".to_string()), edge);

        let result = verify_edge_tolerance(&state);
        assert!(matches!(result, Err(ReplayError::InvariantViolation(_))));
    }

    #[test]
    fn given_edge_with_nan_label_offset_when_verify_edge_tolerance_then_returns_invariant_violation(
    ) {
        let mut state = default_projection_with_nodes(&["n1", "n2"]);
        let mut edge =
            create_default_edge(NodeId::new("n1".to_string()), NodeId::new("n2".to_string()));
        edge.label_offset_t = OrderedFloat(f64::NAN);
        state.edges.insert(EdgeId::new("e1".to_string()), edge);

        let result = verify_edge_tolerance(&state);
        assert!(matches!(result, Err(ReplayError::InvariantViolation(_))));
    }
}
