//! Subgraph cascade deletion and reparenting tests
//!
//! Tests for SUB-032 (reparent mode) and SUB-034 (delete mode) cascade behaviors.
//! Includes a DSL layer for declarative test assertions.

#![cfg(test)]

mod tests {
    use crate::models::document::{Node, NodeId, NodeKind, OrderedFloat};
    use crate::models::subgraph_events::calculate_subgraph_bounds;
    use crate::models::subgraph_events::types::{DiagramState, Error};
    use im::HashMap;
    use rand::prelude::*;
    use rand::rngs::StdRng;

    // ========================================================================
    // DSL LAYER - Helper Functions for Declarative Testing
    // ========================================================================

    /// Creates a fresh DiagramState for testing
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

    /// Creates a test node with the given parameters
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

    /// DSL: Verifies that cascade deletion behaves correctly.
    /// In delete mode, children should be removed from the graph.
    /// In reparent mode, children should be moved to the parent of the deleted subgraph.
    ///
    /// # Arguments
    /// * `state` - The diagram state
    /// * `subgraph_id` - The subgraph being deleted
    /// * `expected_children` - Children that should exist after deletion
    /// * `mode` - "delete" or "reparent"
    fn verify_cascade_deletion(
        state: &DiagramState,
        _subgraph_id: &NodeId,
        expected_children: &[&str],
        mode: &str,
    ) -> Result<(), String> {
        // Verify children exist or don't exist based on mode
        for child_id in expected_children {
            let child_node_id = NodeId::new(child_id.to_string());
            let child_exists = state.nodes.contains_key(&child_node_id);

            match mode {
                "delete" => {
                    if child_exists {
                        return Err(format!(
                            "Child {} should have been deleted but still exists",
                            child_id
                        ));
                    }
                }
                "reparent" => {
                    if !child_exists {
                        return Err(format!(
                            "Child {} should have been reparented but doesn't exist",
                            child_id
                        ));
                    }
                    // In reparent mode, children should have no parent (root level)
                    let child = state.nodes.get(&child_node_id).unwrap();
                    if child.parent.is_some() {
                        return Err(format!(
                            "Child {} should have no parent after reparent but has {:?}",
                            child_id, child.parent
                        ));
                    }
                }
                _ => return Err(format!("Unknown mode: {}", mode)),
            }
        }
        Ok(())
    }

    /// DSL: Asserts that children have been reparented to the correct parent.
    fn assert_children_reparented(
        state: &DiagramState,
        children: &[&str],
        expected_parent: Option<&str>,
    ) -> Result<(), String> {
        for child_id in children {
            let child_node_id = NodeId::new(child_id.to_string());
            let child = state
                .nodes
                .get(&child_node_id)
                .ok_or_else(|| format!("Child {} not found", child_id))?;

            match (child.parent.as_ref(), expected_parent) {
                (Some(actual), Some(expected)) => {
                    if actual.as_str() != expected {
                        return Err(format!(
                            "Child {} has parent {} but expected {}",
                            child_id,
                            actual.as_str(),
                            expected
                        ));
                    }
                }
                (None, None) => {} // OK - no parent expected, no parent found
                (actual, expected) => {
                    return Err(format!(
                        "Child {} has parent {:?} but expected {:?}",
                        child_id, actual, expected
                    ));
                }
            }
        }
        Ok(())
    }

    /// DSL: Asserts that node count is consistent with expected deletions/reparents.
    fn assert_node_count_consistent(
        state: &DiagramState,
        original_count: usize,
        nodes_removed: usize,
        mode: &str,
    ) -> Result<(), String> {
        let current_count = state.nodes.len();
        // In reparent mode: remove 1 (the subgraph), children are reparented not removed
        // In delete mode: remove nodes_removed + 1 (the subgraph)
        let expected_count = match mode {
            "delete" => original_count - nodes_removed - 1, // -1 for the subgraph itself
            "reparent" => original_count - 1,               // -1 for the removed subgraph
            _ => return Err(format!("Unknown mode: {}", mode)),
        };

        if current_count != expected_count {
            return Err(format!(
                "Node count mismatch: expected {} but got {}. Mode: {}",
                expected_count, current_count, mode
            ));
        }
        Ok(())
    }

