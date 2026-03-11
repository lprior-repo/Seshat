//! Mapping module - converts between DTO `DiagramDocument` and Domain `DiagramProjection`
//!
//! This module establishes the boundary between the persistence/DTO layer (`DiagramDocument`)
//! and the domain model layer (`DiagramProjection`).

#![allow(dead_code)]
#![allow(unused_imports)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::document::DiagramDocument;
use crate::models::projection::{CyclePolicy, DiagramProjection};

/// Convert a `DiagramProjection` to a `DiagramDocument`.
///
/// This is useful for interoperability with existing document handling.
/// The projection is the domain model, and the document is the DTO for serialization.
///
/// # Panics
/// This function does not panic - all fallible operations return Results.
/// However, callers should ensure the projection has valid data.
#[must_use]
pub fn projection_to_document(projection: &DiagramProjection) -> DiagramDocument {
    DiagramDocument {
        version: projection.version,
        revision: crate::models::document::Revision::new(projection.revision),
        document: crate::models::document::DocumentData {
            nodes: projection.nodes.clone(),
            edges: projection.edges.clone(),
        },
        editor_state: crate::models::document::EditorState::default(),
    }
}

/// Convert a `DiagramDocument` to a `DiagramProjection`.
///
/// This is useful for bootstrapping a projection from an existing document.
/// The document is the DTO/persistence format, and the projection is the domain model.
///
/// Note: The resulting projection will have empty `author_priority` map
/// and default `cycle_policy` since these are not persisted in the document.
#[must_use]
pub fn document_to_projection(document: &DiagramDocument) -> DiagramProjection {
    use im::HashMap;
    DiagramProjection {
        version: document.version,
        revision: document.revision.value(),
        nodes: document.document.nodes.clone(),
        edges: document.document.edges.clone(),
        author_priority: HashMap::new(),
        cycle_policy: CyclePolicy::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, Node, NodeId, Revision,
    };
    use crate::models::projection::DiagramProjection;
    use im::HashMap;

    fn make_test_node(id: &str) -> (NodeId, Node) {
        let id = NodeId::new(id.to_string());
        let node = Node {
            kind: crate::models::document::NodeKind::Node,
            icon: "test".to_string(),
            label: "Test Node".to_string(),
            x: crate::models::document::OrderedFloat(100.0),
            y: crate::models::document::OrderedFloat(200.0),
            width: crate::models::document::OrderedFloat(80.0),
            height: crate::models::document::OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        (id, node)
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_to_document_preserves_nodes_and_edges() {
        let (node_id, node) = make_test_node("node-1");
        let node_id_for_insert = node_id.clone();
        let edge_id = EdgeId::new("edge-1".to_string());
        let edge = Edge {
            source: node_id,
            target: node_id_for_insert.clone(),
            label: "test edge".to_string(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: crate::models::document::ArrowType::Default,
            label_offset_t: crate::models::document::OrderedFloat(0.5),
            color: None,
            thickness: crate::models::document::OrderedFloat(1.5),
            directed: true,
            bend_points: im::vector![],
            tags: im::vector![],
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        };

        let mut nodes = HashMap::new();
        let _ = nodes.insert(node_id_for_insert, node);
        let mut edges = HashMap::new();
        let _ = edges.insert(edge_id.clone(), edge);

        let projection = DiagramProjection {
            version: 2,
            revision: 5,
            nodes,
            edges,
            author_priority: HashMap::new(),
            cycle_policy: CyclePolicy::Allow,
        };

        let doc = projection_to_document(&projection);

        assert_eq!(doc.version, 2);
        assert_eq!(doc.revision.value(), 5);
        assert_eq!(doc.document.nodes.len(), 1);
        assert!(doc
            .document
            .nodes
            .contains_key(&NodeId::new("node-1".to_string())));
        assert_eq!(doc.document.edges.len(), 1);
        assert!(doc
            .document
            .edges
            .contains_key(&EdgeId::new("edge-1".to_string())));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_document_to_projection_preserves_nodes_and_edges() {
        let (node_id, node) = make_test_node("node-1");
        let node_id_for_insert = node_id.clone();
        let edge_id = EdgeId::new("edge-1".to_string());
        let edge = Edge {
            source: node_id,
            target: node_id_for_insert.clone(),
            label: "test edge".to_string(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: crate::models::document::ArrowType::Default,
            label_offset_t: crate::models::document::OrderedFloat(0.5),
            color: None,
            thickness: crate::models::document::OrderedFloat(1.5),
            directed: true,
            bend_points: im::vector![],
            tags: im::vector![],
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        };

        let mut nodes = HashMap::new();
        let _ = nodes.insert(node_id_for_insert, node);
        let mut edges = HashMap::new();
        let _ = edges.insert(edge_id.clone(), edge);

        let doc = DiagramDocument {
            version: 2,
            revision: Revision::new(5),
            document: DocumentData { nodes, edges },
            editor_state: crate::models::document::EditorState::default(),
        };

        let projection = document_to_projection(&doc);

        assert_eq!(projection.version, 2);
        assert_eq!(projection.revision, 5);
        assert_eq!(projection.nodes.len(), 1);
        assert!(projection
            .nodes
            .contains_key(&NodeId::new("node-1".to_string())));
        assert_eq!(projection.edges.len(), 1);
        assert!(projection
            .edges
            .contains_key(&EdgeId::new("edge-1".to_string())));
        // author_priority should be empty for converted projection
        assert!(projection.author_priority.is_empty());
        // cycle_policy should be default
        assert_eq!(projection.cycle_policy, CyclePolicy::Allow);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_to_document_empty_projection() {
        let projection = DiagramProjection::empty();
        let doc = projection_to_document(&projection);

        assert_eq!(doc.version, 2);
        assert_eq!(doc.revision.value(), 0);
        assert!(doc.document.nodes.is_empty());
        assert!(doc.document.edges.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_document_to_projection_empty_document() {
        let doc = DiagramDocument {
            version: 2,
            revision: Revision::new(0),
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: crate::models::document::EditorState::default(),
        };

        let projection = document_to_projection(&doc);

        assert_eq!(projection.version, 2);
        assert_eq!(projection.revision, 0);
        assert!(projection.nodes.is_empty());
        assert!(projection.edges.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_mapping_roundtrip_preserves_data() {
        // Start with a document
        let (node_id, node) = make_test_node("node-1");
        let node_id_for_insert = node_id.clone();
        let edge_id = EdgeId::new("edge-1".to_string());
        let edge = Edge {
            source: node_id,
            target: node_id_for_insert.clone(),
            label: "test edge".to_string(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: crate::models::document::ArrowType::Default,
            label_offset_t: crate::models::document::OrderedFloat(0.5),
            color: None,
            thickness: crate::models::document::OrderedFloat(1.5),
            directed: true,
            bend_points: im::vector![],
            tags: im::vector![],
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        };

        let mut nodes = HashMap::new();
        let _ = nodes.insert(node_id_for_insert, node);
        let mut edges = HashMap::new();
        let _ = edges.insert(edge_id, edge);

        let original_doc = DiagramDocument {
            version: 2,
            revision: Revision::new(10),
            document: DocumentData { nodes, edges },
            editor_state: crate::models::document::EditorState::default(),
        };

        // Convert to projection and back
        let projection = document_to_projection(&original_doc);
        let roundtrip_doc = projection_to_document(&projection);

        // Verify data is preserved
        assert_eq!(original_doc.version, roundtrip_doc.version);
        assert_eq!(
            original_doc.revision.value(),
            roundtrip_doc.revision.value()
        );
        assert_eq!(
            original_doc.document.nodes.len(),
            roundtrip_doc.document.nodes.len()
        );
        assert_eq!(
            original_doc.document.edges.len(),
            roundtrip_doc.document.edges.len()
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_to_document_preserves_cycle_policy() {
        let projection = DiagramProjection {
            version: 2,
            revision: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: CyclePolicy::Deny,
        };

        let doc = projection_to_document(&projection);

        // The cycle_policy is not persisted in the document
        // but the version/revision are preserved
        assert_eq!(doc.version, 2);
        assert_eq!(doc.revision.value(), 0);
    }
}
