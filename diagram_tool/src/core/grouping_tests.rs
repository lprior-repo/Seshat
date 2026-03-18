// Tests for grouping module - included via #[path] in grouping.rs
#[cfg(test)]
mod tests {
    use crate::core::grouping::{group_selection, ungroup_selection, GroupingError};
    use crate::test_utils::builders::{
        test_edge, test_node, test_subgraph, EdgeBuilder, NodeBuilder,
    };
    use diagram_models::document::{
        DiagramDocument, Edge, LockState, NodeId, NodeKind, OrderedFloat,
    };

    // =====================================================================
    // Original Kani Tests
    // =====================================================================

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_group_selection_creates_padded_container_and_reparents() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        let n2 = NodeId::new("2".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 100.0, 50.0, 50.0));
        doc.document
            .nodes
            .insert(n2.clone(), test_node(200.0, 200.0, 50.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2.as_str().to_string());
        let group_id = NodeId::new("g1".to_string());
        group_selection(&mut doc, &group_id).unwrap();
        let group = doc.document.nodes.get(&group_id).unwrap();
        assert_eq!(group.kind, NodeKind::Subgraph);
        assert_eq!(group.x.0, 80.0);
        assert_eq!(group.y.0, 80.0);
        assert_eq!(group.width.0, 190.0);
        assert_eq!(group.height.0, 190.0);
        assert_eq!(
            doc.document.nodes.get(&n1).unwrap().parent,
            Some(group_id.clone())
        );
        assert_eq!(
            doc.document.nodes.get(&n2).unwrap().parent,
            Some(group_id.clone())
        );
        assert_eq!(doc.editor_state.selected_items.len(), 1);
        assert!(doc.editor_state.selected_items.contains(group_id.as_str()));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_ungroup_selection_empty() {
        let mut doc = DiagramDocument::default();
        assert_eq!(
            ungroup_selection(&mut doc),
            Err(GroupingError::EmptySelection)
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_ungroup_selection_no_subgraphs_selected() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(0.0, 0.0, 50.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        assert_eq!(
            ungroup_selection(&mut doc),
            Err(GroupingError::EmptySelection)
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_ungroup_selection_deletes_subgraph_and_orphans_children() {
        let mut doc = DiagramDocument::default();
        let group_id = NodeId::new("g1".to_string());
        doc.document.nodes.insert(group_id.clone(), test_subgraph());
        let mut child1 = test_node(10.0, 10.0, 20.0, 20.0);
        child1.parent = Some(group_id.clone());
        let c1_id = NodeId::new("c1".to_string());
        doc.document.nodes.insert(c1_id.clone(), child1);
        let mut child2 = test_node(40.0, 40.0, 20.0, 20.0);
        child2.parent = Some(group_id.clone());
        let c2_id = NodeId::new("c2".to_string());
        doc.document.nodes.insert(c2_id.clone(), child2);
        doc.editor_state
            .selected_items
            .insert(group_id.as_str().to_string());
        assert_eq!(ungroup_selection(&mut doc), Ok(()));
        assert!(!doc.document.nodes.contains_key(&group_id));
        assert_eq!(doc.document.nodes.get(&c1_id).unwrap().parent, None);
        assert_eq!(doc.document.nodes.get(&c2_id).unwrap().parent, None);
        assert_eq!(doc.editor_state.selected_items.len(), 2);
        assert!(doc.editor_state.selected_items.contains(c1_id.as_str()));
        assert!(doc.editor_state.selected_items.contains(c2_id.as_str()));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_ungroup_selection_nested_subgraphs() {
        let mut doc = DiagramDocument::default();
        let parent_group_id = NodeId::new("pg".to_string());
        doc.document
            .nodes
            .insert(parent_group_id.clone(), test_subgraph());
        let mut sub_group = test_subgraph();
        sub_group.parent = Some(parent_group_id.clone());
        let sub_group_id = NodeId::new("sg".to_string());
        doc.document.nodes.insert(sub_group_id.clone(), sub_group);
        let mut child = test_node(10.0, 10.0, 20.0, 20.0);
        child.parent = Some(sub_group_id.clone());
        let c_id = NodeId::new("c".to_string());
        doc.document.nodes.insert(c_id.clone(), child);
        doc.editor_state
            .selected_items
            .insert(sub_group_id.as_str().to_string());
        assert_eq!(ungroup_selection(&mut doc), Ok(()));
        assert!(!doc.document.nodes.contains_key(&sub_group_id));
        assert!(doc.document.nodes.contains_key(&parent_group_id));
        assert_eq!(
            doc.document.nodes.get(&c_id).unwrap().parent,
            Some(parent_group_id)
        );
        assert_eq!(doc.editor_state.selected_items.len(), 1);
        assert!(doc.editor_state.selected_items.contains(c_id.as_str()));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_ungroup_selection_removes_edges_connected_to_subgraph() {
        let mut doc = DiagramDocument::default();
        let group_id = NodeId::new("g1".to_string());
        doc.document.nodes.insert(group_id.clone(), test_subgraph());
        let node_id = NodeId::new("n1".to_string());
        doc.document
            .nodes
            .insert(node_id.clone(), test_node(100.0, 100.0, 20.0, 20.0));
        let edge_id = diagram_models::document::EdgeId::new("e1".to_string());
        let edge = Edge {
            source: group_id.clone(),
            target: node_id.clone(),
            label: String::new(),
            style: diagram_models::document::EdgeStyle::default(),
            arrow_type: diagram_models::document::ArrowType::default(),
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
        };
        doc.document.edges.insert(edge_id.clone(), edge);
        doc.editor_state
            .selected_items
            .insert(group_id.as_str().to_string());
        assert_eq!(ungroup_selection(&mut doc), Ok(()));
        assert!(!doc.document.edges.contains_key(&edge_id));
    }

    // =====================================================================
    // Invariant Checking Functions
    // =====================================================================

    /// INV1: Parent chain validity - every node's parent must be a valid subgraph
    fn check_parent_chain_validity(doc: &DiagramDocument) -> bool {
        doc.document.nodes.values().all(|node| {
            node.parent.as_ref().is_none_or(|parent_id| {
                doc.document
                    .nodes
                    .get(parent_id)
                    .is_some_and(|p| p.kind == NodeKind::Subgraph)
            })
        })
    }

    /// INV2: No orphan edges - every edge must connect to existing nodes
    fn check_no_orphan_edges(doc: &DiagramDocument) -> bool {
        doc.document.edges.values().all(|edge| {
            doc.document.nodes.contains_key(&edge.source)
                && doc.document.nodes.contains_key(&edge.target)
        })
    }

    /// INV3: Node count consistency
    fn check_node_count_consistency(
        original_count: usize,
        deleted_subgraphs: usize,
        doc: &DiagramDocument,
    ) -> bool {
        doc.document.nodes.len() == original_count - deleted_subgraphs
    }

    // =====================================================================
    // Property-Based Tests (using deterministic enumeration for simplicity)
    // These test graph invariants with various input configurations
    // =====================================================================

    /// Property: Group selection maintains invariants
    #[test]
    fn test_group_selection_invariants_2_nodes() {
        let coords = vec![(100.0, 100.0, 50.0, 50.0), (200.0, 200.0, 50.0, 50.0)];
        test_group_selection_invariants_with_coords(coords);
    }

    #[test]
    fn test_group_selection_invariants_3_nodes() {
        let coords = vec![
            (100.0, 100.0, 50.0, 50.0),
            (200.0, 200.0, 50.0, 50.0),
            (300.0, 300.0, 50.0, 50.0),
        ];
        test_group_selection_invariants_with_coords(coords);
    }

    #[test]
    fn test_group_selection_invariants_5_nodes() {
        let coords = vec![
            (100.0, 100.0, 50.0, 50.0),
            (200.0, 200.0, 50.0, 50.0),
            (300.0, 300.0, 50.0, 50.0),
            (400.0, 400.0, 50.0, 50.0),
            (500.0, 500.0, 50.0, 50.0),
        ];
        test_group_selection_invariants_with_coords(coords);
    }

    fn test_group_selection_invariants_with_coords(coords: Vec<(f64, f64, f64, f64)>) {
        let mut doc = DiagramDocument::default();
        let mut node_ids = Vec::new();

        for (i, (x, y, w, h)) in coords.into_iter().enumerate() {
            let id = NodeId::new(format!("n{i}"));
            doc.document.nodes.insert(id.clone(), test_node(x, y, w, h));
            doc.editor_state
                .selected_items
                .insert(id.as_str().to_string());
            node_ids.push(id);
        }

        let group_id = NodeId::new("group".to_string());
        let result = group_selection(&mut doc, &group_id);
        assert!(result.is_ok(), "Group selection should succeed");

        // INV1: All children should have the group as parent
        for id in &node_ids {
            let node = doc.document.nodes.get(id).unwrap();
            assert_eq!(
                node.parent.as_ref(),
                Some(&group_id),
                "Node {id} should have group as parent"
            );
        }

        // INV2: All edges should be valid
        assert!(
            check_no_orphan_edges(&doc),
            "No orphan edges after grouping"
        );
    }

    /// Property: Ungroup maintains invariants - various configurations
    #[test]
    fn test_ungroup_invariants_1sg_0child() {
        test_ungroup_invariants_with_config(1, 0);
    }

    #[test]
    fn test_ungroup_invariants_1sg_1child() {
        test_ungroup_invariants_with_config(1, 1);
    }

    #[test]
    fn test_ungroup_invariants_1sg_2child() {
        test_ungroup_invariants_with_config(1, 2);
    }

    #[test]
    fn test_ungroup_invariants_1sg_3child() {
        test_ungroup_invariants_with_config(1, 3);
    }

    #[test]
    fn test_ungroup_invariants_2sg_2child_each() {
        test_ungroup_invariants_with_config(2, 2);
    }

    #[test]
    fn test_ungroup_invariants_3sg_3child_each() {
        test_ungroup_invariants_with_config(3, 3);
    }

    fn test_ungroup_invariants_with_config(num_subgraphs: usize, children_per_sg: usize) {
        let mut doc = DiagramDocument::default();
        let mut sgs = Vec::new();

        for i in 0..num_subgraphs {
            let sg_id = NodeId::new(format!("sg{i}"));
            let mut sg = test_subgraph();
            sg.x = OrderedFloat((i as f64) * 100.0);
            doc.document.nodes.insert(sg_id.clone(), sg);
            sgs.push(sg_id);
        }

        // Add children
        for (sg_idx, sg_id) in sgs.iter().enumerate() {
            for j in 0..children_per_sg {
                let child_id = NodeId::new(format!("c{sg_idx}-{j}"));
                let mut child = test_node(
                    (sg_idx as f64).mul_add(100.0, (j as f64) * 20.0),
                    50.0,
                    15.0,
                    15.0,
                );
                child.parent = Some(sg_id.clone());
                doc.document.nodes.insert(child_id.clone(), child);
            }
        }

        // Select all subgraphs
        for sg in &sgs {
            doc.editor_state
                .selected_items
                .insert(sg.as_str().to_string());
        }

        let original_count = doc.document.nodes.len();
        let result = ungroup_selection(&mut doc);

        if num_subgraphs > 0 {
            assert!(result.is_ok(), "Ungroup should succeed");

            // INV1: Parent chain valid
            assert!(
                check_parent_chain_validity(&doc),
                "INV1: parent chain valid"
            );

            // INV2: No orphan edges
            assert!(check_no_orphan_edges(&doc), "INV2: no orphan edges");

            // INV3: Node count decreases by num_subgraphs
            assert!(
                check_node_count_consistency(original_count, num_subgraphs, &doc),
                "INV3: node count correct"
            );
        }
    }

    /// Property: Deep nesting maintains invariants
    #[test]
    fn test_nested_subgraph_invariants_depth_1() {
        test_nested_subgraph_invariants_with_depth(1);
    }

    #[test]
    fn test_nested_subgraph_invariants_depth_2() {
        test_nested_subgraph_invariants_with_depth(2);
    }

    #[test]
    fn test_nested_subgraph_invariants_depth_3() {
        test_nested_subgraph_invariants_with_depth(3);
    }

    #[test]
    fn test_nested_subgraph_invariants_depth_4() {
        test_nested_subgraph_invariants_with_depth(4);
    }

    #[test]
    fn test_nested_subgraph_invariants_depth_5() {
        test_nested_subgraph_invariants_with_depth(5);
    }

    fn test_nested_subgraph_invariants_with_depth(depth: usize) {
        let mut doc = DiagramDocument::default();
        let mut parents: Vec<NodeId> = Vec::new();

        for i in 0..depth {
            let id = NodeId::new(format!("sg{i}"));
            let mut sg = test_subgraph();
            sg.x = OrderedFloat((i as f64) * 50.0);
            if i > 0 {
                sg.parent = Some(parents[i - 1].clone());
            }
            doc.document.nodes.insert(id.clone(), sg);
            parents.push(id);
        }

        // Add leaf child
        let leaf_id = NodeId::new("leaf".to_string());
        let mut leaf = test_node(300.0, 50.0, 20.0, 20.0);
        leaf.parent = Some(parents[depth - 1].clone());
        doc.document.nodes.insert(leaf_id.clone(), leaf);

        doc.editor_state
            .selected_items
            .insert(parents[depth - 1].as_str().to_string());

        let result = ungroup_selection(&mut doc);
        assert!(result.is_ok(), "Ungroup should succeed");

        // INV1: Parent chain valid
        assert!(
            check_parent_chain_validity(&doc),
            "INV1: parent chain valid"
        );

        // INV2: No orphan edges
        assert!(check_no_orphan_edges(&doc), "INV2: no orphan edges");

        // Leaf should be reparented
        let leaf_node = doc.document.nodes.get(&leaf_id).unwrap();
        if depth > 1 {
            assert_eq!(leaf_node.parent, Some(parents[depth - 2].clone()));
        } else {
            assert_eq!(leaf_node.parent, None);
        }
    }

    /// Property: Empty selection returns error
    #[test]
    fn test_empty_selection_error() {
        let mut doc = DiagramDocument::default();
        let result = ungroup_selection(&mut doc);
        assert!(matches!(result, Err(GroupingError::EmptySelection)));
    }

    /// Property: Non-subgraph selection returns error
    #[test]
    fn test_non_subgraph_selection_error() {
        let mut doc = DiagramDocument::default();
        let id = NodeId::new("node".to_string());
        doc.document
            .nodes
            .insert(id.clone(), test_node(0.0, 0.0, 50.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(id.as_str().to_string());
        let result = ungroup_selection(&mut doc);
        assert!(matches!(result, Err(GroupingError::EmptySelection)));
    }

    // =====================================================================
    // Deterministic Enumerative Tests for Invariants
    // =====================================================================

    /// Test ungroup with various child counts
    #[test]
    fn test_ungroup_various_counts_invariants() {
        for num_children in 0..=5 {
            let mut doc = DiagramDocument::default();
            let sg_id = NodeId::new("sg".to_string());
            doc.document.nodes.insert(sg_id.clone(), test_subgraph());

            for i in 0..num_children {
                let child_id = NodeId::new(format!("child{i}"));
                let mut child = test_node(f64::from(i) * 30.0, 50.0, 20.0, 20.0);
                child.parent = Some(sg_id.clone());
                doc.document.nodes.insert(child_id.clone(), child);
            }

            doc.editor_state
                .selected_items
                .insert(sg_id.as_str().to_string());

            let original_count = doc.document.nodes.len();
            let result = ungroup_selection(&mut doc);

            if num_children == 0 {
                // Empty subgraph case
                assert!(result.is_ok() || matches!(result, Err(GroupingError::EmptySelection)));
            } else {
                assert!(result.is_ok());
                assert!(
                    check_parent_chain_validity(&doc),
                    "INV1 for {num_children} children"
                );
                assert!(
                    check_no_orphan_edges(&doc),
                    "INV2 for {num_children} children"
                );
                assert_eq!(
                    doc.document.nodes.len(),
                    original_count - 1,
                    "INV3 for {num_children} children"
                );
            }
        }
    }

    /// Test multiple subgraphs ungroup maintains invariants
    #[test]
    fn test_multiple_subgraphs_invariants() {
        for num_sgs in 1..=4 {
            for children_per_sg in 0..=3 {
                let mut doc = DiagramDocument::default();
                let mut sgs = Vec::new();

                for i in 0..num_sgs {
                    let sg_id = NodeId::new(format!("sg{i}"));
                    let mut sg = test_subgraph();
                    sg.x = OrderedFloat((i as f64) * 100.0);
                    doc.document.nodes.insert(sg_id.clone(), sg);
                    sgs.push(sg_id);
                }

                for (sg_idx, sg_id) in sgs.iter().enumerate() {
                    for j in 0..children_per_sg {
                        let child_id = NodeId::new(format!("child-{sg_idx}-{j}"));
                        let mut child = test_node(
                            (sg_idx as f64).mul_add(100.0, f64::from(j) * 20.0),
                            50.0,
                            15.0,
                            15.0,
                        );
                        child.parent = Some(sg_id.clone());
                        doc.document.nodes.insert(child_id.clone(), child);
                    }
                }

                for sg in &sgs {
                    doc.editor_state
                        .selected_items
                        .insert(sg.as_str().to_string());
                }

                let original_count = doc.document.nodes.len();
                let result = ungroup_selection(&mut doc);

                assert!(result.is_ok());
                assert!(check_parent_chain_validity(&doc));
                assert!(check_no_orphan_edges(&doc));
                assert_eq!(doc.document.nodes.len(), original_count - num_sgs);
            }
        }
    }

    /// Test edge cleanup invariants
    #[test]
    fn test_edge_cleanup_invariants() {
        let configs = vec![
            (true, true, true),
            (true, true, false),
            (true, false, true),
            (false, true, true),
            (true, false, false),
            (false, false, true),
        ];

        for (has_from, has_to, has_child) in configs {
            let mut doc = DiagramDocument::default();

            let sg_id = NodeId::new("sg".to_string());
            doc.document.nodes.insert(sg_id.clone(), test_subgraph());

            let child1 = NodeId::new("c1".to_string());
            let mut c1 = test_node(10.0, 10.0, 20.0, 20.0);
            c1.parent = Some(sg_id.clone());
            doc.document.nodes.insert(child1.clone(), c1);

            let child2 = NodeId::new("c2".to_string());
            let mut c2 = test_node(40.0, 40.0, 20.0, 20.0);
            c2.parent = Some(sg_id.clone());
            doc.document.nodes.insert(child2.clone(), c2);

            let ext = NodeId::new("ext".to_string());
            doc.document
                .nodes
                .insert(ext.clone(), test_node(200.0, 200.0, 20.0, 20.0));

            if has_from {
                let eid = diagram_models::document::EdgeId::new("e_from".to_string());
                let edge = Edge {
                    source: sg_id.clone(),
                    target: ext.clone(),
                    label: String::new(),
                    style: diagram_models::document::EdgeStyle::default(),
                    arrow_type: diagram_models::document::ArrowType::default(),
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
                };
                doc.document.edges.insert(eid, edge);
            }

            if has_to {
                let eid = diagram_models::document::EdgeId::new("e_to".to_string());
                let edge = Edge {
                    source: ext.clone(),
                    target: sg_id.clone(),
                    label: String::new(),
                    style: diagram_models::document::EdgeStyle::default(),
                    arrow_type: diagram_models::document::ArrowType::default(),
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
                };
                doc.document.edges.insert(eid, edge);
            }

            if has_child {
                let eid = diagram_models::document::EdgeId::new("e_child".to_string());
                let edge = Edge {
                    source: child1.clone(),
                    target: child2.clone(),
                    label: String::new(),
                    style: diagram_models::document::EdgeStyle::default(),
                    arrow_type: diagram_models::document::ArrowType::default(),
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
                };
                doc.document.edges.insert(eid, edge);
            }

            doc.editor_state
                .selected_items
                .insert(sg_id.as_str().to_string());
            let _ = ungroup_selection(&mut doc);

            assert!(
                check_no_orphan_edges(&doc),
                "INV2 for {:?}",
                (has_from, has_to, has_child)
            );

            if has_from || has_to {
                assert!(
                    !doc.document
                        .edges
                        .values()
                        .any(|e| e.source == sg_id || e.target == sg_id),
                    "Edges to/from deleted subgraph should be removed"
                );
            }
        }
    }

    // =====================================================================
    // Fuzzing Tests with Rand
    // =====================================================================

    use rand::prelude::*;
    use rand::rngs::StdRng;
    use rand::Rng;

    /// Fuzz test: random graph structures with ungroup
    #[test]
    fn fuzz_random_graph_ungroup_invariants() {
        for seed in 0..100u32 {
            let mut rng = StdRng::seed_from_u64(u64::from(seed));
            let mut doc = DiagramDocument::default();

            let num_nodes = 5 + (rng.gen_range(0..16)) as usize;
            let mut node_ids: Vec<NodeId> = Vec::new();

            for i in 0..num_nodes {
                let id = NodeId::new(format!("n{i}"));
                let x = rng.gen_range(0.0..500.0);
                let y = rng.gen_range(0.0..500.0);
                let w = rng.gen_range(10.0..110.0);
                let h = rng.gen_range(10.0..110.0);
                doc.document.nodes.insert(id.clone(), test_node(x, y, w, h));
                node_ids.push(id);
            }

            let num_sgs = 1 + (rng.gen_range(0..3)) as usize;
            let mut sgs: Vec<NodeId> = Vec::new();

            for i in 0..num_sgs {
                let id = NodeId::new(format!("sg{i}"));
                let mut sg = test_subgraph();
                sg.x = OrderedFloat(rng.gen_range(0.0..400.0));
                sg.y = OrderedFloat(rng.gen_range(0.0..400.0));
                sg.width = OrderedFloat(rng.gen_range(50.0..250.0));
                sg.height = OrderedFloat(rng.gen_range(50.0..250.0));

                if !sgs.is_empty() && rng.gen_bool(0.5) {
                    let parent_idx = rng.gen_range(0..sgs.len());
                    sg.parent = Some(sgs[parent_idx].clone());
                }

                doc.document.nodes.insert(id.clone(), sg);
                sgs.push(id);
            }

            // Assign children
            for (i, id) in node_ids.iter().enumerate() {
                if !sgs.is_empty() && i % 3 != 0 {
                    let parent_idx = i % sgs.len();
                    if let Some(node) = doc.document.nodes.get_mut(id) {
                        node.parent = Some(sgs[parent_idx].clone());
                    }
                }
            }

            // Add random edges
            for i in 0..rng.gen_range(0..10) {
                if node_ids.len() >= 2 {
                    let src = rng.gen_range(0..node_ids.len());
                    let dst = rng.gen_range(0..node_ids.len());
                    if src != dst {
                        let eid = diagram_models::document::EdgeId::new(format!("e{i}"));
                        let edge = Edge {
                            source: node_ids[src].clone(),
                            target: node_ids[dst].clone(),
                            label: String::new(),
                            style: diagram_models::document::EdgeStyle::default(),
                            arrow_type: diagram_models::document::ArrowType::default(),
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
                        };
                        doc.document.edges.insert(eid, edge);
                    }
                }
            }

            // Ungroup random subgraph
            if !sgs.is_empty() {
                let sel_idx = rng.gen_range(0..sgs.len());
                doc.editor_state
                    .selected_items
                    .insert(sgs[sel_idx].as_str().to_string());

                let original_count = doc.document.nodes.len();
                let selected_sg = sgs[sel_idx].clone();

                let _ = ungroup_selection(&mut doc);

                assert!(
                    check_parent_chain_validity(&doc),
                    "INV1 violated seed {seed}"
                );
                assert!(check_no_orphan_edges(&doc), "INV2 violated seed {seed}");
                assert!(
                    check_node_count_consistency(original_count, 1, &doc),
                    "INV3 violated seed {seed}"
                );

                // Verify subgraph was deleted
                assert!(!doc.document.nodes.contains_key(&selected_sg));
            }
        }
    }

    /// Fuzz test: sequential group/ungroup operations
    /// SUB-003: LCA parent assignment
    #[test]
    fn test_sub003_mixed_parent_grouping_reparents_to_common_ancestor() {
        let mut doc = DiagramDocument::default();
        let s1_id = NodeId::new("s1".to_string());
        doc.document.nodes.insert(s1_id.clone(), test_subgraph());

        let n1_id = NodeId::new("n1".to_string());
        let mut n1 = test_node(10.0, 10.0, 20.0, 20.0);
        n1.parent = Some(s1_id.clone());
        doc.document.nodes.insert(n1_id.clone(), n1);

        let n2_id = NodeId::new("n2".to_string());
        let mut n2 = test_node(40.0, 40.0, 20.0, 20.0);
        n2.parent = Some(s1_id.clone());
        doc.document.nodes.insert(n2_id.clone(), n2);

        doc.editor_state
            .selected_items
            .insert(n1_id.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2_id.as_str().to_string());

        let group_id = NodeId::new("g1".to_string());
        group_selection(&mut doc, &group_id).unwrap();

        let group = doc.document.nodes.get(&group_id).unwrap();
        assert_eq!(group.parent, Some(s1_id));
        assert_eq!(
            doc.document.nodes.get(&n1_id).unwrap().parent,
            Some(group_id.clone())
        );
        assert_eq!(
            doc.document.nodes.get(&n2_id).unwrap().parent,
            Some(group_id)
        );
    }

    /// SUB-006: Z-index consistency
    #[test]
    fn test_sub006_z_index_is_min_of_children_minus_one() {
        let mut doc = DiagramDocument::default();
        let n1_id = NodeId::new("n1".to_string());
        let mut n1 = test_node(0.0, 0.0, 10.0, 10.0);
        n1.z_index = 100;
        doc.document.nodes.insert(n1_id.clone(), n1);

        let n2_id = NodeId::new("n2".to_string());
        let mut n2 = test_node(20.0, 20.0, 10.0, 10.0);
        n2.z_index = 50;
        doc.document.nodes.insert(n2_id.clone(), n2);

        doc.editor_state
            .selected_items
            .insert(n1_id.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2_id.as_str().to_string());

        let group_id = NodeId::new("g1".to_string());
        group_selection(&mut doc, &group_id).unwrap();

        let group = doc.document.nodes.get(&group_id).unwrap();
        assert_eq!(group.z_index, 49);
    }

    /// SUB-002: Error on locked nodes (returns all)
    #[test]
    fn test_err_locked_node_returns_all_locked_ids() {
        let mut doc = DiagramDocument::default();
        let n1_id = NodeId::new("n1".to_string());
        let mut n1 = test_node(0.0, 0.0, 10.0, 10.0);
        n1.lock_state = LockState::Locked;
        doc.document.nodes.insert(n1_id.clone(), n1);

        let n2_id = NodeId::new("n2".to_string());
        let mut n2 = test_node(20.0, 20.0, 10.0, 10.0);
        n2.lock_state = LockState::Locked;
        doc.document.nodes.insert(n2_id.clone(), n2);

        doc.editor_state
            .selected_items
            .insert(n1_id.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2_id.as_str().to_string());

        let group_id = NodeId::new("g1".to_string());
        let result = group_selection(&mut doc, &group_id);

        match result {
            Err(GroupingError::LockedNode(ids)) => {
                assert!(ids.contains(&n1_id));
                assert!(ids.contains(&n2_id));
                assert_eq!(ids.len(), 2);
            }
            _ => panic!("Expected LockedNode error with multiple IDs"),
        }
    }

    /// P2: Error on node not found
    #[test]
    fn test_err_node_not_found_returns_error() {
        let mut doc = DiagramDocument::default();
        doc.editor_state
            .selected_items
            .insert("missing".to_string());

        let group_id = NodeId::new("g1".to_string());
        let result = group_selection(&mut doc, &group_id);
        assert_eq!(
            result,
            Err(GroupingError::NodeNotFound(NodeId::new(
                "missing".to_string()
            )))
        );
    }

    /// P5: Error on nesting limit
    #[test]
    fn test_err_nesting_depth_exceeded_returns_error() {
        let mut doc = DiagramDocument::default();
        let mut last_id = None;
        for i in 0..5 {
            let id = NodeId::new(format!("s{i}"));
            let mut s = test_subgraph();
            s.parent = last_id;
            doc.document.nodes.insert(id.clone(), s);
            last_id = Some(id);
        }

        let n_id = NodeId::new("n".to_string());
        let mut n = test_node(0.0, 0.0, 10.0, 10.0);
        n.parent = last_id;
        doc.document.nodes.insert(n_id.clone(), n);

        doc.editor_state
            .selected_items
            .insert(n_id.as_str().to_string());

        let group_id = NodeId::new("g1".to_string());
        let result = group_selection(&mut doc, &group_id);
        assert_eq!(result, Err(GroupingError::NestedSubgraphLimitExceeded(5)));
    }

    /// Fuzz test: sequential group/ungroup operations
    #[test]
    fn fuzz_sequential_group_ungroup_invariants() {
        for seed in 0..50u32 {
            let mut rng = StdRng::seed_from_u64(u64::from(seed));
            let mut doc = DiagramDocument::default();

            let num_nodes = 5 + (rng.gen_range(0..10)) as usize;
            let mut node_ids: Vec<NodeId> = Vec::new();

            for i in 0..num_nodes {
                let id = NodeId::new(format!("n{i}"));
                let x = rng.gen_range(0.0..500.0);
                let y = rng.gen_range(0.0..500.0);
                doc.document
                    .nodes
                    .insert(id.clone(), test_node(x, y, 50.0, 50.0));
                node_ids.push(id);
            }

            let operations = 3 + (rng.gen_range(0..5)) as usize;

            for op in 0..operations {
                doc.editor_state.selected_items.clear();

                // Random subset
                let subset_size = 1 + (rng.gen_range(0..node_ids.len()));
                let mut indices: Vec<usize> = (0..node_ids.len()).collect();
                indices.shuffle(&mut rng);
                let indices: Vec<usize> = indices.into_iter().take(subset_size).collect();

                for idx in &indices {
                    if let Some(id) = node_ids.get(*idx) {
                        doc.editor_state
                            .selected_items
                            .insert(id.as_str().to_string());
                    }
                }

                if op % 2 == 0 {
                    let group_id = NodeId::new(format!("group-{op}"));
                    let _ = group_selection(&mut doc, &group_id);
                } else {
                    let _ = ungroup_selection(&mut doc);
                }

                assert!(
                    check_parent_chain_validity(&doc),
                    "INV1 at op {op} seed {seed}"
                );
                assert!(check_no_orphan_edges(&doc), "INV2 at op {op} seed {seed}");
            }
        }
    }
}
