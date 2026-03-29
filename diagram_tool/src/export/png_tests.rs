#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
#[cfg(test)]
mod tests {
    use crate::export::png::export_png;
    use diagram_models::document::{
        DiagramDocument, DocumentData, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::HashMap;
    use tempfile::NamedTempFile;

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

    #[test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn test_export_png_creates_valid_file() {
        let mut nodes = im::HashMap::new();
        let (id, node) = create_test_node("1", 50.0, 50.0);
        nodes.insert(id, node);

        let doc = DiagramDocument {
            version: 2,
            revision: diagram_models::document::Revision::INITIAL,
            document: DocumentData {
                nodes,
                edges: im::HashMap::new(),
            },
            editor_state: Default::default(),
        };

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();

        let result = export_png(&doc, path);
        assert!(result.is_ok(), "PNG export should succeed");

        let bytes = std::fs::read(path).expect("Failed to read created PNG");
        assert!(bytes.len() > 8, "PNG file is too small");
        assert_eq!(
            &bytes[0..8],
            &[137, 80, 78, 71, 13, 10, 26, 10],
            "Invalid PNG signature"
        );
    }
}
