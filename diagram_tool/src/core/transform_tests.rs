#[cfg(test)]
mod tests {
    use crate::core::transform::{
        align_selection, distribute_selection, AlignmentAxis, AlignmentMode, TransformError,
    };
    use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};

    fn test_node(x: f64, width: f64) -> Node {
        Node {
            kind: NodeKind::Text,
            icon: String::new(),
            label: "Test".to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(0.0),
            width: OrderedFloat(width),
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

    #[test]
    fn test_align_left_snaps_all_nodes_to_min_x() {
        let mut doc = DiagramDocument::default();

        let n1 = NodeId::new("1".to_string());
        let n2 = NodeId::new("2".to_string());

        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));
        doc.document
            .nodes
            .insert(n2.clone(), test_node(200.0, 50.0));

        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2.as_str().to_string());

        align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start).unwrap();

        assert_eq!(doc.document.nodes.get(&n1).unwrap().x.0, 100.0);
        assert_eq!(doc.document.nodes.get(&n2).unwrap().x.0, 100.0);
    }

    #[test]
    fn test_locked_element_cannot_be_moved() {
        let mut doc = DiagramDocument::default();

        // Two nodes - one locked, one not
        let n1 = NodeId::new("1".to_string());
        let n2 = NodeId::new("2".to_string());
        
        let mut node1 = test_node(100.0, 50.0);
        node1.locked = true;
        
        let node2 = test_node(200.0, 50.0);

        doc.document.nodes.insert(n1.clone(), node1);
        doc.document.nodes.insert(n2.clone(), node2);
        
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2.as_str().to_string());

        let err =
            align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start).unwrap_err();
        assert_eq!(err, TransformError::LockedNode(n1));
    }

    #[test]
    fn test_distribute_handles_overlapping_nodes() {
        let mut doc = DiagramDocument::default();

        let n1 = NodeId::new("1".to_string());
        let n2 = NodeId::new("2".to_string());
        let n3 = NodeId::new("3".to_string());

        doc.document.nodes.insert(n1.clone(), test_node(0.0, 50.0));
        doc.document.nodes.insert(n2.clone(), test_node(10.0, 50.0)); // Overlaps n1
        doc.document
            .nodes
            .insert(n3.clone(), test_node(200.0, 50.0));

        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n3.as_str().to_string());

        distribute_selection(&mut doc, AlignmentAxis::Horizontal).unwrap();

        // Total space = (200 + 50) - 0 = 250
        // Sum of widths = 50 * 3 = 150
        // Available space = 100
        // Spacing = 100 / 2 = 50

        assert_eq!(doc.document.nodes.get(&n1).unwrap().x.0, 0.0);
        assert_eq!(doc.document.nodes.get(&n2).unwrap().x.0, 100.0); // 0 + 50 (width) + 50 (spacing)
        assert_eq!(doc.document.nodes.get(&n3).unwrap().x.0, 200.0); // 100 + 50 (width) + 50 (spacing)
    }
}
