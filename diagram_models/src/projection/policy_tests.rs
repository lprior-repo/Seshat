#[cfg(test)]
mod tests {
    use crate::document::{Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat};
    use crate::envelope::DomainOp;
    use crate::projection::policy::{apply_policy_op, enforce_cycle_policy};
    use crate::projection::types::{CyclePolicy, DiagramProjection, ReplayError};
    use im::{HashMap, Vector};

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
            tags: Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn create_test_edge(source: &str, target: &str) -> Edge {
        Edge {
            source: NodeId::new(source.to_string()),
            target: NodeId::new(target.to_string()),
            label: String::new(),
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat(0.5),
            directed: true,
            bend_points: Vector::new(),
            tags: Vector::new(),
            metadata: HashMap::new(),
            color: None,
            thickness: OrderedFloat(1.0),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    fn build_projection(policy: CyclePolicy, edges: Vec<(&str, &str, &str)>) -> DiagramProjection {
        let mut nodes_map = HashMap::new();
        let mut edges_map = HashMap::new();

        // Extract nodes from edges
        for (_, source, target) in &edges {
            if !nodes_map.contains_key(&NodeId::new(source.to_string())) {
                nodes_map.insert(NodeId::new(source.to_string()), create_test_node(source));
            }
            if !nodes_map.contains_key(&NodeId::new(target.to_string())) {
                nodes_map.insert(NodeId::new(target.to_string()), create_test_node(target));
            }
        }

        for (edge_id, source, target) in edges {
            edges_map.insert(
                EdgeId::new(edge_id.to_string()),
                create_test_edge(source, target),
            );
        }

        DiagramProjection {
            version: 0,
            revision: 0,
            nodes: nodes_map,
            edges: edges_map,
            author_priority: HashMap::new(),
            cycle_policy: policy,
        }
    }

    #[test]
    fn given_graph_with_cycle_when_policy_allow_then_enforce_returns_ok() {
        let proj = build_projection(
            CyclePolicy::Allow,
            vec![
                ("e1", "n1", "n2"),
                ("e2", "n2", "n3"),
                ("e3", "n3", "n1"), // Cycle
            ],
        );

        let result = enforce_cycle_policy(&proj);
        assert!(matches!(result, Ok(())));
    }

    #[test]
    fn given_graph_with_cycle_when_policy_deny_then_enforce_returns_error() {
        let proj = build_projection(
            CyclePolicy::Deny,
            vec![
                ("e1", "n1", "n2"),
                ("e2", "n2", "n3"),
                ("e3", "n3", "n1"), // Cycle
            ],
        );

        let result = enforce_cycle_policy(&proj);
        assert!(matches!(result, Err(ReplayError::CycleViolation(_))));
    }

    #[test]
    fn given_acyclic_graph_when_policy_deny_then_enforce_returns_ok() {
        let proj = build_projection(
            CyclePolicy::Deny,
            vec![
                ("e1", "n1", "n2"),
                ("e2", "n2", "n3"),
                ("e3", "n1", "n3"), // Not a cycle, just multiple paths
            ],
        );

        let result = enforce_cycle_policy(&proj);
        assert!(matches!(result, Ok(())));
    }

    #[test]
    fn given_acyclic_state_when_op_creates_cycle_under_deny_then_apply_returns_error() {
        let proj = build_projection(CyclePolicy::Deny, vec![("e1", "n1", "n2")]);

        let op = DomainOp::EdgeConnect {
            id: EdgeId::new("e2".to_string()),
            source: NodeId::new("n2".to_string()),
            target: NodeId::new("n1".to_string()),
        };

        let result = apply_policy_op(proj, &op);
        assert!(matches!(result, Err(ReplayError::CycleViolation(_))));
    }

    #[test]
    fn given_acyclic_state_when_op_creates_cycle_under_allow_then_apply_returns_ok() {
        let proj = build_projection(CyclePolicy::Allow, vec![("e1", "n1", "n2")]);

        let op = DomainOp::EdgeConnect {
            id: EdgeId::new("e2".to_string()),
            source: NodeId::new("n2".to_string()),
            target: NodeId::new("n1".to_string()),
        };

        let result = apply_policy_op(proj, &op);
        assert!(matches!(result, Ok(_)));

        if let Ok(new_state) = result {
            assert!(new_state.edges.contains_key(&EdgeId::new("e2".to_string())));
        }
    }
}
