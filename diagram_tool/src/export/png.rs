#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::export::svg::generate_svg_string;
use crate::models::document::DiagramDocument;
use anyhow::{Context, Result};
use resvg::usvg;
use tiny_skia::{Pixmap, Transform};

/// Export document to PNG file.
///
/// # Errors
/// Returns an error if SVG generation or PNG encoding fails.
pub fn export_png(doc: &DiagramDocument, path: &str) -> Result<()> {
    let svg_data = generate_svg_string(doc);

    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_str(&svg_data, &opt).context("Failed to parse SVG")?;

    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).context("Failed to create pixmap")?;

    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());

    pixmap.save_png(path).context("Failed to save PNG")?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind,
        OrderedFloat,
    };
    use im::HashMap;
    use tempfile::NamedTempFile;

    /// PNG file magic bytes (signature)
    const PNG_SIGNATURE: &[u8; 8] = &[137, 80, 78, 71, 13, 10, 26, 10];

    fn create_test_node(id: &str, x: f64, y: f64) -> (NodeId, Node) {
        (
            NodeId::new(id.to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: format!("Node {id}"),
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                width: OrderedFloat(100.0),
                height: OrderedFloat(60.0),
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

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_empty_document_when_export_png_then_creates_valid_png_file() -> Result<()> {
        // Given
        let doc = DiagramDocument::default();
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        let output_path = temp_file.path().to_str().context("Invalid path")?;

        // When
        export_png(&doc, output_path)?;

        // Then
        let bytes = std::fs::read(output_path).context("Failed to read PNG file")?;
        assert!(bytes.len() > 8, "PNG file too small: {} bytes", bytes.len());
        assert_eq!(&bytes[0..8], PNG_SIGNATURE, "Invalid PNG signature");
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_single_node_when_export_png_then_creates_valid_png() -> Result<()> {
        // Given
        let (node_id, node) = create_test_node("node1", 50.0, 50.0);
        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);
        let doc = DiagramDocument {
            version: 2,
            revision: crate::models::document::Revision::INITIAL,
            document: DocumentData {
                nodes,
                edges: HashMap::new(),
            },
            editor_state: crate::models::document::EditorState::default(),
        };
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        let output_path = temp_file.path().to_str().context("Invalid path")?;

        // When
        export_png(&doc, output_path)?;

        // Then
        let bytes = std::fs::read(output_path).context("Failed to read PNG file")?;
        assert_eq!(&bytes[0..8], PNG_SIGNATURE, "Invalid PNG signature");
        // Verify file has reasonable size (at least a few KB for a simple PNG)
        assert!(
            bytes.len() > 100,
            "PNG file unexpectedly small: {} bytes",
            bytes.len()
        );
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_multiple_nodes_when_export_png_then_creates_valid_png() -> Result<()> {
        // Given
        let (node_id1, node1) = create_test_node("node1", 0.0, 0.0);
        let (node_id2, node2) = create_test_node("node2", 200.0, 0.0);
        let (node_id3, node3) = create_test_node("node3", 100.0, 150.0);
        let mut node_map = HashMap::new();
        node_map.insert(node_id1, node1);
        node_map.insert(node_id2, node2);
        node_map.insert(node_id3, node3);
        let doc = DiagramDocument {
            version: 2,
            revision: crate::models::document::Revision::INITIAL,
            document: DocumentData {
                nodes: node_map,
                edges: HashMap::new(),
            },
            editor_state: crate::models::document::EditorState::default(),
        };
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        let output_path = temp_file.path().to_str().context("Invalid path")?;

        // When
        export_png(&doc, output_path)?;

        // Then
        let bytes = std::fs::read(output_path).context("Failed to read PNG file")?;
        assert_eq!(&bytes[0..8], PNG_SIGNATURE, "Invalid PNG signature");
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_edges_when_export_png_then_creates_valid_png() -> Result<()> {
        // Given
        let (src_node_id, src_node) = create_test_node("source", 0.0, 50.0);
        let (tgt_node_id, tgt_node) = create_test_node("target", 200.0, 50.0);
        let mut node_map = HashMap::new();
        node_map.insert(src_node_id.clone(), src_node);
        node_map.insert(tgt_node_id.clone(), tgt_node);

        let edge = Edge {
            source: src_node_id,
            target: tgt_node_id,
            label: String::new(),
            style: EdgeStyle::Solid,
            arrow_type: crate::models::document::ArrowType::Default,
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
        };
        let mut edges = HashMap::new();
        edges.insert(EdgeId::new("edge1".to_string()), edge);

        let doc = DiagramDocument {
            version: 2,
            revision: crate::models::document::Revision::INITIAL,
            document: DocumentData {
                nodes: node_map,
                edges,
            },
            editor_state: crate::models::document::EditorState::default(),
        };
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        let output_path = temp_file.path().to_str().context("Invalid path")?;

        // When
        export_png(&doc, output_path)?;

        // Then
        let bytes = std::fs::read(output_path).context("Failed to read PNG file")?;
        assert_eq!(&bytes[0..8], PNG_SIGNATURE, "Invalid PNG signature");
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_document_when_export_png_then_file_exists_on_disk() -> Result<()> {
        // Given
        let doc = DiagramDocument::default();
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        let output_path = temp_file.path().to_str().context("Invalid path")?;

        // When
        export_png(&doc, output_path)?;

        // Then
        assert!(
            temp_file.path().exists(),
            "PNG file should exist at {output_path:?}"
        );
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_document_when_export_png_then_png_has_iend_chunk() -> Result<()> {
        // Given
        let doc = DiagramDocument::default();
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        let output_path = temp_file.path().to_str().context("Invalid path")?;

        // When
        export_png(&doc, output_path)?;

        // Then
        let bytes = std::fs::read(output_path).context("Failed to read PNG file")?;
        // IEND chunk marks the end of a PNG file
        let iend_marker = b"IEND";
        assert!(
            bytes.windows(4).any(|w| w == iend_marker),
            "PNG file should contain IEND chunk marker"
        );
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_document_when_export_png_then_png_has_ihdr_chunk() -> Result<()> {
        // Given
        let doc = DiagramDocument::default();
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        let output_path = temp_file.path().to_str().context("Invalid path")?;

        // When
        export_png(&doc, output_path)?;

        // Then
        let bytes = std::fs::read(output_path).context("Failed to read PNG file")?;
        // IHDR chunk must be the first chunk in a PNG file (after signature)
        let ihdr_marker = b"IHDR";
        assert!(
            bytes.windows(4).any(|w| w == ihdr_marker),
            "PNG file should contain IHDR chunk marker"
        );
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_invalid_output_path_when_export_png_then_returns_error() {
        // Given
        let doc = DiagramDocument::default();
        let invalid_path = "/nonexistent/directory/output.png";

        // When/Then
        let result = export_png(&doc, invalid_path);
        assert!(
            result.is_err(),
            "Expected error when exporting to invalid path"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_large_coordinates_when_export_png_then_creates_valid_png() -> Result<()>
    {
        // Given
        let (node_id, node) = create_test_node("far_node", 10000.0, 10000.0);
        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);
        let doc = DiagramDocument {
            version: 2,
            revision: crate::models::document::Revision::INITIAL,
            document: DocumentData {
                nodes,
                edges: HashMap::new(),
            },
            editor_state: crate::models::document::EditorState::default(),
        };
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        let output_path = temp_file.path().to_str().context("Invalid path")?;

        // When
        export_png(&doc, output_path)?;

        // Then
        let bytes = std::fs::read(output_path).context("Failed to read PNG file")?;
        assert_eq!(&bytes[0..8], PNG_SIGNATURE, "Invalid PNG signature");
        Ok(())
    }
}
