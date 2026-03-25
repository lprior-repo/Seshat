//! Shared test helpers for the `apply` module tests.

use diagram_models::document::types::AuthorId;
use diagram_models::document::types::Revision;
use diagram_models::document::DiagramDocument;
use diagram_models::document::{DocumentData, EditorState};
use std::path::PathBuf;

use crate::apply::types::ApplyProposal;

/// Creates a valid `ApplyProposal` for testing.
pub fn valid_proposal() -> ApplyProposal {
    ApplyProposal {
        base_revision: Revision::new(1),
        proposer: AuthorId::new("test-agent".to_string()),
        proposed_at: diagram_models::document::types::Timestamp::new(1_700_000_000),
        summary: "Test proposal".to_string(),
        changes: vec![
            diagram_models::proposed_changes::ProposedChange::MoveNode {
                node_id: diagram_models::document::types::NodeId::new("n1".to_string()),
                new_x: 100.0,
                new_y: 200.0,
            },
            diagram_models::proposed_changes::ProposedChange::AddNode {
                node: diagram_models::document::Node {
                    kind: diagram_models::document::NodeKind::Node,
                    icon: String::new(),
                    label: "new-node".to_string(),
                    x: diagram_models::document::types::OrderedFloat::new_unchecked(0.0),
                    y: diagram_models::document::types::OrderedFloat::new_unchecked(0.0),
                    width: diagram_models::document::types::OrderedFloat::new_unchecked(100.0),
                    height: diagram_models::document::types::OrderedFloat::new_unchecked(100.0),
                    font_size: None,
                    font_weight: None,
                    lock_state: diagram_models::document::LockState::Unlocked,
                    parent: None,
                    dag_rank: None,
                    tags: im::Vector::new(),
                    metadata: im::HashMap::new(),
                    z_index: 0,
                    style: None,
                    collapsed: None,
                },
            },
            diagram_models::proposed_changes::ProposedChange::UpdateNodeLabel {
                node_id: diagram_models::document::types::NodeId::new("n1".to_string()),
                new_label: "updated".to_string(),
            },
        ],
    }
}

/// Creates a valid `DiagramDocument` for testing.
pub fn valid_document(revision: u64) -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::new(revision),
        document: DocumentData {
            nodes: im::HashMap::new(),
            edges: im::HashMap::new(),
        },
        editor_state: EditorState::default(),
    }
}

/// Creates a valid proposal JSON string.
pub fn valid_proposal_json(revision: u64) -> String {
    r#"{
        "base_revision": REPL_REV,
        "proposer": "test-agent",
        "proposed_at": 1700000000,
        "summary": "Test proposal",
        "changes": [
            {"change_type": "move_node", "node_id": "n1", "new_x": 100.0, "new_y": 200.0},
            {"change_type": "add_node", "node": {"kind": "node", "icon": "", "label": "new-node", "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0, "lock_state": "unlocked", "z_index": 0}},
            {"change_type": "update_node_label", "node_id": "n1", "new_label": "updated"}
        ]
    }"#
    .replace("REPL_REV", &revision.to_string())
}

/// Creates a valid `DiagramDocument` JSON string.
pub fn valid_document_json(revision: u64) -> String {
    format!(
        r#"{{
            "version": 2,
            "revision": {revision},
            "document": {{"nodes": {{}}, "edges": {{}}}},
            "editor_state": {{"camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0}}
        }}"#
    )
}

/// A writer that always fails on write.
pub struct AlwaysFailsWriter;

impl std::io::Write for AlwaysFailsWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "injected write error",
        ))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "injected write error",
        ))
    }
}

#[allow(clippy::unwrap_used)]
pub fn apply_source_file_path() -> PathBuf {
    PathBuf::from("/tmp/proposal.json")
}
