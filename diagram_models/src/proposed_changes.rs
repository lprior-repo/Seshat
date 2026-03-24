//! `ProposedChanges` — the container for an AI agent's proposed modifications.
//!
//! This module defines the minimal shape needed by the revision-mismatch gate
//! in [`crate::apply::check_revision_mismatch`].  A sibling bead will flesh out
//! the full `ProposedChange` enum and additional fields.

use crate::document::types::{AuthorId, EdgeId, NodeId, Revision, Timestamp};
use crate::document::{DocumentError, Node};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedChanges {
    pub base_revision: Revision,
    pub proposer: AuthorId,
    pub proposed_at: Timestamp,
    pub summary: String,
    pub changes: Vec<ProposedChange>,
}

impl ProposedChanges {
    #[must_use]
    pub const fn new(
        base_revision: Revision,
        proposer: AuthorId,
        proposed_at: Timestamp,
        summary: String,
        changes: Vec<ProposedChange>,
    ) -> Self {
        Self {
            base_revision,
            proposer,
            proposed_at,
            summary,
            changes,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.changes.len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GhostDiffBadge {
    Add,
    Modify,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum ProposedChange {
    DeleteNode {
        node_id: NodeId,
        was_node_id: NodeId,
        was: Node,
    },
    /// Placeholder variant for testing the "unsupported change variant" dispatch.
    /// Will be removed when real variants (MoveNode, AddEdge, etc.) are added.
    TestUnsupportedVariant,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
            ProposedChange::TestUnsupportedVariant => {}
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

    #[test]
    fn proposed_changes_new_creates_instance_with_all_fields() {
        let changes = ProposedChanges::new(
            Revision::new(1),
            AuthorId::new("agent-1".into()),
            Timestamp::new(1234567890),
            "Test proposal".to_string(),
            vec![],
        );
        assert_eq!(changes.base_revision, Revision::new(1));
        assert_eq!(changes.proposer, AuthorId::new("agent-1".into()));
        assert_eq!(changes.proposed_at, Timestamp::new(1234567890));
        assert_eq!(changes.summary, "Test proposal");
        assert!(changes.changes.is_empty());
    }

    #[test]
    fn proposed_changes_is_empty_returns_true_for_empty_changes() {
        let changes = ProposedChanges::new(
            Revision::new(0),
            AuthorId::new("agent".into()),
            Timestamp::new(0),
            String::new(),
            vec![],
        );
        assert!(changes.is_empty());
    }

    #[test]
    fn proposed_changes_is_empty_returns_false_for_non_empty_changes() {
        let node = test_node("n1");
        let change = ProposedChange::DeleteNode {
            node_id: NodeId::new("n1".into()),
            was_node_id: NodeId::new("n1".into()),
            was: node,
        };
        let changes = ProposedChanges::new(
            Revision::new(0),
            AuthorId::new("agent".into()),
            Timestamp::new(0),
            String::new(),
            vec![change],
        );
        assert!(!changes.is_empty());
    }

    #[test]
    fn proposed_changes_len_returns_change_count() {
        let node = test_node("n1");
        let change1 = ProposedChange::DeleteNode {
            node_id: NodeId::new("n1".into()),
            was_node_id: NodeId::new("n1".into()),
            was: node.clone(),
        };
        let change2 = ProposedChange::DeleteNode {
            node_id: NodeId::new("n2".into()),
            was_node_id: NodeId::new("n2".into()),
            was: node,
        };
        let changes = ProposedChanges::new(
            Revision::new(0),
            AuthorId::new("agent".into()),
            Timestamp::new(0),
            String::new(),
            vec![change1, change2],
        );
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn proposed_changes_serializes_and_deserializes_correctly() {
        let node = test_node("n1");
        let original = ProposedChanges::new(
            Revision::new(5),
            AuthorId::new("claude-3".into()),
            Timestamp::new(1700000000),
            "Move node to new position".to_string(),
            vec![ProposedChange::DeleteNode {
                node_id: NodeId::new("n1".into()),
                was_node_id: NodeId::new("n1".into()),
                was: node,
            }],
        );
        let json = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: ProposedChanges =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn proposed_changes_empty_serialization_roundtrip() {
        let original = ProposedChanges::new(
            Revision::new(0),
            AuthorId::new("agent".into()),
            Timestamp::new(0),
            String::new(),
            vec![],
        );
        let json = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: ProposedChanges =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn ghost_diff_badge_serialization_roundtrip() {
        for badge in [
            GhostDiffBadge::Add,
            GhostDiffBadge::Modify,
            GhostDiffBadge::Delete,
        ] {
            let json = serde_json::to_string(&badge).expect("serialization should succeed");
            let deserialized: GhostDiffBadge =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(badge, deserialized);
        }
    }

    #[test]
    fn apply_error_serialization_roundtrip() {
        let err = ApplyError::NodeNotFound(NodeId::new("missing".into()));
        let json = serde_json::to_string(&err).expect("serialization should succeed");
        let deserialized: ApplyError =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(err, deserialized);
    }

    #[test]
    fn delete_node_result_serialization_roundtrip() {
        let result = DeleteNodeResult {
            deleted_node_id: NodeId::new("n1".into()),
            cascade_deleted_edge_ids: vec![EdgeId::new("e1".into()), EdgeId::new("e2".into())],
        };
        let json = serde_json::to_string(&result).expect("serialization should succeed");
        let deserialized: DeleteNodeResult =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(result, deserialized);
    }
}
