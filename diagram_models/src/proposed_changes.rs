//! `ProposedChanges` — the container for an AI agent's proposed modifications.
//!
//! This module defines the minimal shape needed by the revision-mismatch gate
//! in [`crate::apply::check_revision_mismatch`].  A sibling bead will flesh out
//! the full `ProposedChange` enum and additional fields.

use crate::document::types::{AuthorId, EdgeId, NodeId, Revision, Timestamp};
use crate::document::{DocumentError, Edge, Node, SerializedPoint};
use serde_json::Value;

/// A complete proposal submitted by an AI agent.
///
/// Only `base_revision` is consumed by the revision-mismatch gate; the
/// remaining fields are placeholders for downstream beads.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposedChanges {
    /// The document revision this proposal was built against.
    /// Must match `DiagramDocument::revision` at apply time.
    pub base_revision: Revision,
    /// Identifier of the proposing AI agent.
    pub proposer: AuthorId,
    /// Wall-clock time when the proposal was generated.
    pub proposed_at: Timestamp,
    /// Human-readable summary for UI display.
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GhostDiffBadge {
    Add,
    Modify,
    Delete,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_f64_allow_nan<S: serde::Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
    if v.is_nan() {
        s.serialize_str("NaN")
    } else {
        s.serialize_f64(*v)
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "change_type", rename_all = "snake_case", deny_unknown_fields)]
#[allow(clippy::large_enum_variant)]
pub enum ProposedChange {
    MoveNode {
        node_id: NodeId,
        #[serde(serialize_with = "serialize_f64_allow_nan")]
        new_x: f64,
        #[serde(serialize_with = "serialize_f64_allow_nan")]
        new_y: f64,
    },
    AddNode {
        node: Node,
    },
    UpdateNodeLabel {
        node_id: NodeId,
        new_label: String,
    },
    UpdateNodeProperty {
        node_id: NodeId,
        property: String,
        value: Value,
    },
    AddEdge {
        edge_id: EdgeId,
        edge: Edge,
    },
    DeleteEdge {
        edge_id: EdgeId,
    },
    UpdateEdgeRouting {
        edge_id: EdgeId,
        bend_points: im::Vector<SerializedPoint>,
    },
    UpdateEdgeLabel {
        edge_id: EdgeId,
        new_label: String,
    },
    DeleteNode {
        node_id: NodeId,
        was_node_id: NodeId,
        was: Node,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError {
    #[error("cannot delete node: {0} not found in document")]
    NodeNotFound(NodeId),

    #[error("delete node snapshot mismatch: declared {declared}, snapshot has {snapshot}")]
    SnapshotIdMismatch { declared: NodeId, snapshot: NodeId },

    #[error("document mutation failed during delete node: {0}")]
    DocumentError(DocumentError),

    #[error("unsupported change variant")]
    UnsupportedChangeVariant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteNodeResult {
    pub deleted_node_id: NodeId,
    pub cascade_deleted_edge_ids: Vec<EdgeId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::types::OrderedFloat;
    use crate::document::LockState;
    use crate::document::NodeKind;

    fn test_node(id: &str) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat::new_unchecked(0.0),
            y: OrderedFloat::new_unchecked(0.0),
            width: OrderedFloat::new_unchecked(100.0),
            height: OrderedFloat::new_unchecked(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn proposed_change_delete_node_stores_node_id_and_snapshot() {
        let node = test_node("n1");
        let change = ProposedChange::DeleteNode {
            node_id: NodeId::new("n1".into()),
            was_node_id: NodeId::new("n1".into()),
            was: node.clone(),
        };
        match change {
            ProposedChange::DeleteNode {
                node_id,
                was_node_id,
                was,
            } => {
                assert_eq!(node_id, NodeId::new("n1".into()));
                assert_eq!(was_node_id, NodeId::new("n1".into()));
                assert_eq!(was, node);
            }
            _ => {}
        }
    }

    #[test]
    fn apply_error_snapshot_id_mismatch_displays_both_ids() {
        let err = ApplyError::SnapshotIdMismatch {
            declared: NodeId::new("n1".into()),
            snapshot: NodeId::new("n2".into()),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("n1"),
            "message must contain declared id: {msg}"
        );
        assert!(
            msg.contains("n2"),
            "message must contain snapshot id: {msg}"
        );
    }

    #[test]
    fn apply_error_node_not_found_displays_node_id() {
        let err = ApplyError::NodeNotFound(NodeId::new("missing".into()));
        let msg = err.to_string();
        assert!(
            msg.contains("missing"),
            "message must contain node id: {msg}"
        );
        assert!(
            msg.contains("not found"),
            "message must contain 'not found': {msg}"
        );
    }

    #[test]
    fn apply_error_document_error_displays_wrapped_message() {
        let inner = DocumentError::NodeNotFound(NodeId::new("x".into()));
        let err = ApplyError::DocumentError(inner);
        let msg = err.to_string();
        assert!(
            msg.contains("document mutation failed"),
            "message must contain wrapping context: {msg}"
        );
    }

    #[test]
    fn delete_node_result_stores_deleted_node_and_cascade_edges() {
        let result = DeleteNodeResult {
            deleted_node_id: NodeId::new("n1".into()),
            cascade_deleted_edge_ids: vec![EdgeId::new("e1".into())],
        };
        assert_eq!(result.deleted_node_id, NodeId::new("n1".into()));
        assert_eq!(
            result.cascade_deleted_edge_ids,
            vec![EdgeId::new("e1".into())]
        );
    }
}
