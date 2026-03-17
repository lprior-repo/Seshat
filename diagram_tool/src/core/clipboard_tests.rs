#[cfg(test)]
mod tests {

    use crate::test_utils::builders::{test_edge, test_edge_builder, test_node, test_node_default};
    use diagram_models::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_copy_with_empty_selection_returns_error() {
        let doc = DiagramDocument::default();
        let result = copy_selection(&doc);
        assert_eq!(result.unwrap_err(), ClipboardError::EmptySelection);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_copy_single_node_populates_clipboard() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string()).unwrap();
        doc.document.nodes.insert(n1.clone(), test_node_default());
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        let clipboard = copy_selection(&doc).unwrap();
        assert_eq!(clipboard.nodes.len(), 1);
        assert_eq!(clipboard.nodes[0].0, n1);
        assert_eq!(clipboard.edges.len(), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_cut_with_empty_selection_returns_empty_selection_error() {
        let mut doc = DiagramDocument::default();
        let result = crate::core::clipboard::cut_selection(&mut doc);
        assert_eq!(result.unwrap_err(), ClipboardError::EmptySelection);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_duplicate_with_empty_selection_returns_empty_selection_error() {
        let mut doc = DiagramDocument::default();
        let result = crate::core::clipboard::duplicate_selection(&mut doc);
        assert_eq!(result.unwrap_err(), ClipboardError::EmptySelection);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_cut_single_node_returns_clipboard_and_removes_from_doc() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string()).unwrap();
        doc.document.nodes.insert(n1.clone(), test_node_default());
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        let clipboard = crate::core::clipboard::cut_selection(&mut doc).unwrap();
        assert_eq!(clipboard.nodes.len(), 1);
        assert_eq!(doc.document.nodes.len(), 0);
        assert_eq!(doc.editor_state.selected_items.len(), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_cut_multiple_nodes_with_edges_removes_subgraph() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string()).unwrap();
        let n2 = NodeId::new("n2".to_string()).unwrap();
        let n3 = NodeId::new("n3".to_string()).unwrap();

        doc.document.nodes.insert(n1.clone(), test_node_default());
        doc.document.nodes.insert(n2.clone(), test_node_default());
        doc.document.nodes.insert(n3.clone(), test_node_default());

        let e1 = diagram_models::document::EdgeId::new("e1".to_string());
        let e2 = diagram_models::document::EdgeId::new("e2".to_string());

        doc.document
            .edges
            .insert(e1.clone(), test_edge(n1.clone(), n2.clone()));

        doc.document
            .edges
            .insert(e2.clone(), test_edge(n2.clone(), n3.clone()));

        // Select n1 and n2
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2.as_str().to_string());

        let clipboard = crate::core::clipboard::cut_selection(&mut doc).unwrap();

        assert_eq!(clipboard.nodes.len(), 2);
        assert_eq!(clipboard.edges.len(), 1); // e1 is internal to selection

        // n3 should remain
        assert_eq!(doc.document.nodes.len(), 1);
        assert!(doc.document.nodes.contains_key(&n3));

        // no edges should remain (e1 cut, e2 dangling and removed)
        assert_eq!(doc.document.edges.len(), 0);

        assert_eq!(doc.editor_state.selected_items.len(), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_duplicate_single_node_creates_new_node_with_offset() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string()).unwrap();
        let mut node = test_node_default();
        node.x = OrderedFloat(10.0);
        node.y = OrderedFloat(10.0);
        doc.document.nodes.insert(n1.clone(), node);
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        crate::core::clipboard::duplicate_selection(&mut doc).unwrap();

        assert_eq!(doc.document.nodes.len(), 2);
        assert_eq!(doc.editor_state.selected_items.len(), 1);

        let new_id_str = doc.editor_state.selected_items.iter().next().unwrap();
        let new_id = NodeId::new(new_id_str.clone().unwrap());
        assert_ne!(new_id, n1);

        let new_node = doc.document.nodes.get(&new_id).unwrap();
        assert_eq!(new_node.x, OrderedFloat(30.0));
        assert_eq!(new_node.y, OrderedFloat(30.0));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_duplicate_multiple_nodes_with_edges_preserves_topology() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string()).unwrap();
        let n2 = NodeId::new("n2".to_string()).unwrap();
        doc.document.nodes.insert(n1.clone(), test_node_default());
        doc.document.nodes.insert(n2.clone(), test_node_default());

        let e1 = diagram_models::document::EdgeId::new("e1".to_string());
        doc.document
            .edges
            .insert(e1.clone(), test_edge(n1.clone(), n2.clone()));

        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2.as_str().to_string());

        crate::core::clipboard::duplicate_selection(&mut doc).unwrap();

        assert_eq!(doc.document.nodes.len(), 4);
        assert_eq!(doc.document.edges.len(), 2);
        assert_eq!(doc.editor_state.selected_items.len(), 2);

        let new_edges: Vec<_> = doc
            .document
            .edges
            .iter()
            .filter(|(k, _)| **k != e1)
            .collect();
        assert_eq!(new_edges.len(), 1);
        let new_edge = new_edges[0].1;

        // Edge should connect to the newly created nodes
        assert!(doc
            .editor_state
            .selected_items
            .contains(new_edge.source.as_str()));
        assert!(doc
            .editor_state
            .selected_items
            .contains(new_edge.target.as_str()));
    }
}
