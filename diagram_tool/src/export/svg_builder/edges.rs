use super::styles;
use diagram_models::document::DiagramDocument;
use std::fmt::Write;

pub fn render_edges(doc: &DiagramDocument, svg: &mut String) {
    for edge in doc.document.edges.values() {
        if let Some((src, tgt)) = doc
            .document
            .nodes
            .get(&edge.source)
            .zip(doc.document.nodes.get(&edge.target))
        {
            let sx = src.x.0 + src.width.0 / 2.0;
            let sy = src.y.0 + src.height.0 / 2.0;
            let tx = tgt.x.0 + tgt.width.0 / 2.0;
            let ty = tgt.y.0 + tgt.height.0 / 2.0;
            let stroke_color = styles::get_edge_stroke_color(edge);
            let _ = write!(
                svg,
                "<line x1='{sx}' y1='{sy}' x2='{tx}' y2='{ty}' stroke='{stroke_color}' stroke-width='{}' />",
                edge.thickness.0
            );
        }
    }
}

#[cfg(test)]
mod proptests {
    use crate::export::svg::generate_svg_string;
    use diagram_models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, LockState, Node, NodeId, NodeKind,
        OrderedFloat, Revision,
    };
    use im::HashMap;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseError;

    fn create_test_node(
        id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        label: &str,
    ) -> (NodeId, Node) {
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
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        )
    }

    fn create_test_document() -> DiagramDocument {
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

    fn verify_svg(doc: &DiagramDocument) -> Result<(), TestCaseError> {
        let svg = generate_svg_string(doc);
        prop_assert!(svg.starts_with("<svg"));
        prop_assert!(svg.ends_with("</svg>"));
        Ok(())
    }

    fn verify_node(
        id: &str,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        label: &str,
    ) -> Result<(), TestCaseError> {
        let mut doc = create_test_document();
        let (id, node) = create_test_node(id, x, y, w, h, label);
        doc.document.nodes.insert(id, node);
        verify_svg(&doc)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_nan_coordinates(y in any::<f64>(), w in any::<f64>(), h in any::<f64>()) {
            verify_node("n1", f64::NAN, y, w, h, "NaN Node")?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_infinity_coordinates(x in prop_oneof![Just(f64::INFINITY), Just(f64::NEG_INFINITY)]) {
            verify_node("n1", x, 0.0, 100.0, 50.0, "Inf Node")?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_extreme_coordinates(coord in -1e300_f64..1e300_f64) {
            verify_node("n1", coord, coord, 100.0, 50.0, "Extreme")?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_negative_dimensions(w in any::<f64>(), h in any::<f64>()) {
            verify_node("n1", 100.0, 100.0, w, h, "Negative")?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_zero_sized_nodes(w in 0.0_f64..0.001, h in 0.0_f64..0.001) {
            verify_node("n1", 50.0, 50.0, w, h, "Tiny")?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_many_nodes(nodes in prop::collection::vec((any::<f64>(), any::<f64>(), 1.0_f64..500.0, 1.0_f64..500.0), 1..20)) {
            let mut doc = create_test_document();
            for (i, (x, y, w, h)) in nodes.into_iter().enumerate() {
                let (id, node) = create_test_node(&format!("n{i}"), x, y, w, h, "Node");
                doc.document.nodes.insert(id, node);
            }
            verify_svg(&doc)?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_edges_without_nodes(edge_count in 0usize..10) {
            let mut doc = create_test_document();
            for i in 0..edge_count {
                let edge = Edge {
                    source: NodeId::new(format!("src{i}")), target: NodeId::new(format!("tgt{i}")),
                    label: String::new(), style: diagram_models::document::EdgeStyle::Solid,
                    arrow_type: diagram_models::document::ArrowType::Default, label_offset_t: OrderedFloat(0.5),
                    color: None, thickness: OrderedFloat(1.5), directed: true, bend_points: im::Vector::new(),
                    tags: im::Vector::new(), metadata: HashMap::new(), font_size: None, source_port: None, target_port: None,
                };
                doc.document.edges.insert(EdgeId::new(format!("e{i}")), edge);
            }
            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
            prop_assert_eq!(svg.matches("<line").count(), 0);
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_unicode_labels(label in ".*") { verify_node("n1", 0.0, 0.0, 100.0, 50.0, &label)?; }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_special_xml_chars(label in "[<>&\\\"\\']{0,10}") { verify_node("n1", 0.0, 0.0, 100.0, 50.0, &label)?; }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_calculate_bounds(coords in prop::collection::vec((any::<f64>(), any::<f64>(), any::<f64>(), any::<f64>()), 1..10)) {
            let mut doc = create_test_document();
            for (i, (x, y, w, h)) in coords.into_iter().enumerate() {
                let (id, node) = create_test_node(&format!("n{i}"), x, y, w, h, "Node");
                doc.document.nodes.insert(id, node);
            }
            let (min_x, min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);
            if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() && max_x >= min_x && max_y >= min_y {
                let svg = generate_svg_string(&doc);
                prop_assert!(svg.starts_with("<svg"));
            }
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_subnormal_floats(w in any::<f64>(), h in any::<f64>()) {
            verify_node("n1", f64::from_bits(1), f64::from_bits(1), w, h, "Subnormal")?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_floating_point_edge_cases(val in prop_oneof![
            Just(f64::NAN), Just(f64::INFINITY), Just(f64::NEG_INFINITY), Just(f64::MAX),
            Just(f64::MIN), Just(f64::MIN_POSITIVE), Just(0.0_f64), Just(-0.0_f64), Just(f64::EPSILON)
        ]) {
            verify_node("n1", val, val, val.abs().max(1.0), val.abs().max(1.0), "Edge")?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_coordinate_near_max(coord in (f64::MAX / 2.0)..f64::MAX) {
            verify_node("n1", coord, coord, 100.0, 50.0, "NearMax")?;
        }

        #[cfg(kani)] #[kani::proof] #[test] #[allow(clippy::unwrap_used)]
        fn prop_very_long_label(len in 0usize..10000) {
            verify_node("n1", 0.0, 0.0, 100.0, 50.0, &"X".repeat(len))?;
        }
    }
}
