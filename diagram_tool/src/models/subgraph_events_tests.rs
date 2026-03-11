#[cfg(test)]
mod tests {
    use crate::models::document::{Node, NodeId, NodeKind, OrderedFloat};
    use crate::models::subgraph_events::*;
    use im::HashMap;

    fn create_test_state() -> DiagramState {
        DiagramState {
            version: 1,
            revision: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: crate::models::projection::types::CyclePolicy::Allow,
        }
    }

    fn create_test_node(
        id: &str,
        kind: NodeKind,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        parent: Option<&str>,
    ) -> (NodeId, Node) {
        let node_id = NodeId::new(id.to_string());
        let node = Node {
            kind: kind.clone(),
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat::new_unchecked(x),
            y: OrderedFloat::new_unchecked(y),
            width: OrderedFloat::new_unchecked(w),
            height: OrderedFloat::new_unchecked(h),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: parent.map(|s| NodeId::new(s.to_string())),
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: if kind == NodeKind::Subgraph { 0 } else { 10 },
            style: None,
            collapsed: None,
        };
        (node_id, node)
    }

    #[test]
    fn test_subgraph_bounds_expand_when_child_added() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) = create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, None);

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);

        // Before adding N2
        let bounds = calculate_subgraph_bounds(&s1_id, &state).unwrap();
        // padding is 20.0, min_x=0, max_x=10 => x=-20, width=50
        assert_eq!(bounds.x.0, -20.0);
        assert_eq!(bounds.width.0, 50.0);

        // Action
        add_node_to_subgraph(&n2_id, &s1_id, &mut state).unwrap();

        // After adding N2
        let s1 = state.nodes.get(&s1_id).unwrap();
        // min_x=0, min_y=0, max_x=30, max_y=30
        // with padding 20 => x=-20, width=30-0+40=70
        assert_eq!(s1.x.0, -20.0);
        assert_eq!(s1.width.0, 70.0);
        assert_eq!(s1.height.0, 70.0);
    }

    #[test]
    fn test_subgraph_z_index_orders_children_above_container() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, mut n1) =
            create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, Some("S1"));
        n1.z_index = -5; // Below container initially

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        update_z_index_ordering(&s1_id, &mut state).unwrap();

        let n1_updated = state.nodes.get(&n1_id).unwrap();
        let s1_updated = state.nodes.get(&s1_id).unwrap();
        assert!(n1_updated.z_index > s1_updated.z_index);
    }

    #[test]
    fn test_add_node_updates_parent_reference() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, None);

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        add_node_to_subgraph(&n1_id, &s1_id, &mut state).unwrap();

        let n1_updated = state.nodes.get(&n1_id).unwrap();
        assert_eq!(n1_updated.parent, Some(s1_id));
    }

    #[test]
    fn test_remove_node_clears_parent_reference() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        remove_node_from_subgraph(&n1_id, &mut state).unwrap();

        let n1_updated = state.nodes.get(&n1_id).unwrap();
        assert_eq!(n1_updated.parent, None);
    }

    #[test]
    fn test_batch_add_updates_multiple_nodes_and_bounds_once() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, None);
        let (n2_id, n2) = create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, None);

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);

        batch_add_nodes_to_subgraph(&[n1_id.clone(), n2_id.clone()], &s1_id, &mut state).unwrap();

        let n1_updated = state.nodes.get(&n1_id).unwrap();
        let n2_updated = state.nodes.get(&n2_id).unwrap();
        assert_eq!(n1_updated.parent, Some(s1_id.clone()));
        assert_eq!(n2_updated.parent, Some(s1_id.clone()));

        let s1_updated = state.nodes.get(&s1_id).unwrap();
        assert_eq!(s1_updated.width.0, 70.0);
    }

    #[test]
    fn test_remove_all_nodes_leaves_empty_container() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);

        remove_all_nodes_from_subgraph(&s1_id, &mut state).unwrap();

        let n1_updated = state.nodes.get(&n1_id).unwrap();
        let n2_updated = state.nodes.get(&n2_id).unwrap();
        assert_eq!(n1_updated.parent, None);
        assert_eq!(n2_updated.parent, None);

        let s1_updated = state.nodes.get(&s1_id).unwrap();
        assert_eq!(s1_updated.width.0, 0.0);
        assert_eq!(s1_updated.height.0, 0.0);
    }

    #[test]
    fn test_add_node_returns_error_when_subgraph_not_found() {
        let mut state = create_test_state();
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, None);
        state.nodes.insert(n1_id.clone(), n1);

        let s1_missing = NodeId::new("S1".to_string());
        let result = add_node_to_subgraph(&n1_id, &s1_missing, &mut state);
        assert_eq!(result, Err(Error::NodeNotFound(s1_missing)));
    }

    #[test]
    fn test_add_node_returns_error_when_child_not_found() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        state.nodes.insert(s1_id.clone(), s1);

        let n1_missing = NodeId::new("N1".to_string());
        let result = add_node_to_subgraph(&n1_missing, &s1_id, &mut state);
        assert_eq!(result, Err(Error::NodeNotFound(n1_missing)));
    }

    #[test]
    fn test_add_node_returns_error_on_cycle_detection() {
        let mut state = create_test_state();
        let (s1_id, s1) =
            create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, Some("S2"));
        let (s2_id, s2) = create_test_node("S2", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(s2_id.clone(), s2);

        // S1 is child of S2. If we try to make S2 child of S1, that's a cycle.
        let result = add_node_to_subgraph(&s2_id, &s1_id, &mut state);
        assert_eq!(result, Err(Error::CycleDetected(s2_id, s1_id)));
    }

    #[test]
    fn test_subgraph_bounds_contract_when_outlier_child_removed() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 100.0, 100.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);

        // Before: large bounds due to N2
        let bounds_before = calculate_subgraph_bounds(&s1_id, &state).unwrap();
        assert_eq!(bounds_before.width.0, 150.0); // 110 - 0 + 40

        // Remove N2
        remove_node_from_subgraph(&n2_id, &mut state).unwrap();

        // After: bounds shrink to just N1
        let s1_updated = state.nodes.get(&s1_id).unwrap();
        assert_eq!(s1_updated.width.0, 50.0); // 10 - 0 + 40
    }

    #[test]
    fn test_batch_add_with_empty_list_does_nothing() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        state.nodes.insert(s1_id.clone(), s1);

        batch_add_nodes_to_subgraph(&[], &s1_id, &mut state).unwrap();

        // Bounds shouldn't change to 0 if it was already 10x10, wait, it just does nothing
        let s1_updated = state.nodes.get(&s1_id).unwrap();
        assert_eq!(s1_updated.width.0, 10.0);
    }
}
