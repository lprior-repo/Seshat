use crate::domain::LayoutCommand;
use crate::error::LayoutError;
use diagram_models::physical_io;
use diagram_tool::layout::dag::{dag_layout, DagLayoutSettings};

/// Executes the layout command
///
/// # Errors
/// Returns `LayoutError` if loading or saving the document fails.
pub fn execute_layout(cmd: &LayoutCommand) -> Result<(), LayoutError> {
    let doc = physical_io::load_document(&cmd.input)
        .map_err(|e| LayoutError::LoadFailed(e.to_string()))?;

    let laid_out = dag_layout(&doc, &DagLayoutSettings::default());

    physical_io::save_document(&cmd.output, &laid_out)
        .map_err(|e| LayoutError::SaveFailed(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod tests {
    use super::*;
    use diagram_models::document::{
        ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, Node,
        NodeId, NodeKind, NodeStyle, OrderedFloat, Revision,
    };
    use im::HashMap;
    use std::fs;
    use tempfile::tempdir;

    fn make_node(id: &str, x: f64, y: f64) -> (NodeId, Node) {
        (
            NodeId::new(id.to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: id.to_string(),
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                width: OrderedFloat(220.0),
                height: OrderedFloat(68.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            },
        )
    }

    fn make_edge(src: &NodeId, tgt: &NodeId) -> (EdgeId, Edge) {
        (
            EdgeId::new(format!("edge-{}-{}", src.as_str(), tgt.as_str())),
            Edge {
                source: src.clone(),
                target: tgt.clone(),
                label: String::new(),
                style: diagram_models::document::EdgeStyle::Solid,
                arrow_type: ArrowType::Default,
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
        )
    }

    fn create_test_doc() -> DiagramDocument {
        let (a, node_a) = make_node("A", 0.0, 0.0);
        let (b, node_b) = make_node("B", 0.0, 0.0);
        let (c, node_c) = make_node("C", 0.0, 0.0);

        let (e1, edge_ab) = make_edge(&a, &b);
        let (e2, edge_bc) = make_edge(&b, &c);

        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: vec![(a, node_a), (b, node_b), (c, node_c)]
                    .into_iter()
                    .collect(),
                edges: vec![(e1, edge_ab), (e2, edge_bc)].into_iter().collect(),
            },
            editor_state: EditorState::default(),
        }
    }

    #[test]
    fn layout_command_processes_sequential_dag() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.json");
        let output_path = dir.path().join("output.json");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = LayoutCommand {
            input: input_path,
            output: output_path.clone(),
        };
        let result = execute_layout(&cmd);
        assert!(result.is_ok(), "layout command should succeed");

        assert!(output_path.exists(), "output file should be created");

        let output_doc = physical_io::load_document(&output_path).unwrap();
        assert_eq!(output_doc.document.nodes.len(), 3);

        let a = output_doc
            .document
            .nodes
            .get(&NodeId::new("A".to_string()))
            .unwrap();
        let b = output_doc
            .document
            .nodes
            .get(&NodeId::new("B".to_string()))
            .unwrap();
        let c = output_doc
            .document
            .nodes
            .get(&NodeId::new("C".to_string()))
            .unwrap();

        assert!(a.x.0 < b.x.0, "A.x should be < B.x after layout");
        assert!(b.x.0 < c.x.0, "B.x should be < C.x after layout");
    }

    #[test]
    fn layout_command_fails_on_missing_input() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("nonexistent.json");
        let output_path = dir.path().join("output.json");

        let cmd = LayoutCommand {
            input: input_path,
            output: output_path,
        };
        let result = execute_layout(&cmd);
        assert!(result.is_err(), "should fail on missing input file");
    }

    #[test]
    fn layout_command_fails_on_invalid_json() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("invalid.json");
        let output_path = dir.path().join("output.json");

        fs::write(&input_path, "not valid json").unwrap();

        let cmd = LayoutCommand {
            input: input_path,
            output: output_path,
        };
        let result = execute_layout(&cmd);
        assert!(result.is_err(), "should fail on invalid JSON");
    }

    #[test]
    fn layout_command_handles_empty_document() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("empty.json");
        let output_path = dir.path().join("output.json");

        let empty_doc = DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        };
        physical_io::save_document(&input_path, &empty_doc).unwrap();

        let cmd = LayoutCommand {
            input: input_path,
            output: output_path.clone(),
        };
        let result = execute_layout(&cmd);
        assert!(result.is_ok(), "should handle empty document");
        assert!(output_path.exists());
    }

    #[test]
    fn layout_command_increments_revision() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.json");
        let output_path = dir.path().join("output.json");

        let doc = create_test_doc();
        let original_revision = doc.revision;
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = LayoutCommand {
            input: input_path,
            output: output_path.clone(),
        };
        execute_layout(&cmd).unwrap();

        let output_doc = physical_io::load_document(&output_path).unwrap();
        assert!(
            output_doc.revision > original_revision,
            "revision should increment after layout"
        );
    }
}
