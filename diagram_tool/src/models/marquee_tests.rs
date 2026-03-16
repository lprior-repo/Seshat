#[cfg(test)]
mod marquee_tests {
    use crate::models::document::{
        DiagramDocument, DocumentError, LockState, Node, NodeId, NodeKind, OrderedFloat, ValidRect,
    };
    use crate::models::spatial_index::MarqueeMode;
    use im::HashMap;
    use serde_json::json;

    fn setup_doc_with_nodes() -> DiagramDocument {
        let mut doc = DiagramDocument::default();

        // N1: (10, 10) 50x50. Enclosed by (0,0)->(100,100)
        let n1 = create_node("n1", 10.0, 10.0, 50.0, 50.0);

        // N2: (80, 80) 50x50. Intersects with (0,0)->(100,100)
        let n2 = create_node("n2", 80.0, 80.0, 50.0, 50.0);

        // N3: (150, 150) 50x50. Outside (0,0)->(100,100)
        let n3 = create_node("n3", 150.0, 150.0, 50.0, 50.0);

        // N4: (10, 10) 50x50, but rotated 45 degrees, so it goes slightly outside (10..60) -> approx (-5..75)
        let mut n4 = create_node("n4", 10.0, 10.0, 50.0, 50.0);
        n4.metadata
            .insert("rotation".to_string(), json!(std::f64::consts::FRAC_PI_4));

        doc.document.nodes.insert(NodeId::new("n1".to_string()), n1);
        doc.document.nodes.insert(NodeId::new("n2".to_string()), n2);
        doc.document.nodes.insert(NodeId::new("n3".to_string()), n3);
        doc.document.nodes.insert(NodeId::new("n4".to_string()), n4);

        doc
    }

    fn create_node(id: &str, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat::new_unchecked(x),
            y: OrderedFloat::new_unchecked(y),
            width: OrderedFloat::new_unchecked(w),
            height: OrderedFloat::new_unchecked(h),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn should_reject_marquee_with_negative_dimensions() {
        let result = ValidRect::new(0.0, 0.0, -10.0, 10.0);
        assert_eq!(result.unwrap_err(), DocumentError::InvalidMarqueeBounds);
    }

    #[test]
    fn should_report_fully_enclosed_nodes_as_selected_in_contain_mode() {
        let mut doc = setup_doc_with_nodes();
        // Bounds: x=0, y=0, w=70, h=70
        // N1 (10..60, 10..60) is fully enclosed.
        // N4 is rotated, bounding box goes outside 0..70 (wait, rotated 50x50 around center 35,35 -> radius 35.3 -> bounds ~ -0.3 to 70.3, so NOT fully enclosed)
        let bounds = ValidRect::new(0.0, 0.0, 70.0, 70.0).unwrap();

        // Before state (cloned to check Q4)
        let doc_before = doc.clone();

        doc.select_marquee(bounds, MarqueeMode::Contain).unwrap();

        assert!(doc.editor_state.selected_items.contains("n1"));
        assert!(!doc.editor_state.selected_items.contains("n2")); // Intersects only
        assert!(!doc.editor_state.selected_items.contains("n3")); // Outside

        // Q4: Observable state is unchanged
        assert_eq!(doc.document, doc_before.document);
    }

    #[test]
    fn should_report_intersecting_nodes_as_selected_in_intersect_mode() {
        let mut doc = setup_doc_with_nodes();
        // Bounds: x=0, y=0, w=100, h=100
        let bounds = ValidRect::new(0.0, 0.0, 100.0, 100.0).unwrap();

        doc.select_marquee(bounds, MarqueeMode::Intersect).unwrap();

        assert!(doc.editor_state.selected_items.contains("n1")); // Enclosed
        assert!(doc.editor_state.selected_items.contains("n2")); // Intersects
        assert!(!doc.editor_state.selected_items.contains("n3")); // Outside
        assert!(doc.editor_state.selected_items.contains("n4")); // Enclosed/Intersects
    }

    #[test]
    fn should_accurately_select_rotated_nodes_within_marquee() {
        let mut doc = setup_doc_with_nodes();
        // N4 is at (10, 10) to (60, 60), center (35, 35). Rotated 45 deg, half-diagonal is 50 * sqrt(2) / 2 = 35.355.
        // Bounding box is [35 - 35.355, 35 + 35.355] = [-0.355, 70.355].
        // If we use a contain marquee of [-1.0, -1.0, 72.0, 72.0], it should contain it.
        let bounds = ValidRect::new(-1.0, -1.0, 73.0, 73.0).unwrap();

        doc.select_marquee(bounds, MarqueeMode::Contain).unwrap();
        assert!(doc.editor_state.selected_items.contains("n4"));
    }

    #[test]
    fn should_successfully_process_3000_node_grid_without_crashing() {
        let mut doc = DiagramDocument::default();
        for i in 0..3000 {
            let id = format!("n{i}");
            let x = (f64::from(i) * 10.0) % 1000.0;
            let y = (f64::from(i) * 10.0) / 1000.0 * 10.0;
            doc.document
                .nodes
                .insert(NodeId::new(id.clone()), create_node(&id, x, y, 5.0, 5.0));
        }

        let bounds = ValidRect::new(0.0, 0.0, 100.0, 100.0).unwrap();

        let doc_before = doc.clone();
        doc.select_marquee(bounds, MarqueeMode::Intersect).unwrap();

        assert!(!doc.editor_state.selected_items.is_empty());
        assert_eq!(doc.document, doc_before.document);
    }
}
