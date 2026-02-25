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

    // Edges
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
            let _ = write!(
                &mut svg,
                "<line x1='{sx}' y1='{sy}' x2='{tx}' y2='{ty}' stroke='black' stroke-width='2' />"
            );
        }
    });

    // Nodes
    doc.document.nodes.values().for_each(|node| {
        let _ = write!(&mut svg, "<g transform='translate({}, {})'>", node.x.0, node.y.0);
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

        let _ = write!(
            &mut svg,
            "<text x='{}' y='{}' text-anchor='middle' font-family='sans-serif' font-size='10'>{}</text>",
            node.width.0 / 2.0,
            node.height.0 - 5.0,
            node.label
        );
        let _ = write!(&mut svg, "</g>");
    });

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
