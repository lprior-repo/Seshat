#[cfg(test)]
mod tests {
    use crate::models::clipboard_contract::{
        calculate_paste, copy, cut, ClipboardData, Error, Selection,
    };
    use crate::models::document::{
        DiagramDocument, Edge, EdgeId, Node, NodeId, NodeKind, OrderedFloat,
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
            locked: false,
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
            .insert(e1.clone(), create_test_edge(n1.clone(), n2.clone()));

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
            nodes: vec![p1.clone(), c1.clone()],
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

        let selection = Selection {
            nodes: vec![n1.clone()],
        };
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
            create_test_edge(n1.clone(), n1.clone()),
        ));

        let valid_node = NodeId::new("valid".to_string());
        clipboard
            .nodes
            .push((valid_node.clone(), create_test_node()));

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

        clipboard.nodes.push((n1.clone(), child_node));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::InvalidParentReference)));
    }

    #[test]
    fn test_corrupt_clipboard_with_duplicate_node_ids() {
        let doc = DiagramDocument::default();
        let mut clipboard = ClipboardData::empty();
        let n1 = NodeId::new("n1".to_string());
        clipboard.nodes.push((n1.clone(), create_test_node()));
        clipboard.nodes.push((n1.clone(), create_test_node()));
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

        clipboard.nodes.push((n1.clone(), node1));
        clipboard.nodes.push((n2.clone(), node2));

        let result = calculate_paste(&clipboard, &doc);
        assert!(matches!(result, Err(Error::CyclicParentReference)));
    }
}
