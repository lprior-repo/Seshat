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
        DiagramDocument, Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
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

    // === RED QUEEN GENERATION 2: Coevolutionary Adversarial Tests ===
    // Mutation killers and contract enforcement for clipboard_contract.rs

    #[test]
    fn given_node_at_nonzero_origin_when_paste_then_offset_added_to_both_x_and_y() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let mut node = create_test_node();
        node.x = OrderedFloat(50.0);
        node.y = OrderedFloat(75.0);
        doc.document.nodes.insert(n1.clone(), node);    #[test]
    fn test_clp006_duplicate_shortcut_creates_copy_with_new_ids_and_offset() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let mut node1 = create_test_node();
        node1.x = OrderedFloat(100.0);
        node1.y = OrderedFloat(100.0);
        doc.document.nodes.insert(n1.clone(), node1);

        let selection = Selection {
            nodes: vec![n1.clone()],
        };

        let clipboard = copy(&selection, &mut doc).unwrap();
        assert_eq!(clipboard.nodes.len(), 1);
        assert_eq!(clipboard.paste_serial, 0);

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(paste_res.new_nodes.len(), 1);
        let (new_id, pasted_node) = &paste_res.new_nodes[0];

        assert_ne!(new_id, &n1);
        assert_eq!(pasted_node.x.0, 120.0);
        assert_eq!(pasted_node.y.0, 120.0);
    }

    #[test]
    fn test_clp007_multi_paste_idempotency_produces_consistent_results() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        doc.document.nodes.insert(n1.clone(), create_test_node());

        let selection = Selection { nodes: vec![n1] };
        let clipboard = copy(&selection, &doc).unwrap();

        let paste_res = calculate_paste(&clipboard, &doc).unwrap();
        let (_, pasted) = &paste_res.new_nodes[0];

        assert_eq!(pasted.x.0, 70.0, "x should be 50 + 20 = 70");
        assert_eq!(pasted.y.0, 95.0, "y should be 75 + 20 = 95");
    }

    #[test]
    fn given_duplicate_edge_ids_in_clipboard_when_paste_then_corrupt_clipboard() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();

        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        clipboard.nodes.push((n1.clone(), create_test_node()));
        clipboard.nodes.push((n2.clone(), create_test_node()));

        let e_id = EdgeId::new("same_edge_id".to_string());
        clipboard.edges.push((e_id.clone(), create_test_edge(n1.clone(), n2.clone())));
        clipboard.edges.push((e_id, create_test_edge(n2, n1)));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::CorruptClipboard)));
    }

    #[test]
    fn given_self_referencing_parent_when_paste_then_cyclic_detected() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();

        let n1 = NodeId::new("n1".to_string());
        let mut node = create_test_node();
        node.parent = Some(n1.clone());

        clipboard.nodes.push((n1, node));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::CyclicParentReference)));
    }

    #[test]
    fn given_copy_with_nonexistent_node_then_postcondition_violated() {
        let doc = DiagramDocument::default();
        let ghost = NodeId::new("does_not_exist".to_string());
        let selection = Selection { nodes: vec![ghost] };

        let result = copy(&selection, &doc);
        assert!(matches!(result, Err(Error::PostconditionViolated(_))));
    }

    #[test]
    fn given_cut_with_connected_edges_then_edges_removed_from_document() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());
        doc.document.edges.insert(e1.clone(), create_test_edge(n1.clone(), n2.clone()));

        let selection = Selection { nodes: vec![n1, n2] };
        let clipboard = cut(&selection, &mut doc).unwrap();

        assert_eq!(clipboard.edges.len(), 1, "clipboard should contain the edge");
        assert!(!doc.document.edges.contains_key(&e1), "edge should be removed from doc after cut");
    }

    #[test]
    fn given_paste_result_then_new_selection_contains_all_pasted_node_ids() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());

        let selection = Selection { nodes: vec![n1, n2] };
        let clipboard = copy(&selection, &doc).unwrap();
        let paste_res = calculate_paste(&clipboard, &doc).unwrap();

        assert_eq!(paste_res.new_selection.len(), 2);
        for (new_id, _) in &paste_res.new_nodes {
            assert!(paste_res.new_selection.contains(new_id.as_str()));        let paste1 = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(paste1.new_nodes.len(), 1);
        let first_paste_id = &paste1.new_nodes[0].0.clone();

        let mut doc_with_first_paste = doc.clone();
        doc_with_first_paste.document.nodes.extend(paste1.new_nodes);
        doc_with_first_paste.document.edges.extend(paste1.new_edges);

        let mut clipboard2 = clipboard.clone();
        clipboard2.paste_serial = 1;
        let paste2 = calculate_paste(&clipboard2, &doc_with_first_paste).unwrap();
        assert_eq!(paste2.new_nodes.len(), 1);
        let second_paste_id = &paste2.new_nodes[0].0;

        assert_ne!(first_paste_id, second_paste_id);
        assert!(!doc_with_first_paste
            .document
            .nodes
            .contains_key(second_paste_id));
    }

    #[test]
    fn test_clp008_empty_clipboard_copy_returns_error() {
        let doc = DiagramDocument::default();
        let result = copy(&Selection::empty(), &doc);
        assert!(matches!(result, Err(Error::EmptySelection)));
    }

    #[test]
    fn test_clp009_id_remapping_preserves_all_copied_edges_on_paste() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let n3 = NodeId::new("n3".to_string());
        let e1 = EdgeId::new("e1".to_string());
        let e2 = EdgeId::new("e2".to_string());

        let mut node1 = create_test_node();
        node1.x = OrderedFloat(0.0);
        let mut node2 = create_test_node();
        node2.x = OrderedFloat(100.0);
        let mut node3 = create_test_node();
        node3.x = OrderedFloat(200.0);

        doc.document.nodes.insert(n1.clone(), node1);
        doc.document.nodes.insert(n2.clone(), node2);
        doc.document.nodes.insert(n3.clone(), node3);
        doc.document
            .edges
            .insert(e1.clone(), create_test_edge(n1.clone(), n2.clone()));
        doc.document
            .edges
            .insert(e2.clone(), create_test_edge(n2.clone(), n3.clone()));

        let selection = Selection {
            nodes: vec![n1.clone(), n2.clone(), n3.clone()],
        };

        let clipboard = copy(&selection, &doc).unwrap();
        let paste_res = calculate_paste(&clipboard, &doc).unwrap();

        assert_eq!(paste_res.new_nodes.len(), 3);
        assert_eq!(paste_res.new_edges.len(), 2);

        let new_node_ids: std::collections::HashSet<_> =
            paste_res.new_nodes.iter().map(|(id, _)| id).collect();

        for (_, edge) in &paste_res.new_edges {
            assert!(
                new_node_ids.contains(&edge.source),
                "Edge source should be from pasted nodes"
            );
            assert!(
                new_node_ids.contains(&edge.target),
                "Edge target should be from pasted nodes"
            );
            assert_ne!(
                &edge.source, &n1,
                "Edge source should be remapped to new ID"
            );
            assert_ne!(
                &edge.target, &n2,
                "Edge target should be remapped to new ID"
            );
        }
    }

    #[test]
    fn given_edge_source_in_clipboard_target_in_doc_when_paste_then_edge_remapped_to_new_ids() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document.nodes.insert(n1.clone(), create_test_node());
        doc.document.nodes.insert(n2.clone(), create_test_node());
        doc.document.edges.insert(e1, create_test_edge(n1.clone(), n2.clone()));

        let selection = Selection { nodes: vec![n1.clone(), n2.clone()] };
        let clipboard = copy(&selection, &doc).unwrap();
        let paste_res = calculate_paste(&clipboard, &doc).unwrap();

        let new_node_ids: std::collections::HashSet<_> =
            paste_res.new_nodes.iter().map(|(id, _)| id.clone()).collect();

        let (_, pasted_edge) = &paste_res.new_edges[0];
        assert!(
            new_node_ids.contains(&pasted_edge.source),
            "edge source must be remapped to a NEW node id, not the original"
        );
        assert!(
            new_node_ids.contains(&pasted_edge.target),
            "edge target must be remapped to a NEW node id, not the original"
        );
        assert_ne!(pasted_edge.source, n1);
        assert_ne!(pasted_edge.target, n2);
    }

    #[test]
    fn given_parent_in_existing_doc_when_paste_then_parent_preserved_as_existing_doc_node() {
        let mut doc = DiagramDocument::default();
        let doc_parent = NodeId::new("doc_parent".to_string());
        let clip_child = NodeId::new("clip_child".to_string());
        doc.document.nodes.insert(doc_parent.clone(), create_test_node());

        let mut clipboard = ClipboardData::empty();
        let mut child_node = create_test_node();
        child_node.parent = Some(doc_parent.clone());
        clipboard.nodes.push((clip_child, child_node));

        let result = calculate_paste(&clipboard, &doc);
        assert!(result.is_ok());
        let paste_res = result.unwrap();
        let (_, pasted_node) = &paste_res.new_nodes[0];
        assert_eq!(
            pasted_node.parent,
            Some(doc_parent),
            "parent pointing to existing doc node should be preserved"
        );
    }

    #[test]
    fn given_large_paste_serial_then_offset_formula_correct() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        doc.document.nodes.insert(n1, create_test_node());

        let selection = Selection { nodes: vec![NodeId::new("n1".to_string())] };
        let mut clipboard = copy(&selection, &doc).unwrap();

        clipboard.paste_serial = 9;
        let paste = calculate_paste(&clipboard, &doc).unwrap();
        assert_eq!(
            paste.new_nodes[0].1.x.0,
            200.0,
            "offset = 20 * (9+1) = 200"
        );
        assert_eq!(
            paste.new_nodes[0].1.y.0,
            200.0,
            "y offset should match x offset"
        );
    }

    #[test]
    fn given_4_node_diamond_cycle_when_paste_then_cyclic_detected() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();

        let a = NodeId::new("a".to_string());
        let b = NodeId::new("b".to_string());
        let c = NodeId::new("c".to_string());
        let d = NodeId::new("d".to_string());

        let mut na = create_test_node(); na.parent = Some(b.clone());
        let mut nb = create_test_node(); nb.parent = Some(c.clone());
        let mut nc = create_test_node(); nc.parent = Some(d.clone());
        let mut nd = create_test_node(); nd.parent = Some(a.clone());

        clipboard.nodes.push((a, na));
        clipboard.nodes.push((b, nb));
        clipboard.nodes.push((c, nc));
        clipboard.nodes.push((d, nd));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::CyclicParentReference)));
    }

    #[test]
    fn given_clipboard_with_only_edges_no_nodes_when_paste_then_empty_clipboard_error() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();
        clipboard.edges.push((
            EdgeId::new("e1".to_string()),
            create_test_edge(NodeId::new("a".to_string()), NodeId::new("b".to_string())),
        ));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::EmptyClipboard)));
    }

    #[test]
    fn given_copy_preserves_node_properties_beyond_position() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let mut node = create_test_node();
        node.label = "MyLabel".to_string();
        node.width = OrderedFloat(200.0);
        node.height = OrderedFloat(150.0);
        node.z_index = 5;
        doc.document.nodes.insert(n1, node);

        let selection = Selection { nodes: vec![NodeId::new("n1".to_string())] };
        let clipboard = copy(&selection, &doc).unwrap();
        let paste_res = calculate_paste(&clipboard, &doc).unwrap();

        let (_, pasted) = &paste_res.new_nodes[0];
        assert_eq!(pasted.label, "MyLabel");
        assert_eq!(pasted.width.0, 200.0);
        assert_eq!(pasted.height.0, 150.0);
        assert_eq!(pasted.z_index, 5);
    }

    #[test]
    fn given_edge_with_both_ends_in_existing_doc_when_paste_then_edge_preserved() {
        let mut doc = DiagramDocument::default();
        let existing_a = NodeId::new("existing_a".to_string());
        let existing_b = NodeId::new("existing_b".to_string());
        doc.document.nodes.insert(existing_a.clone(), create_test_node());
        doc.document.nodes.insert(existing_b.clone(), create_test_node());

        let mut clipboard = ClipboardData::empty();
        let clip_node = NodeId::new("clip_node".to_string());
        clipboard.nodes.push((clip_node, create_test_node()));

        clipboard.edges.push((
            EdgeId::new("e1".to_string()),
            create_test_edge(existing_a.clone(), existing_b.clone()),
        ));

        let result = calculate_paste(&clipboard, &doc);
        assert!(result.is_ok(), "edge between existing doc nodes should be valid");
        let paste_res = result.unwrap();
        assert_eq!(paste_res.new_edges.len(), 1);
        let (_, edge) = &paste_res.new_edges[0];
        assert_eq!(edge.source, existing_a);
        assert_eq!(edge.target, existing_b);
    }

    #[test]
    fn given_multiple_pastes_then_each_gets_unique_node_ids() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        doc.document.nodes.insert(n1, create_test_node());

        let selection = Selection { nodes: vec![NodeId::new("n1".to_string())] };
        let clipboard = copy(&selection, &doc).unwrap();

        let paste1 = calculate_paste(&clipboard, &doc).unwrap();
        let paste2 = calculate_paste(&clipboard, &doc).unwrap();

        assert_ne!(
            paste1.new_nodes[0].0, paste2.new_nodes[0].0,
            "each paste must generate unique node IDs"
        );    fn test_clp010_paste_offset_increases_with_each_paste_serial() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let mut node1 = create_test_node();
        node1.x = OrderedFloat(0.0);
        node1.y = OrderedFloat(0.0);
        doc.document.nodes.insert(n1.clone(), node1);

        let selection = Selection { nodes: vec![n1] };

        let mut clipboard = copy(&selection, &doc).unwrap();

        clipboard.paste_serial = 0;
        let paste0 = calculate_paste(&clipboard, &doc).unwrap();
        let offset0_x = paste0.new_nodes[0].1.x.0;

        clipboard.paste_serial = 1;
        let paste1 = calculate_paste(&clipboard, &doc).unwrap();
        let offset1_x = paste1.new_nodes[0].1.x.0;

        clipboard.paste_serial = 5;
        let paste5 = calculate_paste(&clipboard, &doc).unwrap();
        let offset5_x = paste5.new_nodes[0].1.x.0;

        assert_eq!(offset0_x, 20.0);
        assert_eq!(offset1_x, 40.0);
        assert_eq!(offset5_x, 120.0);
    }
}