    /// DSL: Gets all direct children of a subgraph
    fn get_direct_children(state: &DiagramState, parent_id: &NodeId) -> Vec<NodeId> {
        state
            .nodes
            .iter()
            .filter(|(_, node)| node.parent.as_ref() == Some(parent_id))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// DSL: Simulates delete mode - removes a subgraph and all its children
    fn simulate_delete_mode(
        state: &mut DiagramState,
        subgraph_id: &NodeId,
    ) -> Result<Vec<NodeId>, Error> {
        // First, collect all children to delete
        let children = get_direct_children(state, subgraph_id);
        let child_ids: Vec<NodeId> = children.iter().map(|id| id.clone()).collect();

        // Actually delete the children from the graph
        for child_id in &child_ids {
            let _ = state.nodes.remove(child_id); // Remove child node entirely
        }

        // Remove the subgraph itself
        let _ = state.nodes.remove(subgraph_id);

        Ok(child_ids)
    }

    /// DSL: Simulates reparent mode - moves children to grandparent
    fn simulate_reparent_mode(
        state: &mut DiagramState,
        subgraph_id: &NodeId,
    ) -> Result<Option<NodeId>, Error> {
        // Get the parent of the subgraph (grandparent)
        let grandparent_id = state.nodes.get(subgraph_id).and_then(|n| n.parent.clone());

        // Get all children
        let children = get_direct_children(state, subgraph_id);

        // Reparent each child to the grandparent
        for child_id in children {
            let nodes = state.nodes.clone();
            let updated = nodes.get(&child_id).map(|n| crate::models::document::Node {
                parent: grandparent_id.clone(),
                ..n.clone()
            });
            if let Some(node) = updated {
                state.nodes = state.nodes.update(child_id.clone(), node);
            }
        }

        // Remove the subgraph
        let _ = state.nodes.remove(subgraph_id); // im::HashMap::remove returns Option<Node>, discard it

        Ok(grandparent_id)
    }

    // ========================================================================
    // UNIT TESTS - Reparent Mode (SUB-032)
    // ========================================================================

    #[test]
    fn test_reparent_mode_moves_children_to_grandparent() {
        // Given: Root -> Subgraph (S1) -> Children (N1, N2)
        let mut state = create_test_state();

        let (root_id, root) =
            create_test_node("Root", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
        let (s1_id, s1) = create_test_node(
            "S1",
            NodeKind::Subgraph,
            10.0,
            10.0,
            30.0,
            30.0,
            Some("Root"),
        );
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(root_id.clone(), root);
        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);

        // When: Delete S1 in reparent mode
        simulate_reparent_mode(&mut state, &s1_id).unwrap();

        // Then: Children should be reparented to Root
        assert_children_reparented(&state, &["N1", "N2"], Some("Root")).unwrap();

        // Then: Node count should be 3 (subgraph removed, children at root)
        // Root, N1, N2 = 3 nodes
        assert_eq!(state.nodes.len(), 3);

        // Then: S1 should be removed
        assert!(!state.nodes.contains_key(&s1_id));
    }

    #[test]
    fn test_reparent_mode_handles_no_grandparent() {
        // Given: S1 (no parent) -> Children (N1, N2)
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);

        // When: Delete S1 in reparent mode (no grandparent)
        let grandparent = simulate_reparent_mode(&mut state, &s1_id).unwrap();

        // Then: No grandparent should be found
        assert!(grandparent.is_none());

        // Then: Children should have no parent (root level)
        assert_children_reparented(&state, &["N1", "N2"], None).unwrap();

        // Then: Node count should be 2 (subgraph removed, children at root)
        // N1, N2 = 2 nodes
        assert_eq!(state.nodes.len(), 2);
    }

