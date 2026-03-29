#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

#[path = "svg_builder/edges.rs"]
pub mod edges;
#[cfg(test)]
#[path = "svg_builder/edges_tests/mod.rs"]
pub mod edges_tests;
#[path = "svg_builder/fonts.rs"]
pub mod fonts;
#[path = "svg_builder/grid.rs"]
pub mod grid;
#[path = "svg_builder/nodes.rs"]
pub mod nodes;
#[cfg(test)]
#[path = "svg_builder/nodes_tests/mod.rs"]
pub mod nodes_tests;
#[path = "svg_builder/styles.rs"]
pub mod styles;

use diagram_models::document::DiagramDocument;
use std::fmt::Write;

/// Pure function to generate SVG string from document.
#[must_use]
pub fn generate_svg_string(doc: &DiagramDocument) -> String {
    let (min_x, min_y, max_x, max_y) = grid::calculate_bounds(doc);
    let (view_min_x, view_min_y, width, height) =
        grid::calculate_viewbox(min_x, min_y, max_x, max_y);

    let mut svg = String::new();
    let _ = write!(
        &mut svg,
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='{view_min_x} {view_min_y} {width} {height}' width='{width}' height='{height}'>"
    );

    edges::render_edges(doc, &mut svg);
    nodes::render_nodes(doc, &mut svg);

    let _ = write!(&mut svg, "</svg>");
    svg
}
