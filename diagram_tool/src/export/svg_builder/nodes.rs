use base64::{engine::general_purpose, Engine as _};
use diagram_models::document::DiagramDocument;
use std::fmt::Write;

use crate::export::svg::fonts;

/// Attempt to read an icon file and embed it as a base64 data-URL for portable SVG export.
/// Returns `None` if the file cannot be read (graceful degradation to URL href).
fn embed_icon_as_data_url(href: &str) -> Option<String> {
    // Extract the relative path from the URL (strip "/assets/resources/" prefix)
    let relpath = href.strip_prefix("/assets/resources/")?;
    let full_path = std::path::Path::new("resources").join(relpath);
    let bytes = std::fs::read(&full_path).ok()?;

    let ext = full_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let mime = if ext.eq_ignore_ascii_case("svg") {
        "image/svg+xml"
    } else {
        "image/png"
    };

    Some(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(&bytes)
    ))
}

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

        // icon_url metadata already contains the full URL path (e.g. "/assets/resources/aws/ec2.png")
        // node.icon is a bare key (e.g. "aws/analytics/athena.png") that needs the prefix
        let href = node
            .metadata
            .get("icon_url")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("/assets/resources/{}", node.icon));

        // Try to embed icon as base64 data-URL for portable SVG export
        let data_href = embed_icon_as_data_url(&href);

        let icon_size = 32.0;
        let ix = (node.width.0 - icon_size) / 2.0;
        let iy = (node.height.0 - icon_size) / 2.0 - 5.0;
        let _ = write!(
            svg,
            "<image href='{}' width='{icon_size}' height='{icon_size}' x='{ix}' y='{iy}' />",
            data_href.as_deref().unwrap_or(&href)
        );

        fonts::render_text(svg, node.width.0 / 2.0, node.height.0 - 5.0, &node.label);
        let _ = write!(svg, "</g>");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use diagram_models::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};

    fn make_test_node(
        icon: &str,
        label: &str,
        metadata: im::HashMap<String, serde_json::Value>,
    ) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: icon.to_string(),
            label: label.to_string(),
            x: OrderedFloat::new_unchecked(0.0),
            y: OrderedFloat::new_unchecked(0.0),
            width: OrderedFloat::new_unchecked(100.0),
            height: OrderedFloat::new_unchecked(60.0),
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
        }
    }

    #[test]
    fn render_nodes_produces_svg_with_image_element() {
        let node = make_test_node("aws/ec2.png", "EC2", im::HashMap::new());
        let mut doc = DiagramDocument::default();
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), node);
        let mut svg = String::new();
        render_nodes(&doc, &mut svg);
        assert!(
            svg.contains("<image"),
            "SVG should contain <image element, got: {svg}"
        );
    }

    #[test]
    fn render_nodes_uses_metadata_icon_url() {
        // metadata icon_url is already a full URL path (e.g. from drag-and-drop)
        // It must NOT be double-prefixed with /assets/resources/
        let metadata = im::HashMap::from(vec![(
            "icon_url".to_string(),
            serde_json::Value::String("/assets/resources/aws/ec2.png".to_string()),
        )]);
        let node = make_test_node("", "Test", metadata);
        let mut doc = DiagramDocument::default();
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), node);
        let mut svg = String::new();
        render_nodes(&doc, &mut svg);
        // Must contain the path exactly once (no double-prefix)
        assert!(
            svg.contains("/assets/resources/aws/ec2.png"),
            "SVG should contain the metadata icon_url path, got: {svg}"
        );
        // Must NOT double-prefix
        assert!(
            !svg.contains("/assets/resources//assets/resources/"),
            "SVG must NOT double-prefix the icon URL, got: {svg}"
        );
    }
}
