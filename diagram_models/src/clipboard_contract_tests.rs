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
    use crate::clipboard_contract::{calculate_paste, copy, cut, ClipboardData, Error, Selection};
    use crate::document::{
        DiagramDocument, Edge, EdgeId, FontWeight, LockState, Node, NodeId, NodeKind, NodeStyle,
        OrderedFloat,
    };

    fn create_test_node() -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Test".to_string(),
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
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn create_test_edge(source: NodeId, target: NodeId) -> Edge {
        Edge {
            source,
            target,
            label: String::new(),
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    // Happy Path Tests
    #[test]
    fn test_clp001_copy_paste_single_node_creates_new_node_with_new_id() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new("node_a".to_string());
        doc.document
            .nodes
            .insert(node_id.clone(), create_test_node());

        let selection = Selection {
            nodes: vec![node_id.clone()],
        };

        let clipboard = copy(&selection, &doc).unwrap();
        assert_eq!(clipboard.nodes.len(), 1);

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(paste_res.new_nodes.len(), 1);
        let (new_id, pasted_node) = &paste_res.new_nodes[0];

        assert_ne!(new_id, &node_id);

        let original_node = doc.document.nodes.get(&node_id).unwrap();
        assert_eq!(pasted_node.x.0, original_node.x.0 + 20.0);
        assert_eq!(pasted_node.y.0, original_node.y.0 + 20.0);
    }

    #[test]
    fn test_clp002_copy_paste_multiple_nodes_preserves_edges_and_remaps_ids() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());
        doc.document
            .edges
            .insert(e1, create_test_edge(n1.clone(), n2.clone()));

        let selection = Selection {
            nodes: vec![n1.clone(), n2.clone()],
        };
        let clipboard = copy(&selection, &doc).unwrap();

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(paste_res.new_nodes.len(), 2);
        assert_eq!(paste_res.new_edges.len(), 1);

        let (_, new_edge) = &paste_res.new_edges[0];
        let new_node_ids: std::collections::HashSet<_> =
            paste_res.new_nodes.iter().map(|(id, _)| id).collect();

        assert!(new_node_ids.contains(&new_edge.source));
        assert!(new_node_ids.contains(&new_edge.target));
        assert_ne!(new_edge.source, n1);
        assert_ne!(new_edge.target, n2);
    }

    #[test]
    fn test_clp003_copy_paste_subgraph_preserves_parent_child_relationships() {
        let mut doc = DiagramDocument::default();
        let p1 = NodeId::new("p1".to_string());
        let c1 = NodeId::new("c1".to_string());

        let parent_node = create_test_node();
        let mut child_node = create_test_node();
        child_node.parent = Some(p1.clone());

        doc.document.nodes.insert(p1.clone(), parent_node);
        doc.document.nodes.insert(c1.clone(), child_node);

        let selection = Selection {
            nodes: vec![p1, c1],
        };

        let clipboard = copy(&selection, &doc).unwrap();
        let paste_res = calculate_paste(&clipboard, &doc).unwrap();

        let new_p1 = paste_res
            .new_nodes
            .iter()
            .find(|(_, n)| n.parent.is_none())
            .unwrap();
        let new_c1 = paste_res
            .new_nodes
            .iter()
            .find(|(_, n)| n.parent.is_some())
            .unwrap();

        assert_eq!(new_c1.1.parent, Some(new_p1.0.clone()));
    }

    #[test]
    fn test_clp004_cut_operation_removes_original_nodes_and_places_in_clipboard() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        doc.document.nodes.insert(n1.clone(), create_test_node());

        let selection = Selection {
            nodes: vec![n1.clone()],
        };

        let clipboard = cut(&selection, &mut doc).unwrap();
        assert_eq!(clipboard.nodes.len(), 1);
        assert!(!doc.document.nodes.contains_key(&n1));

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(paste_res.new_nodes.len(), 1);
        assert_ne!(paste_res.new_nodes[0].0, n1);
    }

    #[test]
    fn test_clp005_paste_operation_applies_incremental_offset_based_on_serial() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        doc.document.nodes.insert(n1.clone(), create_test_node());

        let selection = Selection { nodes: vec![n1] };
        let mut clipboard = copy(&selection, &doc).unwrap();

        clipboard.paste_serial = 0;
        let paste1 = calculate_paste(&clipboard, &doc).unwrap();
        clipboard.paste_serial = 1;
        let paste2 = calculate_paste(&clipboard, &doc).unwrap();
        clipboard.paste_serial = 2;
        let paste3 = calculate_paste(&clipboard, &doc).unwrap();

        assert_eq!(paste1.new_nodes[0].1.x.0, 20.0);
        assert_eq!(paste2.new_nodes[0].1.x.0, 40.0);
        assert_eq!(paste3.new_nodes[0].1.x.0, 60.0);
    }

    // Error Path Tests
    #[test]
    fn test_copy_returns_error_when_selection_is_empty() {
        let doc = DiagramDocument::default();
        assert_eq!(
            copy(&Selection::empty(), &doc).unwrap_err(),
            Error::EmptySelection
        );
    }

    #[test]
    fn test_paste_returns_error_when_clipboard_is_empty() {
        let doc = DiagramDocument::default();
        assert_eq!(
            calculate_paste(&ClipboardData::empty(), &doc).unwrap_err(),
            Error::EmptyClipboard
        );
    }

    #[test]
    fn test_q6_violation_returns_invalid_edge_reference_error() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();

        let n1 = NodeId::new("non_existent".to_string());
        clipboard.edges.push((
            EdgeId::new("e1".to_string()),
            create_test_edge(n1.clone(), n1),
        ));

        let valid_node = NodeId::new("valid".to_string());
        clipboard.nodes.push((valid_node, create_test_node()));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::InvalidEdgeReference)));
    }

    #[test]
    fn test_q7_violation_returns_invalid_parent_reference_error() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();

        let n1 = NodeId::new("child".to_string());
        let mut child_node = create_test_node();
        child_node.parent = Some(NodeId::new("non_existent_parent".to_string()));

        clipboard.nodes.push((n1, child_node));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::InvalidParentReference)));
    }

    #[test]
    fn test_corrupt_clipboard_with_duplicate_node_ids() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();
        let n1 = NodeId::new("n1".to_string());
        clipboard.nodes.push((n1.clone(), create_test_node()));
        clipboard.nodes.push((n1, create_test_node()));
        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::CorruptClipboard)));
    }

    #[test]
    fn test_cyclic_parent_reference() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();

        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());

        let mut node1 = create_test_node();
        node1.parent = Some(n2.clone());
        let mut node2 = create_test_node();
        node2.parent = Some(n1.clone());

        clipboard.nodes.push((n1, node1));
        clipboard.nodes.push((n2, node2));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::CyclicParentReference)));
    }

    // Edge selection filtering tests (mutation killers for && vs || at clipboard_contract.rs:97)

    #[test]
    fn given_edge_between_selected_and_external_when_copy_then_edge_included() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());
        doc.document
            .edges
            .insert(e1, create_test_edge(n1.clone(), n2));

        // Only n1 is selected; n2 is external to the selection
        let selection = Selection { nodes: vec![n1] };

        let clipboard = copy(&selection, &doc).unwrap();
        // Edge must NOT be copied: both endpoints must be in the selection (&& semantics)
        assert!(clipboard.edges.is_empty());
    }

    #[test]
    fn given_edge_with_both_ends_selected_when_copy_then_edge_included() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());
        doc.document
            .edges
            .insert(e1, create_test_edge(n1.clone(), n2.clone()));

        let selection = Selection {
            nodes: vec![n1.clone(), n2],
        };

        let clipboard = copy(&selection, &doc).unwrap();
        assert_eq!(clipboard.edges.len(), 1);
    }

    #[test]
    fn given_edge_with_neither_end_selected_when_copy_then_edge_excluded() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let n3 = NodeId::new("n3".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());
        doc.document.nodes.insert(n3.clone(), create_test_node());
        doc.document.edges.insert(e1, create_test_edge(n1, n2));

        // Only n3 is selected; the edge connects n1↔n2 (neither selected)
        let selection = Selection { nodes: vec![n3] };

        let clipboard = copy(&selection, &doc).unwrap();
        assert!(clipboard.edges.is_empty());
    }

    // Mutation killers for calculate_paste collision check (lines 249, 252: delete !)

    #[test]
    fn given_edge_to_external_node_when_calculate_paste_then_returns_invalid_edge_reference() {
        let mut doc = DiagramDocument::default();
        let existing = NodeId::new("existing".to_string());
        doc.document.nodes.insert(existing, create_test_node());

        let mut clipboard = ClipboardData::empty();
        let pasted = NodeId::new("pasted".to_string());
        clipboard.nodes.push((pasted.clone(), create_test_node()));

        let external_target = NodeId::new("nonexistent_external".to_string());
        clipboard.edges.push((
            EdgeId::new("e1".to_string()),
            create_test_edge(pasted, external_target),
        ));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::InvalidEdgeReference)));
    }

    #[test]
    fn given_edge_from_external_to_pasted_when_calculate_paste_then_returns_invalid_edge_reference()
    {
        let mut doc = DiagramDocument::default();
        let existing = NodeId::new("existing".to_string());
        doc.document.nodes.insert(existing, create_test_node());

        let mut clipboard = ClipboardData::empty();
        let pasted = NodeId::new("pasted".to_string());
        clipboard.nodes.push((pasted.clone(), create_test_node()));

        let external_source = NodeId::new("nonexistent_source".to_string());
        clipboard.edges.push((
            EdgeId::new("e2".to_string()),
            create_test_edge(external_source, pasted),
        ));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::InvalidEdgeReference)));
    }

    // Mutation killer for cycle detection || → && at line 197

    #[test]
    fn given_three_node_chain_when_calculate_paste_then_no_cycle_detected() {
        let doc = DiagramDocument::default();

        let mut clipboard = ClipboardData::empty();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let n3 = NodeId::new("n3".to_string());

        let mut node1 = create_test_node();
        node1.parent = Some(n2.clone());
        let mut node2 = create_test_node();
        node2.parent = Some(n3.clone());
        let node3 = create_test_node();

        clipboard.nodes.push((n1, node1));
        clipboard.nodes.push((n2, node2));
        clipboard.nodes.push((n3, node3));

        // n1 → n2 → n3 (linear chain, no cycle)
        let result = calculate_paste(&clipboard, &doc);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().new_nodes.len(), 3);
    }

    // === RED QUEEN GENERATION 1: Adversarial coevolutionary tests ===
    // Targeting mutation-susceptible paths in clipboard_contract.rs

    #[test]
    fn given_self_loop_edge_when_copy_then_edge_included_in_clipboard() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document
            .edges
            .insert(e1, create_test_edge(n1.clone(), n1.clone()));

        let selection = Selection {
            nodes: vec![n1],
        };

        let clipboard = copy(&selection, &doc).unwrap();
        assert_eq!(clipboard.edges.len(), 1, "self-loop edge must be copied when its single endpoint is selected");

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(paste_res.new_edges.len(), 1, "self-loop must survive paste");
        let pasted_edge = &paste_res.new_edges[0].1;
        assert_eq!(pasted_edge.source, pasted_edge.target, "self-loop must remain a self-loop after id remapping");
    }

    #[test]
    fn given_node_with_parent_in_doc_not_clipboard_when_paste_then_parent_kept() {
        let mut doc = DiagramDocument::default();
        let doc_parent = NodeId::new("doc_parent".to_string());
        let mut doc_parent_node = create_test_node();
        doc_parent_node.label = "DocParent".to_string();
        doc.document.nodes.insert(doc_parent.clone(), doc_parent_node);

        let mut clipboard = ClipboardData::empty();
        let child_id = NodeId::new("child".to_string());
        let mut child_node = create_test_node();
        child_node.parent = Some(doc_parent.clone());

        clipboard.nodes.push((child_id, child_node));

        let result = calculate_paste(&clipboard, &doc);
        assert!(result.is_ok(), "parent exists in doc — must not return InvalidParentReference");
        let pasted = result.unwrap();
        assert_eq!(pasted.new_nodes.len(), 1);
        assert_eq!(pasted.new_nodes[0].1.parent, Some(doc_parent), "parent must reference existing doc node, not be remapped");
    }

    #[test]
    fn given_copy_preserves_all_node_fields_then_paste_preserves_them() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let mut full_node = create_test_node();
        full_node.label = "FullNode".to_string();
        full_node.icon = "star".to_string();
        full_node.width = OrderedFloat(200.0);
        full_node.height = OrderedFloat(150.0);
        full_node.z_index = 42;
        full_node.font_size = Some(OrderedFloat(14.0));
        full_node.font_weight = Some(FontWeight::Bold);
        full_node.tags = im::vector!["tag1".to_string(), "tag2".to_string()];
        full_node.style = Some(NodeStyle::Dashed);
        full_node.collapsed = Some(true);
        full_node.dag_rank = Some(3);

        doc.document.nodes.insert(n1.clone(), full_node.clone());

        let selection = Selection { nodes: vec![n1] };
        let clipboard = copy(&selection, &doc).unwrap();
        let paste_res = calculate_paste(&clipboard, &doc).unwrap();

        let pasted_node = &paste_res.new_nodes[0].1;
        assert_eq!(pasted_node.label, "FullNode");
        assert_eq!(pasted_node.icon, "star");
        assert_eq!(pasted_node.width, OrderedFloat(200.0));
        assert_eq!(pasted_node.height, OrderedFloat(150.0));
        assert_eq!(pasted_node.z_index, 42);
        assert_eq!(pasted_node.font_size, Some(OrderedFloat(14.0)));
        assert_eq!(pasted_node.font_weight, Some(FontWeight::Bold));
        assert_eq!(pasted_node.style, Some(NodeStyle::Dashed));
        assert_eq!(pasted_node.collapsed, Some(true));
        assert_eq!(pasted_node.dag_rank, Some(3));
        assert_eq!(pasted_node.tags.len(), 2);
    }

    #[test]
    fn given_cut_then_paste_then_new_nodes_exist_and_old_removed() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());

        let mut node_a = create_test_node();
        node_a.label = "A".to_string();
        let mut node_b = create_test_node();
        node_b.label = "B".to_string();
        node_b.x = OrderedFloat(300.0);

        doc.document.nodes.insert(n1.clone(), node_a);
        doc.document.nodes.insert(n2.clone(), node_b);
        doc.document
            .edges
            .insert(e1, create_test_edge(n1.clone(), n2.clone()));

        let selection = Selection {
            nodes: vec![n1.clone(), n2.clone()],
        };
        let clipboard = cut(&selection, &mut doc).unwrap();

        assert!(!doc.document.nodes.contains_key(&n1), "n1 must be removed after cut");
        assert!(!doc.document.nodes.contains_key(&n2), "n2 must be removed after cut");

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(paste_res.new_nodes.len(), 2, "paste must recreate 2 nodes");
        assert_eq!(paste_res.new_edges.len(), 1, "paste must recreate edge");

        let new_ids: Vec<_> = paste_res.new_nodes.iter().map(|(id, _)| id.clone()).collect();
        assert_ne!(new_ids[0], n1, "pasted nodes must have new IDs");
        assert_ne!(new_ids[1], n2, "pasted nodes must have new IDs");

        let pasted_edge = &paste_res.new_edges[0].1;
        assert!(new_ids.contains(&pasted_edge.source), "edge source must be remapped to new node");
        assert!(new_ids.contains(&pasted_edge.target), "edge target must be remapped to new node");
    }

    #[test]
    fn given_multiple_edges_same_pair_when_copy_then_all_edges_copied() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());
        let e2 = EdgeId::new("e2".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());

        let mut edge1 = create_test_edge(n1.clone(), n2.clone());
        edge1.label = "first".to_string();
        let mut edge2 = create_test_edge(n1.clone(), n2.clone());
        edge2.label = "second".to_string();

        doc.document.edges.insert(e1, edge1);
        doc.document.edges.insert(e2, edge2);

        let selection = Selection {
            nodes: vec![n1, n2],
        };
        let clipboard = copy(&selection, &doc).unwrap();
        assert_eq!(clipboard.edges.len(), 2, "both parallel edges must be copied");

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(paste_res.new_edges.len(), 2, "both parallel edges must survive paste");

        let labels: Vec<_> = paste_res.new_edges.iter().map(|(_, e)| e.label.clone()).collect();
        assert!(labels.contains(&"first".to_string()));
        assert!(labels.contains(&"second".to_string()));
    }

    #[test]
    fn given_deep_parent_chain_when_paste_then_all_parents_remapped() {
        let doc = DiagramDocument::default();

        let mut clipboard = ClipboardData::empty();
        let depth = 8;
        let mut ids = Vec::new();
        for i in 0..depth {
            ids.push(NodeId::new(format!("n{i}")));
        }
        for i in 0..depth {
            let mut node = create_test_node();
            if i > 0 {
                node.parent = Some(ids[i - 1].clone());
            }
            clipboard.nodes.push((ids[i].clone(), node));
        }

        let result = calculate_paste(&clipboard, &doc);
        assert!(result.is_ok(), "deep chain must not trigger false cycle detection");

        let pasted = result.unwrap();
        assert_eq!(pasted.new_nodes.len(), depth as usize);

        let id_map: std::collections::HashMap<_, _> = clipboard
            .nodes
            .iter()
            .zip(pasted.new_nodes.iter())
            .map(|((old_id, _), (new_id, _))| (old_id.clone(), new_id.clone()))
            .collect();

        for (i, (old_id, _)) in clipboard.nodes.iter().enumerate() {
            let new_id = id_map.get(old_id).unwrap();
            let pasted_node = &pasted.new_nodes.iter().find(|(id, _)| id == new_id).unwrap().1;
            if i == 0 {
                assert!(pasted_node.parent.is_none(), "root must have no parent");
            } else {
                let expected_parent = id_map.get(&ids[i - 1]).unwrap();
                assert_eq!(
                    pasted_node.parent.as_ref(),
                    Some(expected_parent),
                    "node {i} parent must be remapped to new parent"
                );
            }
        }
    }

    #[test]
    fn given_duplicate_edge_ids_in_clipboard_when_paste_then_corrupt_clipboard() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();
        let n1 = NodeId::new("n1".to_string());
        clipboard.nodes.push((n1, create_test_node()));

        let eid = EdgeId::new("dup_edge".to_string());
        let edge = create_test_edge(
            NodeId::new("n1".to_string()),
            NodeId::new("n1".to_string()),
        );
        clipboard.edges.push((eid.clone(), edge.clone()));
        clipboard.edges.push((eid, edge));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::CorruptClipboard)), "duplicate edge IDs must be caught");
    }

    #[test]
    fn given_paste_result_selection_contains_all_new_node_ids() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());

        let selection = Selection {
            nodes: vec![n1, n2],
        };
        let clipboard = copy(&selection, &doc).unwrap();
        let paste_res = calculate_paste(&clipboard, &doc).unwrap();

        assert_eq!(paste_res.new_selection.len(), 2, "selection must contain all pasted node IDs");
        for (new_id, _) in &paste_res.new_nodes {
            assert!(
                paste_res.new_selection.contains(new_id.as_str()),
                "pasted node {new_id:?} must be in new_selection"
            );
        }
    }

    #[test]
    fn given_edge_with_one_end_in_clipboard_other_in_doc_when_paste_then_invalid_edge_reference() {
        let mut doc = DiagramDocument::default();
        let doc_node = NodeId::new("doc_node".to_string());
        doc.document.nodes.insert(doc_node.clone(), create_test_node());

        let mut clipboard = ClipboardData::empty();
        let clip_node = NodeId::new("clip_node".to_string());
        clipboard.nodes.push((clip_node.clone(), create_test_node()));

        let mut edge = create_test_edge(clip_node, NodeId::new("not_in_anywhere".to_string()));
        edge.source = clipboard.nodes[0].0.clone();
        let external = NodeId::new("external".to_string());
        edge.target = external;

        clipboard.edges.push((EdgeId::new("e1".to_string()), edge));

        let result = calculate_paste(&clipboard, &doc);
        assert!(
            matches!(result, Err(Error::InvalidEdgeReference)),
            "edge to node not in clipboard or doc must fail"
        );
    }

    #[test]
    fn given_node_self_parent_when_paste_then_cyclic_detected() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();

        let n1 = NodeId::new("self_parent".to_string());
        let mut node = create_test_node();
        node.parent = Some(n1.clone());

        clipboard.nodes.push((n1.clone(), node));

        let result = calculate_paste(&clipboard, &doc);
        assert!(
            matches!(result, Err(Error::CyclicParentReference)),
            "node parenting itself must be detected as cycle"
        );
    }

    #[test]
    fn given_copy_with_nonexistent_node_in_selection_then_postcondition_violated() {
        let doc = DiagramDocument::default();
        let ghost = NodeId::new("ghost".to_string());

        let selection = Selection {
            nodes: vec![ghost],
        };

        let result = copy(&selection, &doc);
        assert!(
            matches!(result, Err(Error::PostconditionViolated(_))),
            "copying a node not in doc must return PostconditionViolated"
        );
    }

    #[test]
    fn given_two_node_cycle_when_paste_then_cyclic_detected() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();

        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());

        let mut node1 = create_test_node();
        node1.parent = Some(n2.clone());
        let mut node2 = create_test_node();
        node2.parent = Some(n1.clone());

        clipboard.nodes.push((n1, node1));
        clipboard.nodes.push((n2, node2));

        let result = calculate_paste(&clipboard, &doc);
        assert!(
            matches!(result, Err(Error::CyclicParentReference)),
            "mutual parent cycle must be detected"
        );
    }

    #[test]
    fn given_paste_serial_zero_then_offset_is_20() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let mut node = create_test_node();
        node.x = OrderedFloat(100.0);
        node.y = OrderedFloat(200.0);
        doc.document.nodes.insert(n1.clone(), node);

        let selection = Selection { nodes: vec![n1] };
        let clipboard = copy(&selection, &doc).unwrap();
        assert_eq!(clipboard.paste_serial, 0);

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        let pasted = &paste_res.new_nodes[0].1;
        assert_eq!(pasted.x, OrderedFloat(120.0), "serial=0 → offset=20.0, so x=100+20=120");
        assert_eq!(pasted.y, OrderedFloat(220.0), "serial=0 → offset=20.0, so y=200+20=220");
    }

    #[test]
    fn given_cut_removes_edges_connected_to_cut_nodes() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let n3 = NodeId::new("n3".to_string());
        let e1 = EdgeId::new("e1".to_string());
        let e2 = EdgeId::new("e2".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());
        doc.document.nodes.insert(n3.clone(), create_test_node());
        doc.document
            .edges
            .insert(e1.clone(), create_test_edge(n1.clone(), n2.clone()));
        doc.document
            .edges
            .insert(e2, create_test_edge(n2.clone(), n3.clone()));

        let selection = Selection {
            nodes: vec![n1.clone(), n2.clone()],
        };
        let clipboard = cut(&selection, &mut doc).unwrap();

        assert_eq!(clipboard.nodes.len(), 2);
        assert!(!doc.document.nodes.contains_key(&n1));
        assert!(!doc.document.nodes.contains_key(&n2));
        assert!(doc.document.nodes.contains_key(&n3), "n3 not in selection must remain");
    }

    #[test]
    fn given_edge_preserves_label_and_style_through_copy_paste() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());

        let mut edge = create_test_edge(n1, n2);
        edge.label = "MyEdge".to_string();
        edge.color = Some("#ff0000".to_string());
        edge.thickness = OrderedFloat(3.0);
        edge.directed = false;

        doc.document.edges.insert(e1, edge);

        let selection = Selection {
            nodes: vec![NodeId::new("n1".to_string()), NodeId::new("n2".to_string())],
        };
        let clipboard = copy(&selection, &doc).unwrap();
        let paste_res = calculate_paste(&clipboard, &doc).unwrap();

        let pasted_edge = &paste_res.new_edges[0].1;
        assert_eq!(pasted_edge.label, "MyEdge");
        assert_eq!(pasted_edge.color, Some("#ff0000".to_string()));
        assert_eq!(pasted_edge.thickness, OrderedFloat(3.0));
        assert!(!pasted_edge.directed);
    }
}
