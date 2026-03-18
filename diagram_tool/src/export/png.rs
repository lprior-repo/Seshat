#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::export::svg::generate_svg_string;
use anyhow::{Context, Result};
use diagram_models::document::DiagramDocument;
use resvg::usvg;
use std::path::Path;
use tiny_skia::{Pixmap, Transform};

/// Export document to PNG file.
///
/// # Errors
/// Returns an error if SVG generation or PNG encoding fails.
pub fn export_png(doc: &DiagramDocument, path: impl AsRef<Path>) -> Result<()> {
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
    use anyhow::{Context, Result};
    use diagram_models::document::{
        DocumentData, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::HashMap;
    use tempfile::NamedTempFile;

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

    fn create_doc(
        nodes: impl IntoIterator<Item = (NodeId, Node)>,
        edges: impl IntoIterator<Item = (EdgeId, Edge)>,
    ) -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: diagram_models::document::Revision::INITIAL,
            document: DocumentData {
                nodes: nodes.into_iter().collect(),
                edges: edges.into_iter().collect(),
            },
            editor_state: diagram_models::document::EditorState::default(),
        }
    }

    fn export_to_temp(doc: &DiagramDocument) -> Result<(NamedTempFile, Vec<u8>)> {
        let temp_file = NamedTempFile::new().context("Failed to create temp file")?;
        export_png(doc, temp_file.path().to_str().unwrap())?;
        let bytes = std::fs::read(temp_file.path()).context("Failed to read PNG file")?;
        Ok((temp_file, bytes))
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_empty_document_when_export_png_then_creates_valid_png_file() -> Result<()> {
        let (_, bytes) = export_to_temp(&DiagramDocument::default())?;
        assert!(bytes.len() > 8);
        assert_eq!(&bytes[0..8], PNG_SIGNATURE);
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_single_node_when_export_png_then_creates_valid_png() -> Result<()> {
        let doc = create_doc(vec![create_test_node("1", 50.0, 50.0)], vec![]);
        let (_, bytes) = export_to_temp(&doc)?;
        assert_eq!(&bytes[0..8], PNG_SIGNATURE);
        assert!(bytes.len() > 100);
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_multiple_nodes_when_export_png_then_creates_valid_png() -> Result<()> {
        let doc = create_doc(
            vec![
                create_test_node("1", 0.0, 0.0),
                create_test_node("2", 200.0, 0.0),
                create_test_node("3", 100.0, 150.0),
            ],
            vec![],
        );
        let (_, bytes) = export_to_temp(&doc)?;
        assert_eq!(&bytes[0..8], PNG_SIGNATURE);
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_edges_when_export_png_then_creates_valid_png() -> Result<()> {
        let (src_id, src_node) = create_test_node("source", 0.0, 50.0);
        let (tgt_id, tgt_node) = create_test_node("target", 200.0, 50.0);
        let edge = Edge {
            source: src_id.clone(),
            target: tgt_id.clone(),
            label: String::new(),
            style: EdgeStyle::Solid,
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
        };
        let doc = create_doc(
            vec![(src_id, src_node), (tgt_id, tgt_node)],
            vec![(EdgeId::new("edge1".to_string()), edge)],
        );
        let (_, bytes) = export_to_temp(&doc)?;
        assert_eq!(&bytes[0..8], PNG_SIGNATURE);
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_document_when_export_png_then_file_exists_on_disk() -> Result<()> {
        let (temp_file, _) = export_to_temp(&DiagramDocument::default())?;
        assert!(temp_file.path().exists());
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_document_when_export_png_then_png_has_iend_chunk() -> Result<()> {
        let (_, bytes) = export_to_temp(&DiagramDocument::default())?;
        assert!(bytes.windows(4).any(|w| w == b"IEND"));
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_document_when_export_png_then_png_has_ihdr_chunk() -> Result<()> {
        let (_, bytes) = export_to_temp(&DiagramDocument::default())?;
        assert!(bytes.windows(4).any(|w| w == b"IHDR"));
        Ok(())
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_invalid_output_path_when_export_png_then_returns_error() {
        assert!(export_png(
            &DiagramDocument::default(),
            "/nonexistent/directory/output.png"
        )
        .is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_with_large_coordinates_when_export_png_then_creates_valid_png() -> Result<()>
    {
        let doc = create_doc(vec![create_test_node("far", 10000.0, 10000.0)], vec![]);
        let (_, bytes) = export_to_temp(&doc)?;
        assert_eq!(&bytes[0..8], PNG_SIGNATURE);
        Ok(())
    }
}
