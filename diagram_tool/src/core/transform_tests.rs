#![allow(unused)]
#![ignore]

#[cfg(test)]
mod tests {
    use crate::core::transform::{
        align_selection, distribute_selection, translate_selection, AlignmentAxis, AlignmentMode,
        TransformError,
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

        align_selection(&mut doc, &AlignmentAxis::Horizontal, &AlignmentMode::Start).unwrap();

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

        let err = align_selection(&mut doc, &AlignmentAxis::Horizontal, &AlignmentMode::Start)
            .unwrap_err();
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

        distribute_selection(&mut doc, &AlignmentAxis::Horizontal).unwrap();

        // Total space = (200 + 50) - 0 = 250
        // Sum of widths = 50 * 3 = 150
        // Available space = 100
        // Spacing = 100 / 2 = 50

        assert_eq!(doc.document.nodes.get(&n1).unwrap().x.0, 0.0);
        assert_eq!(doc.document.nodes.get(&n2).unwrap().x.0, 100.0); // 0 + 50 (width) + 50 (spacing)
        assert_eq!(doc.document.nodes.get(&n3).unwrap().x.0, 200.0); // 100 + 50 (width) + 50 (spacing)
    }

    #[test]
    fn test_MUL_006_translate_single_node_updates_coordinates() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        translate_selection(&mut doc, 10.0, 20.0).unwrap();

