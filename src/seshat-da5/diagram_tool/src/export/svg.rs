#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::icons::ICONS;
use crate::models::document::DiagramDocument;
use base64::Engine;
use std::fmt::Write;

/// Escape a string for safe inclusion in XML/SVG content
#[must_use]
fn xml_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

/// Pure function to generate SVG string from document.
#[must_use]
pub fn generate_svg_string(doc: &DiagramDocument) -> String {
    let (min_x, min_y, max_x, max_y) = calculate_bounds(doc);

    let margin = 50.0;
    let view_min_x = min_x - margin;
    let view_min_y = min_y - margin;
    let width = 2.0f64.mul_add(margin, max_x - min_x).max(100.0);
    let height = 2.0f64.mul_add(margin, max_y - min_y).max(100.0);

    let mut svg = String::new();
    let _ = write!(
        &mut svg,
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='{view_min_x} {view_min_y} {width} {height}' width='{width}' height='{height}'>"
    );

    // Edges (rendered first, below nodes)
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
            let stroke_color = edge
                .color
                .as_deref()
                .map_or_else(|| "black".to_string(), xml_escape);
            let _ = write!(
                &mut svg,
                "<line x1='{sx}' y1='{sy}' x2='{tx}' y2='{ty}' stroke='{}' stroke-width='{}' />",
                stroke_color, edge.thickness.0
            );
        }
    });

    // Nodes sorted by z_index for proper layering
    let mut nodes: Vec<_> = doc.document.nodes.values().collect();
    nodes.sort_by_key(|node| node.z_index);

    for node in &nodes {
        let _ = write!(
            &mut svg,
            "<g transform='translate({}, {})'>",
            node.x.0, node.y.0
        );
        let _ = write!(
            &mut svg,
            "<rect width='{}' height='{}' fill='white' stroke='black' rx='4' ry='4'/>",
            node.width.0, node.height.0
        );

        if let Some(file) = ICONS.get_file(&node.icon) {
            let b64 = base64::engine::general_purpose::STANDARD.encode(file.contents());
            let icon_size = 32.0;
            let ix = (node.width.0 - icon_size) / 2.0;
            let iy = (node.height.0 - icon_size) / 2.0 - 5.0;
            let _ = write!(
                &mut svg,
                "<image href='data:image/png;base64,{b64}' width='{icon_size}' height='{icon_size}' x='{ix}' y='{iy}' />"
            );
        }

        let escaped_label = xml_escape(&node.label);
        let _ = write!(
            &mut svg,
            "<text x='{}' y='{}' text-anchor='middle' font-family='sans-serif' font-size='10'>{}</text>",
            node.width.0 / 2.0,
            node.height.0 - 5.0,
            escaped_label
        );
        let _ = write!(&mut svg, "</g>");
    }

    let _ = write!(&mut svg, "</svg>");
    svg
}

