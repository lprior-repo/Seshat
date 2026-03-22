#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
#[cfg(test)]
mod tests {
    use crate::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};
    use crate::projection::ops::node_bounds::propagate_bounds_to_ancestors;
    use crate::subgraph_events::*;
    use im::HashMap;

    fn create_test_state() -> DiagramState {
        DiagramState {
            version: 1,
            revision: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: crate::projection::types::CyclePolicy::Allow,
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
            kind,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat::new_unchecked(x),
            y: OrderedFloat::new_unchecked(y),
            width: OrderedFloat::new_unchecked(w),
            height: OrderedFloat::new_unchecked(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
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
        state.nodes.insert(n1_id, n1);
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

        state.nodes.insert(s1_id, s1);
        state.nodes.insert(n1_id.clone(), n1);

        remove_node_from_subgraph(&n1_id, &mut state).unwrap();

        let n1_updated = state.nodes.get(&n1_id).unwrap();
        assert_eq!(n1_updated.parent, None);
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
        state.nodes.insert(n1_id, n1);
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

    // ========================================================================
    // HAPPY PATH TESTS (BDD Format)
    // ========================================================================

    #[test]
    fn given_subgraph_with_two_children_when_calculate_bounds_then_returns_expanded_rect() {
        let state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));

        let mut state = state;
        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id, n1);
        state.nodes.insert(n2_id, n2);

        let bounds = calculate_subgraph_bounds(&s1_id, &state).unwrap();

        // min_x=0, max_x=30, with padding=20: x=-20, width=30+40=70
        assert_eq!(bounds.x.0, -20.0);
        assert_eq!(bounds.y.0, -20.0);
        assert_eq!(bounds.width.0, 70.0);
        assert_eq!(bounds.height.0, 70.0);
    }

    #[test]
    fn given_subgraph_with_child_when_add_node_outside_bounds_then_bounds_expand() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) = create_test_node("N2", NodeKind::Node, 100.0, 100.0, 10.0, 10.0, None);

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id, n1);
        state.nodes.insert(n2_id.clone(), n2);

        add_node_to_subgraph(&n2_id, &s1_id, &mut state).unwrap();

        let s1 = state.nodes.get(&s1_id).unwrap();
        // N1 at (10,10) size 10, N2 at (100,100) size 10
        // min_x=10, max_x=110 => x = 10-20=-10, width=110-10+40=140
        assert_eq!(s1.x.0, -10.0);
        assert_eq!(s1.y.0, -10.0);
        assert_eq!(s1.width.0, 140.0);
    }

    #[test]
    fn given_subgraph_with_child_when_add_node_outside_bounds_then_parent_updated() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) = create_test_node("N2", NodeKind::Node, 100.0, 100.0, 10.0, 10.0, None);

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id, n1);
        state.nodes.insert(n2_id.clone(), n2);

        add_node_to_subgraph(&n2_id, &s1_id, &mut state).unwrap();

        let n2 = state.nodes.get(&n2_id).unwrap();
        assert_eq!(n2.parent, Some(s1_id));
    }

    #[test]
    fn given_subgraph_with_two_children_at_positions_0_and_100_when_remove_outlier_then_bounds_contract(
    ) {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 100.0, 100.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id, n1);
        state.nodes.insert(n2_id.clone(), n2);

        remove_node_from_subgraph(&n2_id, &mut state).unwrap();

        let s1 = state.nodes.get(&s1_id).unwrap();
        // After removing N2, only N1 remains at (0,0) with size (10,10)
        // min_x=0, max_x=10 => x=-20, width=10-0+40=50
        assert_eq!(s1.x.0, -20.0);
        assert_eq!(s1.y.0, -20.0);
        assert_eq!(s1.width.0, 50.0);
        assert_eq!(s1.height.0, 50.0);
    }

    #[test]
    fn given_subgraph_with_two_children_at_positions_0_and_100_when_remove_outlier_then_parent_cleared(
    ) {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 100.0, 100.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id, s1);
        state.nodes.insert(n1_id, n1);
        state.nodes.insert(n2_id.clone(), n2);

        remove_node_from_subgraph(&n2_id, &mut state).unwrap();

        let n2 = state.nodes.get(&n2_id).unwrap();
        assert_eq!(n2.parent, None);
    }

    #[test]
    fn given_subgraph_with_child_below_container_z_index_when_update_z_index_then_child_above() {
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

    // ========================================================================
    // ERROR PATH TESTS (BDD Format)
    // ========================================================================

    #[test]
    fn given_empty_state_when_calculate_bounds_for_missing_subgraph_then_returns_node_not_found() {
        let state = create_test_state();
        let missing_subgraph_id = NodeId::new("missing_s1".to_string());

        let result = calculate_subgraph_bounds(&missing_subgraph_id, &state);

        assert!(matches!(result, Err(Error::NodeNotFound(id)) if id.as_str() == "missing_s1"));
    }

    #[test]
    fn given_state_with_only_child_node_when_add_to_missing_subgraph_then_returns_node_not_found() {
        let mut state = create_test_state();
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, None);
        state.nodes.insert(n1_id.clone(), n1);

        let missing_subgraph_id = NodeId::new("missing_s1".to_string());
        let result = add_node_to_subgraph(&n1_id, &missing_subgraph_id, &mut state);

        assert!(matches!(result, Err(Error::NodeNotFound(id)) if id.as_str() == "missing_s1"));
    }

    #[test]
    fn given_state_with_only_subgraph_when_add_missing_child_then_returns_node_not_found() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        state.nodes.insert(s1_id.clone(), s1);

        let missing_child_id = NodeId::new("missing_n1".to_string());
        let result = add_node_to_subgraph(&missing_child_id, &s1_id, &mut state);

        assert!(matches!(result, Err(Error::NodeNotFound(id)) if id.as_str() == "missing_n1"));
    }

    #[test]
    fn given_nested_subgraphs_s1_child_of_s2_when_add_s2_as_child_of_s1_then_returns_cycle_detected(
    ) {
        let mut state = create_test_state();
        // S1 is child of S2
        let (s1_id, s1) =
            create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, Some("S2"));
        let (s2_id, s2) = create_test_node("S2", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(s2_id.clone(), s2);

        // Try to make S2 child of S1 - that would create a cycle
        let result = add_node_to_subgraph(&s2_id, &s1_id, &mut state);

        assert!(matches!(result, Err(Error::CycleDetected(_, _))));
    }

    #[test]
    fn given_valid_rect_when_create_with_negative_width_then_returns_invalid_bounds() {
        let result = Rect::new(0.0, 0.0, -10.0, 10.0);

        assert!(matches!(result, Err(Error::InvalidBounds(_))));
    }

    #[test]
    fn given_valid_rect_when_create_with_negative_height_then_returns_invalid_bounds() {
        let result = Rect::new(0.0, 0.0, 10.0, -10.0);

        assert!(matches!(result, Err(Error::InvalidBounds(_))));
    }

    // ========================================================================
    // EDGE CASE TESTS (BDD Format)
    // ========================================================================

    #[test]
    fn given_subgraph_with_single_child_when_calculate_bounds_then_returns_child_plus_padding() {
        let state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 50.0, 50.0, 10.0, 10.0, Some("S1"));

        let mut state = state;
        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id, n1);

        let bounds = calculate_subgraph_bounds(&s1_id, &state).unwrap();

        // Single child at (50,50) size (10,10)
        // min_x=50, max_x=60 => x=50-20=30, width=60-50+40=50
        assert_eq!(bounds.x.0, 30.0);
        assert_eq!(bounds.y.0, 30.0);
        assert_eq!(bounds.width.0, 50.0);
        assert_eq!(bounds.height.0, 50.0);
    }

    #[test]
    fn given_empty_subgraph_with_no_children_when_calculate_bounds_then_returns_zero_rect() {
        let state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);

        let mut state = state;
        state.nodes.insert(s1_id.clone(), s1);

        let bounds = calculate_subgraph_bounds(&s1_id, &state).unwrap();

        assert_eq!(bounds.x.0, 0.0);
        assert_eq!(bounds.y.0, 0.0);
        assert_eq!(bounds.width.0, 0.0);
        assert_eq!(bounds.height.0, 0.0);
    }

    #[test]
    fn given_node_not_in_any_subgraph_when_remove_from_subgraph_then_returns_ok() {
        let mut state = create_test_state();
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, None);

        state.nodes.insert(n1_id.clone(), n1);

        let result = remove_node_from_subgraph(&n1_id, &mut state);

        assert!(result.is_ok());
    }

    #[test]
    fn given_multiple_nodes_when_batch_add_then_bounds_calculated_once() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 0.0, 0.0, 10.0, 10.0, None);
        let (n2_id, n2) = create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, None);
        let (n3_id, n3) = create_test_node("N3", NodeKind::Node, 40.0, 40.0, 10.0, 10.0, None);

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);
        state.nodes.insert(n3_id.clone(), n3);

        batch_add_nodes_to_subgraph(&[n1_id, n2_id, n3_id], &s1_id, &mut state).unwrap();

        let s1 = state.nodes.get(&s1_id).unwrap();
        // All three children: N1 at 0-10, N2 at 20-30, N3 at 40-50
        // min_x=0, max_x=50 => x=-20, width=50+40=90
        assert_eq!(s1.x.0, -20.0);
        assert_eq!(s1.width.0, 90.0);
    }

    #[test]
    fn given_subgraph_with_all_children_removed_when_remove_last_child_then_empty_bounds() {
        let mut state = create_test_state();
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        remove_node_from_subgraph(&n1_id, &mut state).unwrap();

        let s1 = state.nodes.get(&s1_id).unwrap();
        assert_eq!(s1.width.0, 0.0);
        assert_eq!(s1.height.0, 0.0);
    }

    // ========================================================================
    // INTEGRATION TESTS (Full Workflow)
    // ========================================================================

    #[test]
    fn integration_full_workflow_subgraph_auto_resize() {
        // Given: RootSubgraph containing Child1 (which is a subgraph) containing Grandchild
        let mut state = create_test_state();

        // Root subgraph at (0,0)
        let (root_id, root) =
            create_test_node("Root", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);

        // Child1 is a subgraph at (10,10)
        let (child1_id, child1) = create_test_node(
            "Child1",
            NodeKind::Subgraph,
            10.0,
            10.0,
            30.0,
            30.0,
            Some("Root"),
        );

        // Grandchild at (20,20)
        let (grandchild_id, grandchild) = create_test_node(
            "Grandchild",
            NodeKind::Node,
            20.0,
            20.0,
            10.0,
            10.0,
            Some("Child1"),
        );

        state.nodes.insert(root_id.clone(), root);
        state.nodes.insert(child1_id.clone(), child1);
        state.nodes.insert(grandchild_id.clone(), grandchild);

        // Step 1: Add new node "NewNode" at (500, 500) to Child1
        let (new_node_id, new_node) =
            create_test_node("NewNode", NodeKind::Node, 500.0, 500.0, 10.0, 10.0, None);
        state.nodes.insert(new_node_id.clone(), new_node);

        add_node_to_subgraph(&new_node_id, &child1_id, &mut state).unwrap();

        // Step 2 & 3: Propagate bounds to ancestors
        state.nodes = propagate_bounds_to_ancestors(state.nodes.clone(), &child1_id);
        state.nodes = propagate_bounds_to_ancestors(state.nodes.clone(), &root_id);

        // Verify Child1 bounds expanded - extract values before further mutation
        let child1_width_before = state.nodes.get(&child1_id).map(|n| n.width.0);
        assert!(child1_width_before.is_some());
        assert!(child1_width_before.unwrap() > 30.0); // Expanded from 30

        // Verify Root bounds expanded
        let root_width_before = state.nodes.get(&root_id).map(|n| n.width.0);
        assert!(root_width_before.is_some());
        assert!(root_width_before.unwrap() > 50.0); // Expanded from 50

        // Step 4: Remove Grandchild from Child1
        remove_node_from_subgraph(&grandchild_id, &mut state).unwrap();

        // Step 5: Propagate bounds again
        state.nodes = propagate_bounds_to_ancestors(state.nodes.clone(), &child1_id);

        // Verify bounds after removal - use the saved width for comparison
        let child1_after_width = state.nodes.get(&child1_id).map(|n| n.width.0);
        // Only NewNode remains at 500,500 with size 10
        // min_x=500, max_x=510 => x=480, width=30+40=70
        assert!(child1_after_width.is_some());
        assert!(child1_after_width.unwrap() < child1_width_before.unwrap()); // Contracted
    }

    #[test]
    fn integration_subgraph_lifecycle_create_read_update_delete() {
        let mut state = create_test_state();

        // 1. Create subgraph S1 at (0,0)
        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 10.0, 10.0, None);
        state.nodes.insert(s1_id.clone(), s1);

        // 2. Add child N1 at (10,10) to S1
        let (n1_id, n1) = create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, None);
        state.nodes.insert(n1_id.clone(), n1);
        add_node_to_subgraph(&n1_id, &s1_id, &mut state).unwrap();

        // 3. Read S1 bounds (verify contains N1 + padding)
        let bounds1 = calculate_subgraph_bounds(&s1_id, &state).unwrap();
        // N1 at 10-20, padding 20 => x=10-20=-10, width=20-10+40=50
        assert_eq!(bounds1.x.0, -10.0);
        assert_eq!(bounds1.width.0, 50.0);

        // 4. Add child N2 at (100,100) to S1
        let (n2_id, n2) = create_test_node("N2", NodeKind::Node, 100.0, 100.0, 10.0, 10.0, None);
        state.nodes.insert(n2_id.clone(), n2);
        add_node_to_subgraph(&n2_id, &s1_id, &mut state).unwrap();

        // 5. Read S1 bounds (verify contains both N1, N2 + padding)
        let bounds2 = calculate_subgraph_bounds(&s1_id, &state).unwrap();
        // min_x=10, max_x=110 => x=10-20=-10, width=110-10+40=140
        assert_eq!(bounds2.x.0, -10.0);
        assert_eq!(bounds2.width.0, 140.0);
        assert!(bounds2.width.0 > bounds1.width.0);

        // 6. Remove N1 from S1
        remove_node_from_subgraph(&n1_id, &mut state).unwrap();

        // 7. Read S1 bounds (verify contains only N2 + padding)
        let bounds3 = calculate_subgraph_bounds(&s1_id, &state).unwrap();
        // N2 at 100-110, padding 20 => x=100-20=80, width=110-100+40=50
        assert_eq!(bounds3.x.0, 80.0);
        assert_eq!(bounds3.width.0, 50.0);
        assert!(bounds3.width.0 < bounds2.width.0);

        // 8. Delete N2 from S1
        remove_node_from_subgraph(&n2_id, &mut state).unwrap();

        // 9. Read S1 bounds (verify empty: 0,0,0,0)
        let bounds4 = calculate_subgraph_bounds(&s1_id, &state).unwrap();
        assert_eq!(bounds4.width.0, 0.0);
        assert_eq!(bounds4.height.0, 0.0);
    }
}
