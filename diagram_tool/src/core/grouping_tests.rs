#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grouping::{group_selection, ungroup_selection, GroupingError};
    use crate::models::document::{DiagramDocument, Edge, Node, NodeId, NodeKind, OrderedFloat};

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

    fn test_subgraph() -> Node {
        Node {
            kind: NodeKind::Subgraph,
            icon: String::new(),
            label: "Group".to_string(),
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

    #[test]
    fn test_ungroup_selection_empty() {
        let mut doc = DiagramDocument::default();
        assert_eq!(
            ungroup_selection(&mut doc),
            Err(GroupingError::EmptySelection)
        );
    }

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

        // Select the group
        doc.editor_state
            .selected_items
            .insert(group_id.as_str().to_string());

        assert_eq!(ungroup_selection(&mut doc), Ok(()));

        // Subgraph should be deleted
        assert!(!doc.document.nodes.contains_key(&group_id));

        // Children should be orphaned
        assert_eq!(doc.document.nodes.get(&c1_id).unwrap().parent, None);
        assert_eq!(doc.document.nodes.get(&c2_id).unwrap().parent, None);

        // Children should be selected
        assert_eq!(doc.editor_state.selected_items.len(), 2);
        assert!(doc.editor_state.selected_items.contains(c1_id.as_str()));
        assert!(doc.editor_state.selected_items.contains(c2_id.as_str()));
    }

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

        // Select the sub_group
        doc.editor_state
            .selected_items
            .insert(sub_group_id.as_str().to_string());

        assert_eq!(ungroup_selection(&mut doc), Ok(()));

        // Subgroup should be deleted
        assert!(!doc.document.nodes.contains_key(&sub_group_id));
        // Parent group should remain
        assert!(doc.document.nodes.contains_key(&parent_group_id));

        // Child should inherit the sub_group's parent
        assert_eq!(
            doc.document.nodes.get(&c_id).unwrap().parent,
            Some(parent_group_id)
        );

        // Child should be selected
        assert_eq!(doc.editor_state.selected_items.len(), 1);
        assert!(doc.editor_state.selected_items.contains(c_id.as_str()));
    }

    #[test]
    fn test_ungroup_selection_removes_edges_connected_to_subgraph() {
        let mut doc = DiagramDocument::default();

        let group_id = NodeId::new("g1".to_string());
        doc.document.nodes.insert(group_id.clone(), test_subgraph());

        let node_id = NodeId::new("n1".to_string());
        doc.document
            .nodes
            .insert(node_id.clone(), test_node(100.0, 100.0, 20.0, 20.0));

        let edge_id = crate::models::document::EdgeId::new("e1".to_string());
        let edge = Edge {
            source: group_id.clone(),
            target: node_id.clone(),
            label: String::new(),
            style: crate::models::document::EdgeStyle::default(),
            arrow_type: crate::models::document::ArrowType::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.0),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            font_size: None,
        };
        doc.document.edges.insert(edge_id.clone(), edge);

        // Select the group
        doc.editor_state
            .selected_items
            .insert(group_id.as_str().to_string());

        assert_eq!(ungroup_selection(&mut doc), Ok(()));

        // Edge should be removed
        assert!(!doc.document.edges.contains_key(&edge_id));
    }
}