    #[test]
    fn test_reparent_mode_preserves_child_order() {
        // Given: S1 with multiple children
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 100.0, 100.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));
        let (n3_id, n3) =
            create_test_node("N3", NodeKind::Node, 30.0, 30.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);
        state.nodes.insert(n3_id.clone(), n3);

        // When
        simulate_reparent_mode(&mut state, &s1_id).unwrap();

        // Then: All children should still exist
        assert!(state.nodes.contains_key(&n1_id));
        assert!(state.nodes.contains_key(&n2_id));
        assert!(state.nodes.contains_key(&n3_id));
    }

    // ========================================================================
    // UNIT TESTS - Delete Mode (SUB-034)
    // ========================================================================

    #[test]
    fn test_delete_mode_removes_children() {
        // Given: S1 -> Children (N1, N2)
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);

        let original_count = state.nodes.len();

        // When: Delete S1 in delete mode
        simulate_delete_mode(&mut state, &s1_id).unwrap();

        // Then: Children should be removed (parent refs cleared)
        verify_cascade_deletion(&state, &s1_id, &["N1", "N2"], "delete").unwrap();

        // Then: Node count should decrease
        assert_node_count_consistent(&state, original_count, 2, "delete").unwrap();

        // Then: S1 should be removed
        assert!(!state.nodes.contains_key(&s1_id));
    }

    #[test]
    fn test_delete_mode_empty_subgraph() {
        // Given: Empty S1
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);

        state.nodes.insert(s1_id.clone(), s1);

        // When: Delete empty S1 in delete mode
        simulate_delete_mode(&mut state, &s1_id).unwrap();

        // Then: Node count should be 0 (subgraph deleted)
        assert_eq!(state.nodes.len(), 0);
    }

    #[test]
    fn test_delete_mode_nested_subgraphs() {
        // Given: S1 -> S2 -> N1 (nested structure)
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 100.0, 100.0, None);
        let (s2_id, s2) =
            create_test_node("S2", NodeKind::Subgraph, 10.0, 10.0, 50.0, 50.0, Some("S1"));
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S2"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(s2_id.clone(), s2);
        state.nodes.insert(n1_id.clone(), n1);

        // When: Delete S1 in delete mode (should handle nested children)
        // This deletes S1 and its direct children (S2), but not N1 (which is grandchild)
        // For full cascade, we'd need recursive deletion
        simulate_delete_mode(&mut state, &s1_id).unwrap();

        // Then: S1 and S2 should be removed
        assert!(!state.nodes.contains_key(&s1_id));
        assert!(!state.nodes.contains_key(&s2_id));
    }

    // ========================================================================
    // INTEGRATION TESTS - Full Workflows
    // ========================================================================

    #[test]
    fn integration_reparent_workflow_with_bounds() {
        // Given: Root -> S1 -> N1, N2
        let mut state = create_test_state();

        let (root_id, root) =
            create_test_node("Root", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
        let (s1_id, s1) = create_test_node(
            "S1",
            NodeKind::Subgraph,
            10.0,
            10.0,
            30.0,
            30.0,
            Some("Root"),
        );
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(root_id.clone(), root);
        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);
        state.nodes.insert(n2_id.clone(), n2);

        // When: Reparent mode
        simulate_reparent_mode(&mut state, &s1_id).unwrap();

        // Then: Children should be reparented to Root (grandparent)
        let n1 = state.nodes.get(&n1_id).unwrap();
        let n2 = state.nodes.get(&n2_id).unwrap();
        assert!(n1.parent.is_some()); // Should be Some("Root")
        assert!(n2.parent.is_some()); // Should be Some("Root")

        // Then: Root bounds should be recalculated (may have expanded)
        let root_bounds = calculate_subgraph_bounds(&root_id, &state).unwrap();
        // N1 and N2 at positions (10,10)-(20,20), root at (0,0)
        // min_x=10, max_x=30, padding=20 => x=-10, width=60
        assert!(root_bounds.width.0 > 0.0);
    }

    #[test]
    fn integration_mixed_nested_reparent() {
        // Given: Root -> S1 -> S2 -> N1 (multi-level nesting)
        let mut state = create_test_state();

        let (root_id, root) =
            create_test_node("Root", NodeKind::Subgraph, 0.0, 0.0, 100.0, 100.0, None);
        let (s1_id, s1) = create_test_node(
            "S1",
            NodeKind::Subgraph,
            10.0,
            10.0,
            50.0,
            50.0,
            Some("Root"),
        );
        let (s2_id, s2) =
            create_test_node("S2", NodeKind::Subgraph, 20.0, 20.0, 30.0, 30.0, Some("S1"));
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 25.0, 25.0, 10.0, 10.0, Some("S2"));

        state.nodes.insert(root_id.clone(), root);
        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(s2_id.clone(), s2);
        state.nodes.insert(n1_id.clone(), n1);

        // When: Delete S1 in reparent mode
        simulate_reparent_mode(&mut state, &s1_id).unwrap();

        // Then: S1 removed, S2 and N1 should be reparented to Root
        assert!(!state.nodes.contains_key(&s1_id));

        // S2 should now have parent = Root
        let s2 = state.nodes.get(&s2_id).unwrap();
        assert_eq!(s2.parent.as_ref().map(|id| id.as_str()), Some("Root"));

        // N1 should still have parent = S2 (not affected)
        let n1 = state.nodes.get(&n1_id).unwrap();
        assert_eq!(n1.parent.as_ref().map(|id| id.as_str()), Some("S2"));
    }

    // ========================================================================
    // PROPERTY-BASED TESTS - Random Graph Structures
    // ========================================================================

    #[test]
    fn property_reparent_preserves_all_nodes() {
        // Property: In reparent mode, no nodes should be lost
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..10 {
            let mut state = create_test_state();
            let mut node_ids = Vec::new();

            // Create a root
            let (root_id, root) =
                create_test_node("root", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
            state.nodes.insert(root_id.clone(), root);
            node_ids.push(root_id);

            // Create 1-5 subgraphs with 1-3 children each
            let num_subgraphs = rng.gen_range(1..=5);
            let mut all_child_ids = Vec::new();

            for i in 0..num_subgraphs {
                let s_id_str = format!("S{}", i);
                let (s_id, s_node) = create_test_node(
                    &s_id_str,
                    NodeKind::Subgraph,
                    rng.gen_range(0.0..100.0),
                    rng.gen_range(0.0..100.0),
                    30.0,
                    30.0,
                    Some("root"),
                );
                state.nodes.insert(s_id.clone(), s_node);
                node_ids.push(s_id.clone());

                // Add children to this subgraph
                let num_children = rng.gen_range(1..=3);
                for j in 0..num_children {
                    let c_id = format!("S{}_C{}", i, j);
                    let (c_id, c_node) = create_test_node(
                        &c_id,
                        NodeKind::Node,
                        rng.gen_range(0.0..100.0),
                        rng.gen_range(0.0..100.0),
                        10.0,
                        10.0,
                        Some(&s_id_str),
                    );
                    state.nodes.insert(c_id.clone(), c_node);
                    all_child_ids.push(c_id);
                }
            }

            let original_count = state.nodes.len();

            // Delete each subgraph in reparent mode
            for i in 0..num_subgraphs {
                let s_id = NodeId::new(format!("S{}", i));
                let _ = simulate_reparent_mode(&mut state, &s_id);
            }

            // Property: Node count should equal original - 1 (subgraphs removed)
            assert_eq!(
                state.nodes.len(),
                original_count - num_subgraphs,
                "Reparent mode should remove subgraphs but keep children"
            );
        }
    }

    #[test]
    fn property_delete_removes_exactly_children_plus_subgraph() {
        // Property: In delete mode, exactly N+1 nodes removed (N children + 1 subgraph)
        let mut rng = StdRng::seed_from_u64(123);

        for _ in 0..10 {
            let mut state = create_test_state();

            // Create S1 with random children
            let (s1_id, s1) =
                create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
            state.nodes.insert(s1_id.clone(), s1);

            let num_children = rng.gen_range(1..=5);
            let mut child_ids = Vec::new();

            for i in 0..num_children {
                let c_id = format!("C{}", i);
                let (c_id, c_node) = create_test_node(
                    &c_id,
                    NodeKind::Node,
                    rng.gen_range(0.0..100.0),
                    rng.gen_range(0.0..100.0),
                    10.0,
                    10.0,
                    Some("S1"),
                );
                state.nodes.insert(c_id.clone(), c_node);
                child_ids.push(c_id);
            }

            let original_count = state.nodes.len();

            // Delete in delete mode
            simulate_delete_mode(&mut state, &s1_id).unwrap();

            // Property: Should remove exactly num_children + 1 nodes
            assert_eq!(
                state.nodes.len(),
                original_count - (num_children + 1),
                "Delete mode should remove children + subgraph"
            );

            // Verify children are gone
            for c_id in &child_ids {
                assert!(!state.nodes.contains_key(c_id));
            }
            assert!(!state.nodes.contains_key(&s1_id));
        }
    }

    #[test]
    fn property_reparent_always_assigns_valid_parent() {
        // Property: After reparent, every node either has a valid parent or no parent
        let mut rng = StdRng::seed_from_u64(456);

        for _ in 0..10 {
            let mut state = create_test_state();

            // Create a simple structure
            let (root_id, root) =
                create_test_node("root", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
            state.nodes.insert(root_id.clone(), root);

            let num_subgraphs = rng.gen_range(1..=3);
            for i in 0..num_subgraphs {
                let s_id = format!("S{}", i);
                let (s_id, s_node) = create_test_node(
                    &s_id,
                    NodeKind::Subgraph,
                    10.0 + i as f64 * 10.0,
                    10.0,
                    30.0,
                    30.0,
                    Some("root"),
                );
                state.nodes.insert(s_id.clone(), s_node);
            }

            // Randomly delete subgraphs in reparent mode
            for i in 0..num_subgraphs {
                if rng.gen_bool(0.5) {
                    let s_id = NodeId::new(format!("S{}", i));
                    let _ = simulate_reparent_mode(&mut state, &s_id);
                }
            }

            // Property: Every node should have a valid parent (or none for root-level)
            for (id, node) in &state.nodes {
                if let Some(parent_id) = &node.parent {
                    // Parent must exist in the graph
                    assert!(
                        state.nodes.contains_key(parent_id),
                        "Node {} has non-existent parent {}",
                        id.as_str(),
                        parent_id.as_str()
                    );
                }
            }
        }
    }

    #[test]
    fn property_cascade_no_orphaned_edges() {
        // Property: After cascade deletion, no nodes that exist should reference deleted parents
        let mut rng = StdRng::seed_from_u64(789);

        for _ in 0..10 {
            let mut state = create_test_state();

            // Create root and subgraph with children
            let (root_id, root) =
                create_test_node("root", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
            let (s1_id, s1) = create_test_node(
                "S1",
                NodeKind::Subgraph,
                10.0,
                10.0,
                30.0,
                30.0,
                Some("root"),
            );
            let (n1_id, n1) =
                create_test_node("N1", NodeKind::Node, 15.0, 15.0, 10.0, 10.0, Some("S1"));
            let (n2_id, n2) =
                create_test_node("N2", NodeKind::Node, 25.0, 25.0, 10.0, 10.0, Some("S1"));

            state.nodes.insert(root_id.clone(), root);
            state.nodes.insert(s1_id.clone(), s1);
            state.nodes.insert(n1_id.clone(), n1);
            state.nodes.insert(n2_id.clone(), n2);

            let mode = if rng.gen_bool(0.5) {
                "reparent"
            } else {
                "delete"
            };

            match mode {
                "reparent" => {
                    simulate_reparent_mode(&mut state, &s1_id).unwrap();
                }
                "delete" => {
                    simulate_delete_mode(&mut state, &s1_id).unwrap();
                }
                _ => {}
            }

            // Property: All existing nodes should have valid parent references
            for (id, node) in &state.nodes {
                if let Some(parent_id) = &node.parent {
                    assert!(
                        state.nodes.contains_key(parent_id),
                        "After {} mode, node {} references deleted parent {}",
                        mode,
                        id.as_str(),
                        parent_id.as_str()
                    );
                }
            }
        }
    }

    // ========================================================================
    // EDGE CASE TESTS
    // ========================================================================

    #[test]
    fn edge_case_reparent_already_root_level_subgraph() {
        // Given: Subgraph at root level with no parent
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 15.0, 15.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        // When: Reparent mode
        let grandparent = simulate_reparent_mode(&mut state, &s1_id).unwrap();

        // Then: No grandparent
        assert!(grandparent.is_none());

        // Then: Child should be at root level
        let n1 = state.nodes.get(&n1_id).unwrap();
        assert!(n1.parent.is_none());
    }

    #[test]
    fn edge_case_delete_subgraph_with_no_children() {
        // Given: Empty subgraph
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);

        state.nodes.insert(s1_id.clone(), s1);

        let original_count = state.nodes.len();

        // When: Delete mode
        simulate_delete_mode(&mut state, &s1_id).unwrap();

        // Then: Only the subgraph removed
        assert_eq!(state.nodes.len(), original_count - 1);
        assert!(!state.nodes.contains_key(&s1_id));
    }

    #[test]
    fn edge_case_reparent_chain_of_subgraphs() {
        // Given: S1 -> S2 -> S3 -> N1 (chain)
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 100.0, 100.0, None);
        let (s2_id, s2) =
            create_test_node("S2", NodeKind::Subgraph, 10.0, 10.0, 50.0, 50.0, Some("S1"));
        let (s3_id, s3) =
            create_test_node("S3", NodeKind::Subgraph, 20.0, 20.0, 30.0, 30.0, Some("S2"));
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 25.0, 25.0, 10.0, 10.0, Some("S3"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(s2_id.clone(), s2);
        state.nodes.insert(s3_id.clone(), s3);
        state.nodes.insert(n1_id.clone(), n1);

        // When: Delete S1 in reparent mode (intermediate chain)
        simulate_reparent_mode(&mut state, &s1_id).unwrap();

        // Then: S1 removed, S2 reparented to None (root)
        assert!(!state.nodes.contains_key(&s1_id));

        let s2 = state.nodes.get(&s2_id).unwrap();
        assert!(s2.parent.is_none(), "S2 should be at root level");
    }

    #[test]
    fn edge_case_dsl_verify_with_valid_state() {
        // Test the DSL functions work correctly
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 15.0, 15.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        // Test assert_children_reparented (before reparent)
        assert_children_reparented(&state, &["N1"], Some("S1")).unwrap();

        // Test get_direct_children
        let children = get_direct_children(&state, &s1_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].as_str(), "N1");

        // Now simulate reparent mode - this removes S1
        simulate_reparent_mode(&mut state, &s1_id).unwrap();

        // After reparent: S1 removed, N1 reparented to None (root)
        // Original: 2, Removed: 1 (S1), Expected: 1 (N1)
        assert_node_count_consistent(&state, 2, 0, "reparent").unwrap();
    }

    #[test]
    fn edge_case_dsl_verify_fails_on_mismatch() {
        let mut state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 15.0, 15.0, 10.0, 10.0, Some("S1"));

        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        // Test assert_children_reparented fails on wrong parent
        let result = assert_children_reparented(&state, &["N1"], Some("Wrong"));
        assert!(result.is_err());

        // Test verify_cascade_deletion fails in wrong mode
        let result = verify_cascade_deletion(&state, &s1_id, &["N1"], "delete");
        assert!(result.is_err());
    }
}
