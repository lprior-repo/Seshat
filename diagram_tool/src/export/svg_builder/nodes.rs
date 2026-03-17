use super::fonts;
use crate::icons::ICONS;
use base64::Engine;
use diagram_models::document::DiagramDocument;
use std::fmt::Write;

pub fn render_nodes(doc: &DiagramDocument, svg: &mut String) {
    let mut nodes: Vec<_> = doc.document.nodes.values().collect();
    nodes.sort_by_key(|node| node.z_index);

    for node in &nodes {
        let _ = write!(svg, "<g transform='translate({}, {})'>", node.x.0, node.y.0);
        let _ = write!(
            svg,
            "<rect width='{}' height='{}' fill='white' stroke='black' rx='4' ry='4'/>",
            node.width.0, node.height.0
        );

        if let Some(file) = ICONS.get_file(&node.icon) {
            let b64 = base64::engine::general_purpose::STANDARD.encode(file.contents());
            let icon_size = 32.0;
            let ix = (node.width.0 - icon_size) / 2.0;
            let iy = (node.height.0 - icon_size) / 2.0 - 5.0;
            let _ = write!(
                svg,
                "<image href='data:image/png;base64,{b64}' width='{icon_size}' height='{icon_size}' x='{ix}' y='{iy}' />"
            );
        }

        fonts::render_text(svg, node.width.0 / 2.0, node.height.0 - 5.0, &node.label);
        let _ = write!(svg, "</g>");
    }
}
