#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::dag::validate_dag;
use crate::models::document::{DiagramDocument, DocumentData, NodeKind};

/// Severity of a validation issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    #[allow(dead_code)]
    Warning,
}

/// A single validation issue discovered in a `DiagramDocument`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub code: &'static str,
    pub message: String,
    pub subject: Option<String>,
}

/// Pure function: validates a `DiagramDocument` and returns all issues found.
///
/// This function is deterministic and has no side effects.
#[must_use]
pub fn validate_document(doc: &DiagramDocument) -> Vec<ValidationIssue> {
    validate_document_data(&doc.document)
}

/// Pure function: validates only the structural document graph.
///
/// Editor/camera state is intentionally ignored.
#[must_use]
pub fn validate_document_data(document: &DocumentData) -> Vec<ValidationIssue> {
    let nodes = &document.nodes;
    let edges = &document.edges;

    let edge_issues = edges.iter().flat_map(|(id, edge)| {
        let src_issue = (!nodes.contains_key(&edge.source)).then(|| ValidationIssue {
            severity: ValidationSeverity::Error,
            code: "edge-dangling",
            message: format!("Edge {id} source '{}' does not exist", edge.source),
            subject: Some(id.to_string()),
        });
        let tgt_issue = (!nodes.contains_key(&edge.target)).then(|| ValidationIssue {
            severity: ValidationSeverity::Error,
            code: "edge-dangling",
            message: format!("Edge {id} target '{}' does not exist", edge.target),
            subject: Some(id.to_string()),
        });
        src_issue.into_iter().chain(tgt_issue)
    });

    let node_issues = nodes.iter().filter_map(|(id, node)| {
        node.parent.as_ref().and_then(|parent_id| {
            if !nodes.contains_key(parent_id) {
                Some(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    code: "invalid-parent",
                    message: format!("Node {id} references non-existent parent {parent_id}"),
                    subject: Some(id.to_string()),
                })
            } else if nodes
                .get(parent_id)
                .is_some_and(|p| p.kind != NodeKind::Subgraph)
            {
                Some(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    code: "invalid-parent",
                    message: format!("Node {id} parent {parent_id} is not a Subgraph"),
                    subject: Some(id.to_string()),
                })
            } else {
                None
            }
        })
    });

    let dag_issues = validate_dag(nodes, edges).err().map(|_| ValidationIssue {
        severity: ValidationSeverity::Error,
        code: "dag-cycle",
        message: String::from("Document contains a cycle — DAGs must be acyclic"),
        subject: None,
    });

    edge_issues.chain(node_issues).chain(dag_issues).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::{
        ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, NodeStyle,
        OrderedFloat,
    };
    use im::HashMap;

    fn make_node(id: &str) -> (NodeId, Node) {
        (
            NodeId::new(id.to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: id.to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(64.0),
                height: OrderedFloat(64.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            },
        )
    }

    fn make_edge(id: &str, src: &str, tgt: &str) -> (EdgeId, Edge) {
        (
            EdgeId::new(id.to_string()),
            Edge {
                source: NodeId::new(src.to_string()),
                target: NodeId::new(tgt.to_string()),
                label: String::new(),
                style: EdgeStyle::default(),
                arrow_type: ArrowType::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: Vec::new(),
                tags: Vec::new(),
                metadata: HashMap::new(),
                font_size: None,
            },
        )
    }

    #[test]
    fn given_edge_to_nonexistent_node_when_validated_then_edge_dangling_error() {
        let mut doc = DiagramDocument::default();
        let (nid, node) = make_node("A");
        doc.document.nodes = doc.document.nodes.update(nid, node);
        let (eid, edge) = make_edge("e1", "A", "MISSING");
        doc.document.edges = doc.document.edges.update(eid, edge);

        let issues = validate_document(&doc);
        assert!(issues.iter().any(|i| i.code == "edge-dangling"));
    }

    #[test]
    fn given_cycle_when_validated_then_dag_cycle_error() {
        let mut doc = DiagramDocument::default();
        let (aid, a) = make_node("A");
        let (bid, b) = make_node("B");
        doc.document.nodes = doc.document.nodes.update(aid, a).update(bid, b);
        let (e1id, e1) = make_edge("e1", "A", "B");
        let (e2id, e2) = make_edge("e2", "B", "A");
        doc.document.edges = doc.document.edges.update(e1id, e1).update(e2id, e2);

        let issues = validate_document(&doc);
        assert!(issues.iter().any(|i| i.code == "dag-cycle"));
    }

    #[test]
    fn given_node_with_non_subgraph_parent_when_validated_then_invalid_parent_error() {
        let mut doc = DiagramDocument::default();
        let (aid, a) = make_node("A"); // kind: Node (not Subgraph)
        let (bid, mut b) = make_node("B");
        b.parent = Some(NodeId::new("A".to_string()));
        doc.document.nodes = doc.document.nodes.update(aid, a).update(bid, b);

        let issues = validate_document(&doc);
        assert!(issues.iter().any(|i| i.code == "invalid-parent"));
    }

    #[test]
    fn given_node_with_existing_subgraph_parent_when_validated_then_no_invalid_parent_issue() {
        let mut doc = DiagramDocument::default();
        let (parent_id, mut parent) = make_node("P");
        parent.kind = NodeKind::Subgraph;
        let (child_id, mut child) = make_node("C");
        child.parent = Some(parent_id.clone());
        doc.document.nodes = doc
            .document
            .nodes
            .update(parent_id, parent)
            .update(child_id, child);

        let issues = validate_document(&doc);
        assert!(!issues.iter().any(|i| i.code == "invalid-parent"));
    }

    #[test]
    fn given_valid_document_when_validated_then_no_issues() {
        let doc = DiagramDocument::default();
        let issues = validate_document(&doc);
        assert!(issues.is_empty());
    }

    #[test]
    fn given_nan_node_geometry_when_validated_then_no_panic() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("nan-node");
        node.x = OrderedFloat(f64::NAN);
        node.y = OrderedFloat(f64::NAN);
        node.width = OrderedFloat(f64::NAN);
        node.height = OrderedFloat(f64::NAN);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        for issue in &issues {
            assert!(
                issue.code != "internal-error",
                "Validation should not create internal error codes for NaN geometry"
            );
        }
    }

    #[test]
    fn given_inf_node_geometry_when_validated_then_no_panic() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("inf-node");
        node.x = OrderedFloat(f64::INFINITY);
        node.y = OrderedFloat(f64::NEG_INFINITY);
        node.width = OrderedFloat(f64::INFINITY);
        node.height = OrderedFloat(f64::INFINITY);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        assert!(issues.iter().all(|i| i.code != "internal-error"));
    }

    #[test]
    fn given_negative_node_dimensions_when_validated_then_no_panic() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("neg-node");
        node.width = OrderedFloat(-10.0);
        node.height = OrderedFloat(-5.0);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        assert!(issues.iter().all(|i| i.code != "internal-error"));
    }

    #[test]
    fn given_nan_editor_zoom_when_validated_then_no_panic() {
        let mut doc = DiagramDocument::default();
        doc.editor_state.zoom = OrderedFloat(f64::NAN);
        let issues = validate_document(&doc);
        assert!(issues.iter().all(|i| i.code != "internal-error"));
    }

    #[test]
    fn given_invalid_editor_zoom_range_when_validated_then_no_panic() {
        let mut doc = DiagramDocument::default();
        doc.editor_state.zoom = OrderedFloat(10.0);
        let issues = validate_document(&doc);
        assert!(issues.iter().all(|i| i.code != "internal-error"));

        doc.editor_state.zoom = OrderedFloat(-1.0);
        let issues2 = validate_document(&doc);
        assert!(issues2.iter().all(|i| i.code != "internal-error"));
    }

    #[test]
    fn given_nan_camera_position_when_validated_then_no_panic() {
        let mut doc = DiagramDocument::default();
        doc.editor_state.camera_x = OrderedFloat(f64::NAN);
        doc.editor_state.camera_y = OrderedFloat(f64::NAN);
        let issues = validate_document(&doc);
        assert!(issues.iter().all(|i| i.code != "internal-error"));
    }

    #[test]
    fn given_valid_node_minimum_size_when_validated_then_accepts() {
        let mut doc = DiagramDocument::default();
        let (nid, node) = make_node("small-valid");
        let small_node = Node {
            width: OrderedFloat(24.0),
            height: OrderedFloat(24.0),
            ..node
        };
        doc.document.nodes = doc.document.nodes.update(nid, small_node);
        let issues = validate_document(&doc);
        assert!(issues.iter().all(|i| i.code != "internal-error"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_finite_f64()(x in -1e6_f64..1e6_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_positive_f64()(x in 1.0_f64..1000.0_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_node_id()(s in "[a-z]{1,3}") -> NodeId { NodeId::new(s) }
    }

    prop_compose! {
        fn arb_node()(
            id in arb_node_id(),
            x in arb_finite_f64(),
            y in arb_finite_f64(),
            width in arb_positive_f64(),
            height in arb_positive_f64(),
        ) -> (NodeId, Node) {
            (
                id.clone(),
                Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: String::new(),
                    x: OrderedFloat(x),
                    y: OrderedFloat(y),
                    width: OrderedFloat(width),
                    height: OrderedFloat(height),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: Vec::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: Some(NodeStyle::default()),
                    collapsed: None,
                },
            )
        }
    }

    fn make_doc_with_nodes(nodes: Vec<(NodeId, Node)>) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        for (id, node) in nodes {
            doc.document.nodes = doc.document.nodes.update(id, node);
        }
        doc
    }

    proptest! {
        fn prop_validate_never_panics_on_finite_geometry(nodes in proptest::collection::vec(arb_node(), 0..10)) {
            let doc = make_doc_with_nodes(nodes);
            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != "internal-error"));
        }

        fn prop_validate_camera_state_ignored(nodes in proptest::collection::vec(arb_node(), 0..5)) {
            let mut doc = make_doc_with_nodes(nodes);
            doc.editor_state.camera_x = OrderedFloat(f64::NAN);
            doc.editor_state.camera_y = OrderedFloat(f64::INFINITY);
            doc.editor_state.zoom = OrderedFloat(-100.0);

            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != "internal-error"));
        }

        fn prop_validate_negative_dimensions_no_panic(
            id in arb_node_id(),
            width in -1000.0_f64..0.0_f64,
            height in -1000.0_f64..0.0_f64,
        ) {
            let mut doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(width),
                height: OrderedFloat(height),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            };
            doc.document.nodes = doc.document.nodes.update(id, node);

            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != "internal-error"));
        }

        fn prop_validate_tiny_dimensions_no_panic(
            id in arb_node_id(),
            dim in 0.0_f64..1.0_f64,
        ) {
            let mut doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(dim),
                height: OrderedFloat(dim),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            };
            doc.document.nodes = doc.document.nodes.update(id, node);

            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != "internal-error"));
        }

        fn prop_validate_extreme_coords_no_panic(
            id in arb_node_id(),
            x in -1e15_f64..1e15_f64,
            y in -1e15_f64..1e15_f64,
        ) {
            let mut doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                width: OrderedFloat(64.0),
                height: OrderedFloat(64.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            };
            doc.document.nodes = doc.document.nodes.update(id, node);

            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != "internal-error"));
        }
    }

    #[test]
    fn prop_validate_empty_doc_has_no_issues() {
        let doc = DiagramDocument::default();
        let issues = validate_document(&doc);
        assert!(issues.is_empty());
    }
}