fn calculate_bounds(doc: &DiagramDocument) -> (f64, f64, f64, f64) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, Node, NodeId, NodeKind, OrderedFloat, Revision,
    };
    use anyhow::Result;
    use im::HashMap;

    fn create_node(
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
                locked: false,
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

    fn create_edge(id: &str, source: &str, target: &str) -> (EdgeId, Edge) {
        (
            EdgeId::new(id.to_string()),
            Edge {
                source: NodeId::new(source.to_string()),
                target: NodeId::new(target.to_string()),
                label: String::new(),
                style: crate::models::document::EdgeStyle::Solid,
                arrow_type: crate::models::document::ArrowType::Default,
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::Vector::new(),
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                font_size: None,
            },
        )
    }

    fn create_empty_document() -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: crate::models::document::EditorState::default(),
        }
    }

    // ============== calculate_bounds tests ==============

    #[test]
    fn given_empty_document_when_calculate_bounds_then_returns_default_bounds() -> Result<()> {
        // Given
        let doc = create_empty_document();

        // When
        let (min_x, min_y, max_x, max_y) = calculate_bounds(&doc);

        // Then
        assert_eq!(min_x, 0.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_x, 800.0);
        assert_eq!(max_y, 600.0);
        Ok(())
    }

    #[test]
    fn given_single_node_at_origin_when_calculate_bounds_then_returns_node_bounds() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Test");
        doc.document.nodes.insert(id, node);

        // When
        let (min_x, min_y, max_x, max_y) = calculate_bounds(&doc);

        // Then - bounds should include full node extent (x + width, y + height)
        assert_eq!(min_x, 0.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_x, 100.0, "max_x should be x + width = 0 + 100");
        assert_eq!(max_y, 50.0, "max_y should be y + height = 0 + 50");
        Ok(())
    }

    #[test]
    fn given_single_node_with_positive_coords_when_calculate_bounds_then_returns_node_bounds(
    ) -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 200.0, 150.0, 100.0, 80.0, "Test");
        doc.document.nodes.insert(id, node);

        // When
        let (min_x, min_y, max_x, max_y) = calculate_bounds(&doc);

        // Then
        assert_eq!(min_x, 200.0);
        assert_eq!(min_y, 150.0);
        assert_eq!(max_x, 300.0, "max_x should be 200 + 100 = 300");
        assert_eq!(max_y, 230.0, "max_y should be 150 + 80 = 230");
        Ok(())
    }

    #[test]
    fn given_two_nodes_when_calculate_bounds_then_returns_combined_bounds() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 100.0, 100.0, 50.0, 50.0, "Node1");
        let (id2, node2) = create_node("n2", 200.0, 300.0, 60.0, 40.0, "Node2");
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);

        // When
        let (min_x, min_y, max_x, max_y) = calculate_bounds(&doc);

        // Then
        assert_eq!(min_x, 100.0, "min_x should be minimum x of all nodes");
        assert_eq!(min_y, 100.0, "min_y should be minimum y of all nodes");
        assert_eq!(
            max_x, 260.0,
            "max_x should be max of (100+50, 200+60) = 260"
        );
        assert_eq!(
            max_y, 340.0,
            "max_y should be max of (100+50, 300+40) = 340"
        );
        Ok(())
    }

    #[test]
    fn given_nodes_with_negative_coords_when_calculate_bounds_then_handles_negative_values(
    ) -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", -100.0, -50.0, 200.0, 100.0, "Test");
        doc.document.nodes.insert(id, node);

        // When
        let (min_x, min_y, max_x, max_y) = calculate_bounds(&doc);

        // Then
        assert_eq!(min_x, -100.0);
        assert_eq!(min_y, -50.0);
        assert_eq!(max_x, 100.0, "max_x should be -100 + 200 = 100");
        assert_eq!(max_y, 50.0, "max_y should be -50 + 100 = 50");
        Ok(())
    }

    #[test]
    fn given_overlapping_nodes_when_calculate_bounds_then_returns_union_bounds() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 100.0, 100.0, 200.0, 200.0, "Big");
        let (id2, node2) = create_node("n2", 150.0, 150.0, 50.0, 50.0, "Small");
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);

        // When
        let (min_x, min_y, max_x, max_y) = calculate_bounds(&doc);

        // Then - should be union of both nodes
        assert_eq!(min_x, 100.0);
        assert_eq!(min_y, 100.0);
        assert_eq!(
            max_x, 300.0,
            "max_x should be max of (100+200, 150+50) = 300"
        );
        assert_eq!(
            max_y, 300.0,
            "max_y should be max of (100+200, 150+50) = 300"
        );
        Ok(())
    }

    // ============== generate_svg_string tests ==============

    #[test]
    fn given_empty_document_when_generate_svg_string_then_contains_valid_svg_structure(
    ) -> Result<()> {
        // Given
        let doc = create_empty_document();

        // When
        let svg = generate_svg_string(&doc);

        // Then
        assert!(svg.starts_with("<svg"), "Should start with svg tag");
        assert!(svg.ends_with("</svg>"), "Should end with closing svg tag");
        assert!(svg.contains("xmlns='http://www.w3.org/2000/svg'"));
        assert!(svg.contains("viewBox="));
        Ok(())
    }

    #[test]
    fn given_empty_document_when_generate_svg_string_then_uses_default_viewbox() -> Result<()> {
        // Given
        let doc = create_empty_document();

        // When
        let svg = generate_svg_string(&doc);

        // Then - empty doc uses default bounds (0, 0, 800, 600) with margin
        // view_min_x = 0 - 50 = -50, view_min_y = 0 - 50 = -50
        // width = 2*50 + (800-0) = 900, height = 2*50 + (600-0) = 700
        assert!(svg.contains("viewBox='-50 -50 900 700'"));
        Ok(())
    }

    #[test]
    fn given_single_node_when_generate_svg_string_then_viewbox_contains_node_with_margin(
    ) -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 100.0, 100.0, 100.0, 50.0, "Test");
        doc.document.nodes.insert(id, node);

        // When
        let svg = generate_svg_string(&doc);

        // Then
        // Bounds: min_x=100, min_y=100, max_x=200, max_y=150
        // view_min_x = 100 - 50 = 50, view_min_y = 100 - 50 = 50
        // width = 2*50 + (200-100) = 200, height = 2*50 + (150-100) = 150
        assert!(svg.contains("viewBox='50 50 200 150'"));
        assert!(svg.contains("width='200'"));
        assert!(svg.contains("height='150'"));
        Ok(())
    }

    #[test]
    fn given_node_when_generate_svg_string_then_contains_node_rect() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 100.0, 100.0, 120.0, 80.0, "MyNode");
        doc.document.nodes.insert(id, node);

        // When
        let svg = generate_svg_string(&doc);

        // Then
        assert!(svg.contains("<rect width='120' height='80'"));
        assert!(svg.contains("<text"));
        assert!(svg.contains(">MyNode<"));
        Ok(())
    }

    #[test]
    fn given_node_when_generate_svg_string_then_transform_uses_node_position() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 250.0, 175.0, 100.0, 50.0, "Test");
        doc.document.nodes.insert(id, node);

        // When
        let svg = generate_svg_string(&doc);

        // Then - transform should use exact x, y coordinates
        assert!(svg.contains("transform='translate(250, 175)'"));
        Ok(())
    }

    #[test]
    fn given_edge_between_nodes_when_generate_svg_string_then_line_connects_centers() -> Result<()>
    {
        // Given
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Source");
        let (id2, node2) = create_node("n2", 200.0, 100.0, 100.0, 50.0, "Target");
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);

        let (edge_id, edge) = create_edge("e1", "n1", "n2");
        doc.document.edges.insert(edge_id, edge);

        // When
        let svg = generate_svg_string(&doc);

        // Then - line should connect node centers
        // Source center: (0 + 100/2, 0 + 50/2) = (50, 25)
        // Target center: (200 + 100/2, 100 + 50/2) = (250, 125)
        assert!(svg.contains("<line"));
        assert!(svg.contains("x1='50'"), "x1 should be source center x = 50");
        assert!(svg.contains("y1='25'"), "y1 should be source center y = 25");
        assert!(
            svg.contains("x2='250'"),
            "x2 should be target center x = 250"
        );
        assert!(
            svg.contains("y2='125'"),
            "y2 should be target center y = 125"
        );
        Ok(())
    }

    #[test]
    fn given_edge_with_offset_nodes_when_generate_svg_string_then_line_uses_correct_arithmetic(
    ) -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 50.0, 75.0, 80.0, 40.0, "A");
        let (id2, node2) = create_node("n2", 300.0, 250.0, 120.0, 60.0, "B");
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);

        let (edge_id, edge) = create_edge("e1", "n1", "n2");
        doc.document.edges.insert(edge_id, edge);

        // When
        let svg = generate_svg_string(&doc);

        // Then - verify exact arithmetic for center calculation
        // Source center: (50 + 80/2, 75 + 40/2) = (50 + 40, 75 + 20) = (90, 95)
        // Target center: (300 + 120/2, 250 + 60/2) = (300 + 60, 250 + 30) = (360, 280)
        assert!(svg.contains("x1='90'"), "x1 should be 50 + 80/2 = 90");
        assert!(svg.contains("y1='95'"), "y1 should be 75 + 40/2 = 95");
        assert!(svg.contains("x2='360'"), "x2 should be 300 + 120/2 = 360");
        assert!(svg.contains("y2='280'"), "y2 should be 250 + 60/2 = 280");
        Ok(())
    }

    #[test]
    fn given_edge_with_missing_source_node_when_generate_svg_string_then_skips_edge() -> Result<()>
    {
        // Given
        let mut doc = create_empty_document();
        let (id2, node2) = create_node("n2", 200.0, 100.0, 100.0, 50.0, "Target");
        doc.document.nodes.insert(id2, node2);

        let (edge_id, edge) = create_edge("e1", "missing", "n2");
        doc.document.edges.insert(edge_id, edge);

        // When
        let svg = generate_svg_string(&doc);

        // Then - no line should be rendered for edge with missing source
        assert!(!svg.contains("<line"));
        Ok(())
    }

    #[test]
    fn given_edge_with_missing_target_node_when_generate_svg_string_then_skips_edge() -> Result<()>
    {
        // Given
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Source");
        doc.document.nodes.insert(id1, node1);

        let (edge_id, edge) = create_edge("e1", "n1", "missing");
        doc.document.edges.insert(edge_id, edge);

        // When
        let svg = generate_svg_string(&doc);

        // Then - no line should be rendered for edge with missing target
        assert!(!svg.contains("<line"));
        Ok(())
    }

    #[test]
    fn given_small_content_when_generate_svg_string_then_enforces_minimum_dimensions() -> Result<()>
    {
        // Given - node at origin with small dimensions
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 0.0, 0.0, 10.0, 10.0, "Tiny");
        doc.document.nodes.insert(id, node);

        // When
        let svg = generate_svg_string(&doc);

        // Then - width/height should be at least 100
        // Raw: width = 2*50 + (10-0) = 110, height = 2*50 + (10-0) = 110
        // Both are > 100, so should be used as-is
        assert!(svg.contains("width='110'"));
        assert!(svg.contains("height='110'"));
        Ok(())
    }

    #[test]
    fn given_wide_document_when_generate_svg_string_then_viewbox_reflects_width() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Left");
        let (id2, node2) = create_node("n2", 1000.0, 0.0, 100.0, 50.0, "Right");
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);

        // When
        let svg = generate_svg_string(&doc);

        // Then
        // Bounds: min_x=0, max_x=1100, min_y=0, max_y=50
        // width = 2*50 + (1100-0) = 1200
        assert!(svg.contains("width='1200'"));
        Ok(())
    }

    #[test]
    fn given_tall_document_when_generate_svg_string_then_viewbox_reflects_height() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "Top");
        let (id2, node2) = create_node("n2", 0.0, 800.0, 100.0, 50.0, "Bottom");
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);

        // When
        let svg = generate_svg_string(&doc);

        // Then
        // Bounds: min_y=0, max_y=850
        // height = 2*50 + (850-0) = 950
        assert!(svg.contains("height='950'"));
        Ok(())
    }

    #[test]
    fn given_node_with_exact_position_when_generate_svg_string_then_text_is_centered() -> Result<()>
    {
        // Given
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 100.0, 100.0, 200.0, 100.0, "Label");
        doc.document.nodes.insert(id, node);

        // When
        let svg = generate_svg_string(&doc);

        // Then - text x should be width/2 = 100, y should be height - 5 = 95
        assert!(
            svg.contains("text x='100'"),
            "text x should be width/2 = 200/2 = 100"
        );
        assert!(
            svg.contains("y='95'"),
            "text y should be height - 5 = 100 - 5 = 95"
        );
        Ok(())
    }

    #[test]
    fn given_multiple_edges_when_generate_svg_string_then_all_edges_rendered() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 0.0, 0.0, 100.0, 50.0, "A");
        let (id2, node2) = create_node("n2", 200.0, 0.0, 100.0, 50.0, "B");
        let (id3, node3) = create_node("n3", 100.0, 150.0, 100.0, 50.0, "C");
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);
        doc.document.nodes.insert(id3, node3);

        let (e1_id, e1) = create_edge("e1", "n1", "n2");
        let (e2_id, e2) = create_edge("e2", "n2", "n3");
        let (e3_id, e3) = create_edge("e3", "n3", "n1");
        doc.document.edges.insert(e1_id, e1);
        doc.document.edges.insert(e2_id, e2);
        doc.document.edges.insert(e3_id, e3);

        // When
        let svg = generate_svg_string(&doc);

        // Then - should have 3 lines
        let line_count = svg.matches("<line").count();
        assert_eq!(line_count, 3, "Should have 3 lines for 3 edges");
        Ok(())
    }

    #[test]
    fn given_viewbox_margin_when_generate_svg_string_then_subtracts_50_from_bounds() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 100.0, 200.0, 100.0, 50.0, "Test");
        doc.document.nodes.insert(id, node);

        // When
        let svg = generate_svg_string(&doc);

        // Then - view_min_x = 100 - 50 = 50, view_min_y = 200 - 50 = 150
        assert!(
            svg.contains("viewBox='50 150"),
            "viewBox should start at (min_x-50, min_y-50)"
        );
        Ok(())
    }

    #[test]
    fn given_node_extent_calculation_when_calculate_bounds_then_adds_width_and_height() -> Result<()>
    {
        // Given - node at (100, 200) with size (150, 80)
        let mut doc = create_empty_document();
        let (id, node) = create_node("n1", 100.0, 200.0, 150.0, 80.0, "Test");
        doc.document.nodes.insert(id, node);

        // When
        let (_min_x, _min_y, max_x, max_y) = calculate_bounds(&doc);

        // Then - max values should be position + dimension
        assert_eq!(max_x, 250.0, "max_x should be 100 + 150 = 250");
        assert_eq!(max_y, 280.0, "max_y should be 200 + 80 = 280");
        Ok(())
    }

    #[test]
    fn given_center_calculation_when_edge_rendered_then_uses_division_by_2() -> Result<()> {
        // Given - odd width/height to verify division
        let mut doc = create_empty_document();
        let (id1, node1) = create_node("n1", 0.0, 0.0, 99.0, 77.0, "A");
        let (id2, node2) = create_node("n2", 0.0, 0.0, 201.0, 303.0, "B");
        doc.document.nodes.insert(id1, node1);
        doc.document.nodes.insert(id2, node2);

        let (edge_id, edge) = create_edge("e1", "n1", "n2");
        doc.document.edges.insert(edge_id, edge);

        // When
        let svg = generate_svg_string(&doc);

        // Then - centers should use exact division
        // Node1 center: (0 + 99/2, 0 + 77/2) = (49.5, 38.5)
        // Node2 center: (0 + 201/2, 0 + 303/2) = (100.5, 151.5)
        assert!(svg.contains("x1='49.5'"), "x1 should be 99/2 = 49.5");
        assert!(svg.contains("y1='38.5'"), "y1 should be 77/2 = 38.5");
        assert!(svg.contains("x2='100.5'"), "x2 should be 201/2 = 100.5");
        assert!(svg.contains("y2='151.5'"), "y2 should be 303/2 = 151.5");
        Ok(())
    }

    #[test]
    fn given_node_with_icon_when_generate_svg_string_then_icon_is_centered_horizontally(
    ) -> Result<()> {
        // Given - node with known width and an icon that exists
        let mut doc = create_empty_document();
        let mut node = create_node("n1", 0.0, 0.0, 100.0, 60.0, "Test");
        node.1.icon = String::from("aws/compute/ec2.png"); // Known icon
        doc.document.nodes.insert(node.0, node.1);

        // When
        let svg = generate_svg_string(&doc);

        // Then - icon should be centered: ix = (width - 32) / 2 = (100 - 32) / 2 = 34
        assert!(
            svg.contains("x='34"),
            "icon x should be (100 - 32) / 2 = 34"
        );
        Ok(())
    }

    #[test]
    fn given_node_with_icon_when_generate_svg_string_then_icon_is_centered_vertically_with_offset(
    ) -> Result<()> {
        // Given - node with known height and an icon that exists
        let mut doc = create_empty_document();
        let mut node = create_node("n1", 0.0, 0.0, 100.0, 60.0, "Test");
        node.1.icon = String::from("aws/compute/ec2.png"); // Known icon
        doc.document.nodes.insert(node.0, node.1);

        // When
        let svg = generate_svg_string(&doc);

        // Then - icon y = (height - 32) / 2 - 5 = (60 - 32) / 2 - 5 = 14 - 5 = 9
        assert!(
            svg.contains("y='9"),
            "icon y should be (60 - 32) / 2 - 5 = 9"
        );
        Ok(())
    }

    #[test]
    fn given_node_with_large_dimensions_when_generate_svg_string_then_icon_position_uses_subtraction(
    ) -> Result<()> {
        // Given - node with 200x100 dimensions
        let mut doc = create_empty_document();
        let mut node = create_node("n1", 0.0, 0.0, 200.0, 100.0, "Test");
        node.1.icon = String::from("aws/compute/ec2.png");
        doc.document.nodes.insert(node.0, node.1);

        // When
        let svg = generate_svg_string(&doc);

        // Then
        // ix = (200 - 32) / 2 = 84
        // iy = (100 - 32) / 2 - 5 = 34 - 5 = 29
        assert!(
            svg.contains("x='84"),
            "icon x should be (200 - 32) / 2 = 84"
        );
        assert!(
            svg.contains("y='29"),
            "icon y should be (100 - 32) / 2 - 5 = 29"
        );
        Ok(())
    }

    #[test]
    fn given_node_with_icon_when_generate_svg_string_then_icon_size_is_32() -> Result<()> {
        // Given
        let mut doc = create_empty_document();
        let mut node = create_node("n1", 0.0, 0.0, 100.0, 60.0, "Test");
        node.1.icon = String::from("aws/compute/ec2.png");
        doc.document.nodes.insert(node.0, node.1);

        // When
        let svg = generate_svg_string(&doc);

        // Then - icon should have width and height of 32
        assert!(svg.contains("width='32"), "icon width should be 32");
        assert!(svg.contains("height='32"), "icon height should be 32");
        Ok(())
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, Node, NodeId, NodeKind, OrderedFloat, Revision,
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
                locked: false,
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
            editor_state: crate::models::document::EditorState::default(),
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

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
                        style: crate::models::document::EdgeStyle::Solid,
                        arrow_type: crate::models::document::ArrowType::Default,
                        label_offset_t: OrderedFloat(0.5),
                        color: None,
                        thickness: OrderedFloat(1.5),
                        directed: true,
                        bend_points: im::Vector::new(),
                        tags: im::Vector::new(),
                        metadata: HashMap::new(),
                        font_size: None,
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

            let (min_x, min_y, max_x, max_y) = calculate_bounds(&doc);

            if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
                if max_x >= min_x && max_y >= min_y {
                    let svg = generate_svg_string(&doc);
                    prop_assert!(svg.starts_with("<svg"));
                }
            }
        }

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

/// IO Tests for bd-1u1: Export Image Bounds Match and Export with Rotated Items
#[cfg(test)]
mod io_tests {
    use super::*;
    use crate::models::document::{
        DiagramDocument, DocumentData, Node, NodeId, NodeKind, OrderedFloat, Revision,
    };
    use anyhow::Result;
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
                locked: false,
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
            editor_state: crate::models::document::EditorState::default(),
        }
    }

    /// IO-TEST-1: Export Image Bounds Match
    /// Given: A document with nodes at specific positions
    /// When: Exporting to SVG
    /// Then: The exported image bounds match the calculated document bounds (with margin)
    #[test]
    fn given_document_with_nodes_when_export_svg_then_bounds_match_with_margin() -> Result<()> {
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
    #[test]
    fn given_empty_document_when_export_svg_then_uses_default_bounds() -> Result<()> {
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
    #[test]
    fn given_node_with_rotation_metadata_when_export_svg_then_succeeds() -> Result<()> {
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
    #[test]
    fn given_multiple_rotated_nodes_when_export_svg_then_succeeds() -> Result<()> {
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
    #[test]
    fn given_node_with_negative_rotation_when_export_svg_then_succeeds() -> Result<()> {
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
