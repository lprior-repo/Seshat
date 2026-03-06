#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grouping::group_selection;
    use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};

    fn test_node(x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind: NodeKind::Text,
            icon: String::new(),
            label: "Test".to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
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
        group_selection(&mut doc, group_id.clone()).unwrap();

        // Assert group created
        let group = doc.document.nodes.get(&group_id).unwrap();
        assert_eq!(group.kind, NodeKind::Subgraph);

        // Bounding box = min_x(100), min_y(100), max_x(250), max_y(250)
        // With padding 20: min_x(80), min_y(80), max_x(270), max_y(270)
        // width = 270 - 80 = 190, height = 270 - 80 = 190
        assert_eq!(group.x.0, 80.0);
        assert_eq!(group.y.0, 80.0);
        assert_eq!(group.width.0, 190.0);
        assert_eq!(group.height.0, 190.0);

        // Assert children reparented
        assert_eq!(
            doc.document.nodes.get(&n1).unwrap().parent,
            Some(group_id.clone())
        );
        assert_eq!(
            doc.document.nodes.get(&n2).unwrap().parent,
            Some(group_id.clone())
        );

        // Assert selection is now just the group
        assert_eq!(doc.editor_state.selected_items.len(), 1);
        assert!(doc.editor_state.selected_items.contains(group_id.as_str()));
    }
}
