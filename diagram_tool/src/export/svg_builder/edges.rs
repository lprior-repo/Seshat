use super::styles;
use diagram_models::document::DiagramDocument;
use std::fmt::Write;

pub fn render_edges(doc: &DiagramDocument, svg: &mut String) {
    doc.document.edges.values().for_each(|edge| {
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
                "<line x1='{sx}' y1='{sy}' x2='{tx}' y2='{ty}' stroke='{}' stroke-width='{}' />",
                stroke_color, edge.thickness.0
            );
        }
    });
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

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_nan_coordinates_do_not_crash(_x in any::<f64>(), y in any::<f64>(), width in any::<f64>(), height in any::<f64>()) {
            let mut doc = create_test_document();
            let node = create_test_node("n1", f64::NAN, y, width, height, "NaN Node");
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_infinity_coordinates_do_not_crash(x in prop_oneof![Just(f64::INFINITY), Just(f64::NEG_INFINITY)]) {
            let mut doc = create_test_document();
            let node = create_test_node("n1", x, 0.0, 100.0, 50.0, "Inf Node");
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_extreme_coordinates_do_not_crash(coord in -1e300_f64..1e300_f64) {
            let mut doc = create_test_document();
            let node = create_test_node("n1", coord, coord, 100.0, 50.0, "Extreme");
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_negative_dimensions_do_not_crash(width in any::<f64>(), height in any::<f64>()) {
            let mut doc = create_test_document();
            let node = create_test_node("n1", 100.0, 100.0, width, height, "Negative");
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_zero_sized_nodes_do_not_crash(width in 0.0_f64..0.001, height in 0.0_f64..0.001) {
            let mut doc = create_test_document();
            let node = create_test_node("n1", 50.0, 50.0, width, height, "Tiny");
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_many_nodes_with_random_coords(
            nodes in prop::collection::vec((any::<f64>(), any::<f64>(), 1.0_f64..500.0, 1.0_f64..500.0), 1..20)
        ) {
            let mut doc = create_test_document();
            for (i, (x, y, w, h)) in nodes.into_iter().enumerate() {
                let node = create_test_node(&format!("n{}", i), x, y, w, h, "Node");
                doc.document.nodes.insert(node.0, node.1);
            }

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_edges_without_nodes_produce_valid_svg(edge_count in 0usize..10) {
            let mut doc = create_test_document();
            for i in 0..edge_count {
                let edge = (
                    EdgeId::new(format!("e{}", i)),
                    Edge {
                        source: NodeId::new(format!("src{}", i)),
                        target: NodeId::new(format!("tgt{}", i)),
                        label: String::new(),
                        style: diagram_models::document::EdgeStyle::Solid,
                        arrow_type: diagram_models::document::ArrowType::Default,
                        label_offset_t: OrderedFloat(0.5),
                        color: None,
                        thickness: OrderedFloat(1.5),
                        directed: true,
                        bend_points: im::Vector::new(),
                        tags: im::Vector::new(),
                        metadata: HashMap::new(),
                        font_size: None,
                        source_port: None,
                        target_port: None,
                    },
                );
                doc.document.edges.insert(edge.0, edge.1);
            }

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
            let line_count = svg.matches("<line").count();
            prop_assert_eq!(line_count, 0, "No lines should render without nodes");
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_unicode_labels_do_not_crash(label in ".*") {
            let mut doc = create_test_document();
            let node = create_test_node("n1", 0.0, 0.0, 100.0, 50.0, &label);
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_label_with_special_xml_chars(label in "[<>&\\\"\\']{0,10}") {
            let mut doc = create_test_document();
            let node = create_test_node("n1", 0.0, 0.0, 100.0, 50.0, &label);
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_calculate_bounds_consistency(
            coords in prop::collection::vec((any::<f64>(), any::<f64>(), any::<f64>(), any::<f64>()), 1..10)
        ) {
            let mut doc = create_test_document();
            for (i, (x, y, w, h)) in coords.into_iter().enumerate() {
                let node = create_test_node(&format!("n{}", i), x, y, w, h, "Node");
                doc.document.nodes.insert(node.0, node.1);
            }

            let (min_x, min_y, max_x, max_y) = crate::export::svg::grid::calculate_bounds(&doc);

            if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
                if max_x >= min_x && max_y >= min_y {
                    let svg = generate_svg_string(&doc);
                    prop_assert!(svg.starts_with("<svg"));
                }
            }
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_subnormal_floats(width in any::<f64>(), height in any::<f64>()) {
            let subnormal = f64::from_bits(1);
            let mut doc = create_test_document();
            let node = create_test_node("n1", subnormal, subnormal, width, height, "Subnormal");
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_floating_point_edge_cases(val in prop_oneof![
            Just(f64::NAN),
            Just(f64::INFINITY),
            Just(f64::NEG_INFINITY),
            Just(f64::MAX),
            Just(f64::MIN),
            Just(f64::MIN_POSITIVE),
            Just(0.0_f64),
            Just(-0.0_f64),
            Just(f64::EPSILON),
        ]) {
            let mut doc = create_test_document();
            let node = create_test_node("n1", val, val, val.abs().max(1.0), val.abs().max(1.0), "Edge");
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_coordinate_near_max(coord in (f64::MAX / 2.0)..f64::MAX) {
            let mut doc = create_test_document();
            let node = create_test_node("n1", coord, coord, 100.0, 50.0, "NearMax");
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_very_long_label(len in 0usize..10000) {
            let label = "X".repeat(len);
            let mut doc = create_test_document();
            let node = create_test_node("n1", 0.0, 0.0, 100.0, 50.0, &label);
            doc.document.nodes.insert(node.0, node.1);

            let svg = generate_svg_string(&doc);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }
    }
}
