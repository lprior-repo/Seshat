use diagram_models::document::DiagramDocument;

pub fn calculate_bounds(doc: &DiagramDocument) -> (f64, f64, f64, f64) {
    if doc.document.nodes.is_empty() {
        (0.0, 0.0, 800.0, 600.0)
    } else {
        doc.document.nodes.values().fold(
            (f64::MAX, f64::MAX, f64::MIN, f64::MIN),
            |(min_x, min_y, max_x, max_y), node| {
                (
                    min_x.min(node.x.0),
                    min_y.min(node.y.0),
                    max_x.max(node.x.0 + node.width.0),
                    max_y.max(node.y.0 + node.height.0),
                )
            },
        )
    }
}

pub fn calculate_viewbox(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> (f64, f64, f64, f64) {
    let margin = 50.0;
    let view_min_x = min_x - margin;
    let view_min_y = min_y - margin;
    let width = 2.0f64.mul_add(margin, max_x - min_x).max(100.0);
    let height = 2.0f64.mul_add(margin, max_y - min_y).max(100.0);
    (view_min_x, view_min_y, width, height)
}

/// IO Tests for bd-1u1: Export Image Bounds Match and Export with Rotated Items
#[cfg(test)]
mod io_tests {
    use crate::export::svg::generate_svg_string;
    use diagram_models::document::{
        DiagramDocument, DocumentData, LockState, Node, NodeId, NodeKind, OrderedFloat, Revision,
    };
    use im::HashMap;

    fn create_io_node(
        id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        label: &str,
        rotation: Option<f64>,
    ) -> (NodeId, Node) {
        let mut metadata = HashMap::new();
        if let Some(rot) = rotation {
            let _ = metadata.insert("rotation".to_string(), serde_json::json!(rot));
        }
        (
            NodeId::new(id.to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: label.to_string(),
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                width: OrderedFloat(width),
                height: OrderedFloat(height),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
                metadata,
                z_index: 0,
                style: None,
                collapsed: None,
            },
        )
    }

    fn create_io_document() -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: diagram_models::document::EditorState::default(),
        }
    }

    /// IO-TEST-1: Export Image Bounds Match
    /// Given: A document with nodes at specific positions
    /// When: Exporting to SVG
    /// Then: The exported image bounds match the calculated document bounds (with margin)
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_nodes_when_export_svg_then_bounds_match_with_margin(
    ) -> Result<(), anyhow::Error> {
        // Given
        let mut doc = create_io_document();
        let (id1, node1) = create_io_node("n1", 100.0, 100.0, 80.0, 50.0, "Node1", None);
        let (id2, node2) = create_io_node("n2", 300.0, 200.0, 100.0, 60.0, "Node2", None);
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);

        // When
        let svg = generate_svg_string(&doc);

        // Then
        // Bounds: min_x=100, min_y=100, max_x=400 (300+100), max_y=260 (200+60)
        // view_min_x = 100 - 50 = 50, view_min_y = 100 - 50 = 50
        // width = 2*50 + (400-100) = 100 + 300 = 400
        // height = 2*50 + (260-100) = 100 + 160 = 260
        assert!(
            svg.contains("viewBox='50 50 400 260'"),
            "viewBox should match calculated bounds with margin"
        );
        assert!(
            svg.contains("width='400'"),
            "width should match calculated bounds"
        );
        assert!(
            svg.contains("height='260'"),
            "height should match calculated bounds"
        );
        Ok(())
    }

    /// IO-TEST-1b: Empty document uses default bounds
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_empty_document_when_export_svg_then_uses_default_bounds() -> Result<(), anyhow::Error>
    {
        // Given
        let doc = create_io_document();

        // When
        let svg = generate_svg_string(&doc);

        // Then - empty doc uses default bounds (0, 0, 800, 600) with 50 margin
        // view_min_x = 0 - 50 = -50, view_min_y = 0 - 50 = -50
        // width = 2*50 + (800-0) = 900, height = 2*50 + (600-0) = 700
        assert!(
            svg.contains("viewBox='-50 -50 900 700'"),
            "empty doc should use default bounds"
        );
        Ok(())
    }

    /// IO-TEST-2: Export with Rotated Items
    /// Given: A document containing nodes with rotation metadata
    /// When: Exporting to SVG
    /// Then: The export completes without crash and produces valid SVG
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_with_rotation_metadata_when_export_svg_then_succeeds() -> Result<(), anyhow::Error>
    {
        // Given
        let mut doc = create_io_document();
        let (id, node) = create_io_node(
            "rotated",
            100.0,
            100.0,
            80.0,
            50.0,
            "Rotated Node",
            Some(45.0),
        );
        doc.document.nodes.insert(id, node);

        // When
        let svg = generate_svg_string(&doc);

        // Then - should produce valid SVG without crash
        assert!(svg.starts_with("<svg"), "SVG should start with svg tag");
        assert!(svg.ends_with("</svg>"), "SVG should end with closing tag");
        assert!(
            svg.contains(">Rotated Node<"),
            "SVG should contain node label"
        );
        Ok(())
    }

    /// IO-TEST-2b: Multiple rotated items
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multiple_rotated_nodes_when_export_svg_then_succeeds() -> Result<(), anyhow::Error> {
        // Given
        let mut doc = create_io_document();
        let (id1, node1) = create_io_node("r1", 0.0, 0.0, 100.0, 50.0, "R0", Some(0.0));
        let (id2, node2) = create_io_node("r2", 150.0, 0.0, 100.0, 50.0, "R90", Some(90.0));
        let (id3, node3) = create_io_node("r3", 300.0, 0.0, 100.0, 50.0, "R180", Some(180.0));
        let (id4, node4) = create_io_node("r4", 450.0, 0.0, 100.0, 50.0, "R270", Some(270.0));
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);
        doc.document.nodes.insert(id3, node3);
        doc.document.nodes.insert(id4, node4);

        // When
        let svg = generate_svg_string(&doc);

        // Then - should produce valid SVG with all nodes
        assert!(svg.starts_with("<svg"), "SVG should start with svg tag");
        assert!(svg.ends_with("</svg>"), "SVG should end with closing tag");
        assert!(svg.contains(">R0<"), "SVG should contain R0");
        assert!(svg.contains(">R90<"), "SVG should contain R90");
        assert!(svg.contains(">R180<"), "SVG should contain R180");
        assert!(svg.contains(">R270<"), "SVG should contain R270");
        Ok(())
    }

    /// IO-TEST-2c: Negative rotation
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_with_negative_rotation_when_export_svg_then_succeeds() -> Result<(), anyhow::Error>
    {
        // Given
        let mut doc = create_io_document();
        let (id, node) = create_io_node("neg_rot", 100.0, 100.0, 80.0, 50.0, "NegRot", Some(-45.0));
        doc.document.nodes.insert(id, node);

        // When
        let svg = generate_svg_string(&doc);

        // Then
        assert!(svg.starts_with("<svg"), "SVG should be valid");
        assert!(svg.ends_with("</svg>"), "SVG should be valid");
        Ok(())
    }
}
