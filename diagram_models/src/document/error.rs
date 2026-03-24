//! Document error types for diagram operations.
//!
//! Provides a domain-specific error taxonomy for document mutations.

use super::types::{EdgeId, NodeId};
use serde::{Deserialize, Serialize};

/// Errors that can occur during document operations
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum DocumentError {
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("edge already exists: {0}")]
    EdgeAlreadyExists(EdgeId),
    #[error("edge not found: {0}")]
    EdgeNotFound(EdgeId),
    #[error("invalid marquee bounds: negative width or height")]
    InvalidMarqueeBounds,
}

#[cfg(test)]
mod tests {
    use super::super::types::{EdgeId, NodeId};
    use super::DocumentError;

    #[test]
    fn document_error_node_not_found_display() {
        let node_id = NodeId::new("test-node".into());
        let err = DocumentError::NodeNotFound(node_id);
        assert_eq!(err.to_string(), "node not found: test-node");
    }

    #[test]
    fn document_error_edge_already_exists_display() {
        let edge_id = EdgeId::new("test-edge".into());
        let err = DocumentError::EdgeAlreadyExists(edge_id);
        assert_eq!(err.to_string(), "edge already exists: test-edge");
    }

    #[test]
    fn document_error_edge_not_found_display() {
        let edge_id = EdgeId::new("test-edge".into());
        let err = DocumentError::EdgeNotFound(edge_id);
        assert_eq!(err.to_string(), "edge not found: test-edge");
    }
}