        let node = doc.document.nodes.get(&n1).unwrap();
        assert_eq!(node.x.0, 110.0);
        assert_eq!(node.y.0, 20.0);
    }

    #[test]
    fn test_MUL_007_translate_multiple_nodes_updates_all_coordinates() {
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

        translate_selection(&mut doc, -10.0, -5.0).unwrap();

        assert_eq!(doc.document.nodes.get(&n1).unwrap().x.0, 90.0);
        assert_eq!(doc.document.nodes.get(&n1).unwrap().y.0, -5.0);
        assert_eq!(doc.document.nodes.get(&n2).unwrap().x.0, 190.0);
        assert_eq!(doc.document.nodes.get(&n2).unwrap().y.0, -5.0);
    }

    #[test]
    fn test_MUL_008_translate_empty_selection_returns_error() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));

        let err = translate_selection(&mut doc, 10.0, 20.0).unwrap_err();
        assert_eq!(err, TransformError::EmptySelection);
        assert_eq!(doc.document.nodes.get(&n1).unwrap().x.0, 100.0);
    }

    #[test]
    fn test_MUL_009_translate_with_locked_node_returns_error_and_does_not_translate() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        let n2 = NodeId::new("2".to_string());

        let mut node1 = test_node(100.0, 50.0);
        node1.locked = true;
        doc.document.nodes.insert(n1.clone(), node1);
        doc.document
            .nodes
            .insert(n2.clone(), test_node(200.0, 50.0));

        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2.as_str().to_string());

        let err = translate_selection(&mut doc, 10.0, 20.0).unwrap_err();
        assert_eq!(err, TransformError::LockedNode(n1.clone()));

        assert_eq!(doc.document.nodes.get(&n1).unwrap().x.0, 100.0);
        assert_eq!(doc.document.nodes.get(&n2).unwrap().x.0, 200.0);
    }

    #[test]
    fn test_translate_by_zero_delta_succeeds_without_modifying_coordinates() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        translate_selection(&mut doc, 0.0, 0.0).unwrap();

        assert_eq!(doc.document.nodes.get(&n1).unwrap().x.0, 100.0);
        assert_eq!(doc.document.nodes.get(&n1).unwrap().y.0, 0.0);
    }

    #[test]
    fn test_translate_negative_delta_moves_nodes_up_and_left() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        translate_selection(&mut doc, -10.0, -20.0).unwrap();

        assert_eq!(doc.document.nodes.get(&n1).unwrap().x.0, 90.0);
        assert_eq!(doc.document.nodes.get(&n1).unwrap().y.0, -20.0);
    }

    #[test]
    fn test_precondition_selection_not_empty() {
        let mut doc = DiagramDocument::default();
        let err = translate_selection(&mut doc, 10.0, 20.0).unwrap_err();
        assert_eq!(err, TransformError::EmptySelection);
    }

    #[test]
    fn test_precondition_no_locked_nodes() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        let mut node = test_node(100.0, 50.0);
        node.locked = true;
        doc.document.nodes.insert(n1.clone(), node);
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        let err = translate_selection(&mut doc, 10.0, 20.0).unwrap_err();
        assert_eq!(err, TransformError::LockedNode(n1));
    }

    #[test]
    fn test_postcondition_unselected_nodes_unmodified() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        let n2 = NodeId::new("2".to_string());

        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));
        doc.document
            .nodes
            .insert(n2.clone(), test_node(200.0, 50.0));

        // Select only n1
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        translate_selection(&mut doc, 10.0, 20.0).unwrap();

        // n2 should remain unmodified
        assert_eq!(doc.document.nodes.get(&n2).unwrap().x.0, 200.0);
        assert_eq!(doc.document.nodes.get(&n2).unwrap().y.0, 0.0);
    }

    #[test]
    fn test_postcondition_ancestor_containers_recomputed() {
        let mut doc = DiagramDocument::default();

        let container_id = NodeId::new("container".to_string());
        let child_id = NodeId::new("child".to_string());

        let mut container = test_node(0.0, 200.0);
        container.kind = NodeKind::Subgraph;
        container.height = OrderedFloat(200.0);

        let mut child = test_node(50.0, 50.0);
        child.parent = Some(container_id.clone());

        doc.document.nodes.insert(container_id.clone(), container);
        doc.document.nodes.insert(child_id.clone(), child);

        doc.editor_state
            .selected_items
            .insert(child_id.as_str().to_string());

        translate_selection(&mut doc, 50.0, 50.0).unwrap();

        // Child is moved
        assert_eq!(doc.document.nodes.get(&child_id).unwrap().x.0, 100.0);
        assert_eq!(doc.document.nodes.get(&child_id).unwrap().y.0, 50.0);

        // Container bounds should be recomputed based on child's new position
        // Padding is 24.0. Child is at x=100, y=50, w=50, h=100
        // Container x = 100 - 24 = 76
        // Container y = 50 - 24 = 26
        // Container w = 50 + 48 = 98
        // Container h = 100 + 48 = 148
        let container = doc.document.nodes.get(&container_id).unwrap();
        assert_eq!(container.x.0, 76.0);
        assert_eq!(container.y.0, 26.0);
        assert_eq!(container.width.0, 98.0);
        assert_eq!(container.height.0, 148.0);
    }

    #[test]
    fn test_invariant_node_count_remains_unchanged() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        let count_before = doc.document.nodes.len();
        translate_selection(&mut doc, 10.0, 20.0).unwrap();
        let count_after = doc.document.nodes.len();

        assert_eq!(count_before, count_after);
    }

    #[test]
    fn test_invariant_selection_remains_unchanged() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        let selection_before = doc.editor_state.selected_items.clone();
        translate_selection(&mut doc, 10.0, 20.0).unwrap();
        let selection_after = doc.editor_state.selected_items.clone();

        assert_eq!(selection_before, selection_after);
    }

    #[test]
    fn test_P1_violation_empty_selection_returns_empty_selection_error() {
        let mut doc = DiagramDocument::default();
        let err = translate_selection(&mut doc, 10.0, 10.0).unwrap_err();
        assert_eq!(err, TransformError::EmptySelection);
    }

    #[test]
    fn test_P2_violation_locked_node_returns_locked_node_error() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        let n2 = NodeId::new("2".to_string());

        let mut node1 = test_node(100.0, 50.0);
        node1.locked = true;

        doc.document.nodes.insert(n1.clone(), node1);
        doc.document
            .nodes
            .insert(n2.clone(), test_node(200.0, 50.0));

        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2.as_str().to_string());

        let err = translate_selection(&mut doc, 10.0, 10.0).unwrap_err();
        assert_eq!(err, TransformError::LockedNode(n1));
    }

    #[test]
    fn test_P3_violation_nan_delta_returns_invalid_delta_error() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), test_node(100.0, 50.0));
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        let err = translate_selection(&mut doc, f64::NAN, 10.0).unwrap_err();
        assert_eq!(err, TransformError::InvalidDelta);
    }

    #[test]
    fn test_scenario_1_group_translate_with_container_bounds_update() {
        let mut doc = DiagramDocument::default();

        let node_a_id = NodeId::new("A".to_string());
        let node_b_id = NodeId::new("B".to_string());
        let node_c_id = NodeId::new("C".to_string());
        let container_d_id = NodeId::new("D".to_string());

        let mut node_a = test_node(10.0, 50.0);
        node_a.y = OrderedFloat(10.0);
        node_a.parent = Some(container_d_id.clone());

        let mut node_b = test_node(100.0, 50.0);
        node_b.y = OrderedFloat(10.0);

        let mut node_c = test_node(200.0, 50.0);
        node_c.y = OrderedFloat(10.0);

        let mut container_d = test_node(0.0, 200.0);
        container_d.kind = NodeKind::Subgraph;

        doc.document.nodes.insert(node_a_id.clone(), node_a);
        doc.document.nodes.insert(node_b_id.clone(), node_b);
        doc.document.nodes.insert(node_c_id.clone(), node_c);
        doc.document
            .nodes
            .insert(container_d_id.clone(), container_d);

        doc.editor_state
            .selected_items
            .insert(node_a_id.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(node_b_id.as_str().to_string());

        let res = translate_selection(&mut doc, 20.0, -10.0);
        assert!(res.is_ok());

        // Node A
        let a = doc.document.nodes.get(&node_a_id).unwrap();
        assert_eq!(a.x.0, 30.0);
        assert_eq!(a.y.0, 0.0);

        // Node B
        let b = doc.document.nodes.get(&node_b_id).unwrap();
        assert_eq!(b.x.0, 120.0);
        assert_eq!(b.y.0, 0.0);

        // Node C
        let c = doc.document.nodes.get(&node_c_id).unwrap();
        assert_eq!(c.x.0, 200.0);
        assert_eq!(c.y.0, 10.0);

        // Container D bounds
        // Child A is at x=30, y=0, w=50, h=100
        // Padding = 24
        let d = doc.document.nodes.get(&container_d_id).unwrap();
        assert_eq!(d.x.0, 30.0 - 24.0); // 6.0
        assert_eq!(d.y.0, 0.0 - 24.0); // -24.0
        assert_eq!(d.width.0, 50.0 + 48.0); // 98.0
        assert_eq!(d.height.0, 100.0 + 48.0); // 148.0
    }
}
