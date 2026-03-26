use diagram_models::document::DiagramDocument;
use std::fmt::Write;

#[cfg(not(target_arch = "wasm32"))]
use base64::{engine::general_purpose, Engine as _};

use crate::export::svg::fonts;

/// Attempt to read an icon file and embed it as a base64 data-URL for portable SVG export.
/// Only available on native targets — WASM builds use URL-only href.
#[cfg(not(target_arch = "wasm32"))]
fn embed_icon_as_data_url(href: &str) -> Option<String> {
    let relpath = href.strip_prefix("/assets/resources/")?;
    // Reject paths with path traversal or absolute components
    if relpath.contains("..") || relpath.starts_with('/') {
        return None;
    }
    let full_path = std::path::Path::new("resources").join(relpath);
    // Verify the joined path stays within resources/ (defends against absolute path injection)
    let canonical = full_path.canonicalize().ok()?;
    let resources_dir = std::path::Path::new("resources").canonicalize().ok()?;
    if !canonical.starts_with(&resources_dir) {
        return None;
    }
    let bytes = std::fs::read(&canonical).ok()?;

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

        // On native targets, try to embed icon as base64 data-URL for portable SVG export.
        // On WASM targets, always use URL-only href (no filesystem access).
        #[cfg(not(target_arch = "wasm32"))]
        let image_href = embed_icon_as_data_url(&href).unwrap_or(href);
        #[cfg(target_arch = "wasm32")]
        let image_href = href;

        let icon_size = 32.0;
        let ix = (node.width.0 - icon_size) / 2.0;
        let iy = (node.height.0 - icon_size) / 2.0 - 5.0;
        let _ = write!(
            svg,
            "<image href='{}' width='{icon_size}' height='{icon_size}' x='{ix}' y='{iy}' />",
            image_href
        );

        fonts::render_text(svg, node.width.0 / 2.0, node.height.0 - 5.0, &node.label);
        let _ = write!(svg, "</g>");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
        assert!(
            svg.contains("/assets/resources/aws/ec2.png"),
            "SVG should contain the metadata icon_url path, got: {svg}"
        );
        assert!(
            !svg.contains("/assets/resources//assets/resources/"),
            "SVG must NOT double-prefix the icon URL, got: {svg}"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn embed_icon_as_data_url_returns_none_for_nonexistent_file() {
        let result = embed_icon_as_data_url("/assets/resources/nonexistent/icon.png");
        assert!(result.is_none(), "Non-existent file must return None");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn embed_icon_as_data_url_returns_none_for_traversal() {
        let result = embed_icon_as_data_url("/assets/resources/../../../etc/passwd");
        assert!(result.is_none(), "Path traversal must return None");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn embed_icon_as_data_url_returns_valid_data_url_for_existing_file() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let icon_dir = tmp.path().join("resources").join("generic").join("os");
        std::fs::create_dir_all(&icon_dir).expect("create dirs");
        std::fs::write(
            icon_dir.join("ubuntu.png"),
            [
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
                0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
                0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
                0x9C, 0x62, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE5, 0x27, 0xDE, 0xFC, 0x00, 0x00,
                0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
            ],
        )
        .expect("write png");

        struct CwdGuard(std::path::PathBuf);
        impl Drop for CwdGuard {
            fn drop(&mut self) {
                let _ = std::env::set_current_dir(&self.0);
            }
        }

        let guard = CwdGuard(std::env::current_dir().expect("cwd"));
        std::env::set_current_dir(tmp.path()).expect("chdir");

        let result = embed_icon_as_data_url("/assets/resources/generic/os/ubuntu.png");
        drop(guard);

        assert!(result.is_some(), "Existing file must return Some");
        let url = result.expect("should have url");
        assert!(
            url.starts_with("data:image/png;base64,"),
            "Must be a base64 data URL with PNG MIME type, got: {url}"
        );
        let b64_part = url
            .strip_prefix("data:image/png;base64,")
            .expect("should have prefix");
        assert!(!b64_part.is_empty(), "Base64 content must not be empty");
        use base64::{engine::general_purpose, Engine as _};
        let decoded = general_purpose::STANDARD
            .decode(b64_part)
            .expect("base64 must be valid");
        assert!(!decoded.is_empty(), "Decoded content must not be empty");
    }
}
