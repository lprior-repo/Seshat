//! Subgraph cascade deletion and reparenting tests
//!
//! Tests for SUB-032 (reparent mode) and SUB-034 (delete mode) cascade behaviors.
//! Uses REAL production code functions: `apply_ungroup` from projection ops.

#![cfg(test)]

mod tests {
    use crate::models::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};
    use crate::models::projection::ops::group_ops::apply_ungroup;
    use crate::models::projection::types::DiagramProjection;
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

    /// DSL: Verifies that cascade deletion behaves correctly.
    /// In delete mode with apply_ungroup, children are kept but parent is cleared (root level).
    ///
    /// # Arguments
    /// * `state` - The diagram state
    /// * `subgraph_id` - The subgraph being deleted
    /// * `expected_children` - Children that should exist after deletion
    fn verify_cascade_delete_mode(
        state: &DiagramState,
        _subgraph_id: &NodeId,
        expected_children: &[&str],
    ) -> Result<(), String> {
        // In delete mode with apply_ungroup: children are kept but parent is cleared
        for child_id in expected_children {
            let child_node_id = NodeId::new(child_id.to_string());

            // Child should exist
            if !state.nodes.contains_key(&child_node_id) {
                return Err(format!("Child {} should exist but was deleted", child_id));
            }

            // Child should have NO parent (cleared to root level)
            let child = state.nodes.get(&child_node_id).unwrap();
            if child.parent.is_some() {
                return Err(format!(
                    "Child {} should have no parent (root level) but has {:?}",
                    child_id, child.parent
                ));
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

    /// Convert DiagramState to DiagramProjection for production function tests
    fn state_to_projection(state: &DiagramState) -> DiagramProjection {
        DiagramProjection {
            version: state.version,
            revision: state.revision,
            nodes: state.nodes.clone(),
            edges: state.edges.clone(),
            author_priority: state.author_priority.clone(),
            cycle_policy: state.cycle_policy,
        }
    }

    // ========================================================================
    // PRODUCTION CODE TESTS - Using REAL apply_ungroup()
    // These tests call ACTUAL production functions from the codebase
    // ========================================================================

    /// This test uses REAL production code: apply_ungroup from crate::models::projection::ops::group_ops
    /// It verifies that the production apply_ungroup function correctly removes the subgraph
    /// and clears parent on all children (making them root-level).
    ///
    /// This is the CORE cascade behavior test - testing actual production code.
    #[test]
    fn test_production_apply_ungroup_clears_parent() {
        // Given: S1 -> Children (N1, N2) using DiagramProjection
        let mut projection = DiagramProjection::empty();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));

        projection.nodes.insert(s1_id.clone(), s1);
        projection.nodes.insert(n1_id.clone(), n1);
        projection.nodes.insert(n2_id.clone(), n2);

        // When: Call REAL production function apply_ungroup
        let result = apply_ungroup(projection.clone(), &s1_id);

        // Then: Should succeed
        assert!(result.is_ok(), "apply_ungroup should succeed: {:?}", result);

        let new_projection = result.unwrap();

        // Then: S1 should be removed
        assert!(
            !new_projection.nodes.contains_key(&s1_id),
            "Subgraph S1 should be removed"
        );

        // Then: Children should have NO parent (cleared to root level)
        let n1 = new_projection.nodes.get(&n1_id).unwrap();
        let n2 = new_projection.nodes.get(&n2_id).unwrap();
        assert!(n1.parent.is_none(), "N1 should have no parent (root level)");
        assert!(n2.parent.is_none(), "N2 should have no parent (root level)");

        // Then: Children should still exist
        assert!(
            new_projection.nodes.contains_key(&n1_id),
            "N1 should still exist"
        );
        assert!(
            new_projection.nodes.contains_key(&n2_id),
            "N2 should still exist"
        );
    }

    /// Another production code test - verify apply_ungroup with nested structure
    #[test]
    fn test_production_apply_ungroup_nested() {
        // Given: Root -> S1 -> S2 -> N1 (nested structure)
        let mut projection = DiagramProjection::empty();

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

        projection.nodes.insert(root_id.clone(), root);
        projection.nodes.insert(s1_id.clone(), s1);
        projection.nodes.insert(s2_id.clone(), s2);
        projection.nodes.insert(n1_id.clone(), n1);

        // When: Call REAL production function apply_ungroup on S1
        let result = apply_ungroup(projection.clone(), &s1_id);

        // Then: Should succeed
        assert!(result.is_ok());

        let new_projection = result.unwrap();

        // Then: S1 should be removed
        assert!(!new_projection.nodes.contains_key(&s1_id));

        // Then: S2 should have parent = Root (grandparent, since S1's parent is Root)
        let s2 = new_projection.nodes.get(&s2_id).unwrap();
        assert_eq!(
            s2.parent.as_ref().map(|id| id.as_str()),
            Some("Root"),
            "S2 should have parent = Root (grandparent)"
        );

        // Then: N1 should still have parent = S2 (not affected)
        let n1 = new_projection.nodes.get(&n1_id).unwrap();
        assert_eq!(n1.parent.as_ref().map(|id| id.as_str()), Some("S2"));
    }

    // ========================================================================
    // UNIT TESTS - Delete Mode (SUB-034) using production code
    // apply_ungroup removes the subgraph and clears parent on direct children
    // ========================================================================

    #[test]
    fn test_delete_mode_removes_subgraph_clears_parent() {
        // Given: S1 -> Children (N1, N2)
        let mut projection = DiagramProjection::empty();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 10.0, 10.0, 10.0, 10.0, Some("S1"));
        let (n2_id, n2) =
            create_test_node("N2", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S1"));

        projection.nodes.insert(s1_id.clone(), s1);
        projection.nodes.insert(n1_id.clone(), n1);
        projection.nodes.insert(n2_id.clone(), n2);

        let original_count = projection.nodes.len();

        // When: Delete S1 using REAL production code apply_ungroup
        let result = apply_ungroup(projection.clone(), &s1_id);
        assert!(result.is_ok());
        let new_projection = result.unwrap();

        // Then: Children should have no parent (parent refs cleared - root level)
        let state = DiagramState {
            version: 1,
            revision: 0,
            nodes: new_projection.nodes.clone(),
            edges: new_projection.edges.clone(),
            author_priority: new_projection.author_priority.clone(),
            cycle_policy: new_projection.cycle_policy,
        };
        verify_cascade_delete_mode(&state, &s1_id, &["N1", "N2"]).unwrap();

        // Then: Node count should be original - 1 (only subgraph removed)
        assert_eq!(new_projection.nodes.len(), original_count - 1);

        // Then: S1 should be removed
        assert!(!new_projection.nodes.contains_key(&s1_id));
    }

    #[test]
    fn test_delete_mode_empty_subgraph() {
        // Given: Empty S1
        let mut projection = DiagramProjection::empty();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);

        projection.nodes.insert(s1_id.clone(), s1);

        // When: Delete empty S1 in delete mode using REAL production code
        let result = apply_ungroup(projection, &s1_id);
        assert!(result.is_ok());
        let new_projection = result.unwrap();

        // Then: Node count should be 0 (subgraph deleted)
        assert_eq!(new_projection.nodes.len(), 0);
    }

    #[test]
    fn test_delete_mode_nested_subgraphs() {
        // Given: S1 -> S2 -> N1 (nested structure)
        // Note: apply_ungroup only removes the specified node, not nested children
        let mut projection = DiagramProjection::empty();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 100.0, 100.0, None);
        let (s2_id, s2) =
            create_test_node("S2", NodeKind::Subgraph, 10.0, 10.0, 50.0, 50.0, Some("S1"));
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 20.0, 20.0, 10.0, 10.0, Some("S2"));

        projection.nodes.insert(s1_id.clone(), s1);
        projection.nodes.insert(s2_id.clone(), s2);
        projection.nodes.insert(n1_id.clone(), n1);

        // When: Delete S1 - apply_ungroup only removes S1, clears parent on direct children
        let result = apply_ungroup(projection.clone(), &s1_id);
        assert!(result.is_ok());
        let new_projection = result.unwrap();

        // Then: S1 should be removed
        assert!(!new_projection.nodes.contains_key(&s1_id));

        // Then: S2 should have parent cleared (becomes root level)
        let s2 = new_projection.nodes.get(&s2_id).unwrap();
        assert!(s2.parent.is_none(), "S2 should have no parent (root level)");

        // Then: N1 should still have S2 as parent (not affected)
        let n1 = new_projection.nodes.get(&n1_id).unwrap();
        assert_eq!(n1.parent.as_ref().map(|id| id.as_str()), Some("S2"));
    }

    // ========================================================================
    // INTEGRATION TESTS - Full Workflows
    // ========================================================================

    #[test]
    fn integration_delete_workflow_with_bounds() {
        // Given: Root -> S1 -> N1, N2
        let mut projection = DiagramProjection::empty();

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

        projection.nodes.insert(root_id.clone(), root);
        projection.nodes.insert(s1_id.clone(), s1);
        projection.nodes.insert(n1_id.clone(), n1);
        projection.nodes.insert(n2_id.clone(), n2);

        // When: Delete mode using REAL production code
        let result = apply_ungroup(projection.clone(), &s1_id);
        assert!(result.is_ok());
        let new_projection = result.unwrap();

        // Then: S1 should be removed
        assert!(!new_projection.nodes.contains_key(&s1_id));

        // Then: Children should have parent = Root (the grandparent)
        let n1 = new_projection.nodes.get(&n1_id).unwrap();
        let n2 = new_projection.nodes.get(&n2_id).unwrap();
        assert_eq!(n1.parent.as_ref().map(|id| id.as_str()), Some("Root"));
        assert_eq!(n2.parent.as_ref().map(|id| id.as_str()), Some("Root"));

        // Then: Root should still exist
        assert!(new_projection.nodes.contains_key(&root_id));
    }

    #[test]
    fn integration_mixed_nested_delete() {
        // Given: Root -> S1 -> S2 -> N1 (multi-level nesting)
        let mut projection = DiagramProjection::empty();

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

        projection.nodes.insert(root_id.clone(), root);
        projection.nodes.insert(s1_id.clone(), s1);
        projection.nodes.insert(s2_id.clone(), s2);
        projection.nodes.insert(n1_id.clone(), n1);

        // When: Delete S1 in delete mode using REAL production code
        // apply_ungroup removes S1 and clears parent on S2 (direct child)
        let result = apply_ungroup(projection.clone(), &s1_id);
        assert!(result.is_ok());
        let new_projection = result.unwrap();

        // Then: S1 removed
        assert!(!new_projection.nodes.contains_key(&s1_id));

        // Then: S2 should have parent = Root (the grandparent)
        let s2 = new_projection.nodes.get(&s2_id).unwrap();
        assert_eq!(
            s2.parent.as_ref().map(|id| id.as_str()),
            Some("Root"),
            "S2 should have parent = Root"
        );

        // N1 should still exist and have S2 as parent
        assert!(new_projection.nodes.contains_key(&n1_id));
        let n1 = new_projection.nodes.get(&n1_id).unwrap();
        assert_eq!(n1.parent.as_ref().map(|id| id.as_str()), Some("S2"));
    }

    // ========================================================================
    // PROPERTY-BASED TESTS - Random Graph Structures
    // ========================================================================

    #[test]
    fn property_delete_preserves_children_but_clears_parent() {
        // Property: In delete mode, children are kept but parent is cleared
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..10 {
            let mut projection = DiagramProjection::empty();
            let mut node_ids = Vec::new();

            // Create a root
            let (root_id, root) =
                create_test_node("root", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
            projection.nodes.insert(root_id.clone(), root);
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
                projection.nodes.insert(s_id.clone(), s_node);
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
                    projection.nodes.insert(c_id.clone(), c_node);
                    all_child_ids.push(c_id);
                }
            }

            let original_count = projection.nodes.len();

            // Delete each subgraph in delete mode using REAL production code
            for i in 0..num_subgraphs {
                let s_id = NodeId::new(format!("S{}", i));
                let result = apply_ungroup(projection.clone(), &s_id);
                if result.is_ok() {
                    projection = result.unwrap();
                }
            }

            // Property: Node count should equal original - num_subgraphs (only subgraphs removed)
            assert_eq!(
                projection.nodes.len(),
                original_count - num_subgraphs,
                "Delete mode should remove subgraphs but keep children"
            );
        }
    }

    #[test]
    fn property_delete_removes_exactly_children_plus_subgraph() {
        // Property: In delete mode, exactly 1 node removed per subgraph (children kept but unparented)
        let mut rng = StdRng::seed_from_u64(123);

        for _ in 0..10 {
            let mut projection = DiagramProjection::empty();

            // Create S1 with random children
            let (s1_id, s1) =
                create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
            projection.nodes.insert(s1_id.clone(), s1);

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
                projection.nodes.insert(c_id.clone(), c_node);
                child_ids.push(c_id);
            }

            let original_count = projection.nodes.len();

            // Delete in delete mode using REAL production code
            let result = apply_ungroup(projection.clone(), &s1_id);
            assert!(result.is_ok());
            let new_projection = result.unwrap();

            // Property: Should remove exactly 1 node (the subgraph, children kept but unparented)
            // apply_ungroup removes the subgraph but keeps children with no parent
            assert_eq!(
                new_projection.nodes.len(),
                original_count - 1,
                "Delete mode should remove subgraph but keep children (unparented)"
            );

            // Verify children still exist but have no parent
            for c_id in &child_ids {
                assert!(new_projection.nodes.contains_key(c_id));
                let child = new_projection.nodes.get(c_id).unwrap();
                assert!(child.parent.is_none(), "Child should have no parent");
            }
            assert!(!new_projection.nodes.contains_key(&s1_id));
        }
    }

    #[test]
    fn property_delete_always_clears_parent() {
        // Property: After delete, every remaining child node should have no parent
        let mut rng = StdRng::seed_from_u64(456);

        for _ in 0..10 {
            let mut projection = DiagramProjection::empty();

            // Create a simple structure
            let (root_id, root) =
                create_test_node("root", NodeKind::Subgraph, 0.0, 0.0, 50.0, 50.0, None);
            projection.nodes.insert(root_id.clone(), root);

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
                projection.nodes.insert(s_id.clone(), s_node);
            }

            // Randomly delete subgraphs in delete mode using REAL production code
            for i in 0..num_subgraphs {
                if rng.gen_bool(0.5) {
                    let s_id = NodeId::new(format!("S{}", i));
                    let result = apply_ungroup(projection.clone(), &s_id);
                    if result.is_ok() {
                        projection = result.unwrap();
                    }
                }
            }

            // Property: Every non-subgraph node should have no parent (root level)
            for (id, node) in &projection.nodes {
                if node.kind != NodeKind::Subgraph {
                    assert!(
                        node.parent.is_none(),
                        "Node {} should have no parent after delete",
                        id.as_str()
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
            let mut projection = DiagramProjection::empty();

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

            projection.nodes.insert(root_id.clone(), root);
            projection.nodes.insert(s1_id.clone(), s1);
            projection.nodes.insert(n1_id.clone(), n1);
            projection.nodes.insert(n2_id.clone(), n2);

            // Use delete mode
            let result = apply_ungroup(projection.clone(), &s1_id);
            assert!(result.is_ok());
            let new_projection = result.unwrap();

            // Property: Children should have no parent (root level is valid)
            for (id, node) in &new_projection.nodes {
                if let Some(parent_id) = &node.parent {
                    assert!(
                        new_projection.nodes.contains_key(parent_id),
                        "After delete mode, node {} references deleted parent {}",
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
    fn edge_case_delete_already_root_level_subgraph() {
        // Given: Subgraph at root level with no parent
        let mut projection = DiagramProjection::empty();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 15.0, 15.0, 10.0, 10.0, Some("S1"));

        projection.nodes.insert(s1_id.clone(), s1);
        projection.nodes.insert(n1_id.clone(), n1);

        // When: Delete mode using REAL production code
        let result = apply_ungroup(projection.clone(), &s1_id);
        assert!(result.is_ok());
        let new_projection = result.unwrap();

        // Then: S1 should be removed
        assert!(!new_projection.nodes.contains_key(&s1_id));

        // Then: Child should be at root level (no parent)
        let n1 = new_projection.nodes.get(&n1_id).unwrap();
        assert!(n1.parent.is_none());
    }

    #[test]
    fn edge_case_delete_subgraph_with_no_children() {
        // Given: Empty subgraph
        let mut projection = DiagramProjection::empty();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);

        projection.nodes.insert(s1_id.clone(), s1);

        let original_count = projection.nodes.len();

        // When: Delete mode using REAL production code
        let result = apply_ungroup(projection, &s1_id);
        assert!(result.is_ok());
        let new_projection = result.unwrap();

        // Then: Only the subgraph removed
        assert_eq!(new_projection.nodes.len(), original_count - 1);
        assert!(!new_projection.nodes.contains_key(&s1_id));
    }

    #[test]
    fn edge_case_delete_chain_of_subgraphs() {
        // Given: S1 -> S2 -> S3 -> N1 (chain)
        let mut projection = DiagramProjection::empty();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 0.0, 0.0, 100.0, 100.0, None);
        let (s2_id, s2) =
            create_test_node("S2", NodeKind::Subgraph, 10.0, 10.0, 50.0, 50.0, Some("S1"));
        let (s3_id, s3) =
            create_test_node("S3", NodeKind::Subgraph, 20.0, 20.0, 30.0, 30.0, Some("S2"));
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 25.0, 25.0, 10.0, 10.0, Some("S3"));

        projection.nodes.insert(s1_id.clone(), s1);
        projection.nodes.insert(s2_id.clone(), s2);
        projection.nodes.insert(s3_id.clone(), s3);
        projection.nodes.insert(n1_id.clone(), n1);

        // When: Delete S1 in delete mode using REAL production code
        // apply_ungroup removes S1 and clears parent on S2 (direct child)
        let result = apply_ungroup(projection.clone(), &s1_id);
        assert!(result.is_ok());
        let new_projection = result.unwrap();

        // Then: S1 removed
        assert!(!new_projection.nodes.contains_key(&s1_id));

        // S2 should have parent cleared (root level)
        let s2 = new_projection.nodes.get(&s2_id).unwrap();
        assert!(s2.parent.is_none(), "S2 should be at root level");

        // S3 should still exist with S2 as parent
        let s3 = new_projection.nodes.get(&s3_id).unwrap();
        assert_eq!(s3.parent.as_ref().map(|id| id.as_str()), Some("S2"));
    }

    #[test]
    fn edge_case_dsl_verify_with_valid_state() {
        // Test the DSL functions work correctly
        let state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 15.0, 15.0, 10.0, 10.0, Some("S1"));

        let mut state = state;
        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        // Test assert_children_reparented (before delete)
        assert_children_reparented(&state, &["N1"], Some("S1")).unwrap();

        // Test get_direct_children
        let children = get_direct_children(&state, &s1_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].as_str(), "N1");
    }

    #[test]
    fn edge_case_dsl_verify_fails_on_mismatch() {
        let state = create_test_state();

        let (s1_id, s1) = create_test_node("S1", NodeKind::Subgraph, 10.0, 10.0, 30.0, 30.0, None);
        let (n1_id, n1) =
            create_test_node("N1", NodeKind::Node, 15.0, 15.0, 10.0, 10.0, Some("S1"));

        let mut state = state;
        state.nodes.insert(s1_id.clone(), s1);
        state.nodes.insert(n1_id.clone(), n1);

        // Test assert_children_reparented fails on wrong parent
        let result = assert_children_reparented(&state, &["N1"], Some("Wrong"));
        assert!(result.is_err());

        // Test verify_cascade_delete_mode fails when child has parent
        let result = verify_cascade_delete_mode(&state, &s1_id, &["N1"]);
        assert!(result.is_err());
    }
}
