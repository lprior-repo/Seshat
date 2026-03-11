#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

use crate::models::dag::{validate_dag, CycleError};
use crate::models::document::{DiagramDocument, DocumentData, NodeKind};

/// Severity of a validation issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationSeverity {
    Warning,
    Error,
}

impl PartialOrd for ValidationSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValidationSeverity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Error, Self::Error) | (Self::Warning, Self::Warning) => {
                std::cmp::Ordering::Equal
            }
            (Self::Error, Self::Warning) => std::cmp::Ordering::Greater,
            (Self::Warning, Self::Error) => std::cmp::Ordering::Less,
        }
    }
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

    let node_issues = nodes.iter().flat_map(|(id, node)| {
        let parent_issue = node.parent.as_ref().and_then(|parent_id| {
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
        });

        let nan_issue = if !node.x.0.is_finite() || !node.y.0.is_finite() {
            Some(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "invalid-numeric",
                message: format!(
                    "Node {id} has non-finite coordinates: x={}, y={}",
                    node.x.0, node.y.0
                ),
                subject: Some(id.to_string()),
            })
        } else {
            None
        };

        let dimension_issue = if node.width.0 < 0.0
            || node.height.0 < 0.0
            || !node.width.0.is_finite()
            || !node.height.0.is_finite()
        {
            Some(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "invalid-numeric",
                message: format!(
                    "Node {id} has invalid dimensions: width={}, height={}",
                    node.width.0, node.height.0
                ),
                subject: Some(id.to_string()),
            })
        } else {
            None
        };

        parent_issue
            .into_iter()
            .chain(nan_issue)
            .chain(dimension_issue)
    });

    let dag_issues = validate_dag(nodes, edges).err().map(|e| {
        let (code, message) = match e {
            CycleError::CycleDetected(_) => (
                "dag-cycle",
                "Document contains a cycle — DAGs must be acyclic",
            ),
            CycleError::DisconnectedGraph(n) => (
                "dag-disconnected",
                &*format!("Graph has {n} disconnected components — all nodes must be connected",),
            ),
        };
        ValidationIssue {
            severity: ValidationSeverity::Error,
            code,
            message: message.to_string(),
            subject: None,
        }
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
                tags: im::Vector::new(),
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
                bend_points: im::Vector::new(),
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
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
    fn given_nan_node_geometry_when_validated_then_returns_error() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("nan-node");
        node.x = OrderedFloat::new_unchecked(f64::NAN);
        node.y = OrderedFloat::new_unchecked(f64::NAN);
        node.width = OrderedFloat::new_unchecked(f64::NAN);
        node.height = OrderedFloat::new_unchecked(f64::NAN);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        assert!(
            issues.iter().any(|i| i.code == "invalid-numeric"),
            "Validation should report invalid-numeric for NaN geometry"
        );
    }

    #[test]
    fn given_inf_node_geometry_when_validated_then_returns_error() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("inf-node");
        node.x = OrderedFloat::new_unchecked(f64::INFINITY);
        node.y = OrderedFloat::new_unchecked(f64::NEG_INFINITY);
        node.width = OrderedFloat::new_unchecked(f64::INFINITY);
        node.height = OrderedFloat::new_unchecked(f64::INFINITY);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        assert!(
            issues.iter().any(|i| i.code == "invalid-numeric"),
            "Validation should report invalid-numeric for Inf geometry"
        );
    }

    #[test]
    fn given_negative_node_dimensions_when_validated_then_returns_error() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("neg-node");
        node.width = OrderedFloat::new_unchecked(-10.0);
        node.height = OrderedFloat::new_unchecked(-5.0);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        assert!(
            issues.iter().any(|i| i.code == "invalid-numeric"),
            "Validation should report invalid-numeric for negative dimensions"
        );
    }

    #[test]
    fn given_valid_node_minimum_size_when_validated_then_accepts() {
        let mut doc = DiagramDocument::default();
        let (nid, node) = make_node("small-valid");
        let small_node = Node {
            width: OrderedFloat::new_unchecked(24.0),
            height: OrderedFloat::new_unchecked(24.0),
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
                    x: OrderedFloat::new_unchecked(x),
                    y: OrderedFloat::new_unchecked(y),
                    width: OrderedFloat::new_unchecked(width),
                    height: OrderedFloat::new_unchecked(height),
                    font_size: None,
                    font_weight: None,
                    locked: false,
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
    }

    proptest! {
        fn prop_validate_never_panics_on_finite_geometry(nodes in proptest::collection::vec(arb_node(), 0..10)) {
            let mut doc = DiagramDocument::default();
            for (id, node) in nodes {
                doc.document.nodes = doc.document.nodes.update(id, node);
            }
            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != "internal-error"));
        }

        fn prop_validate_negative_dimensions_returns_error(
            id in arb_node_id(),
            width in -1000.0_f64..0.0_f64,
            height in -1000.0_f64..0.0_f64,
        ) {
            let mut doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat::new_unchecked(0.0),
                y: OrderedFloat::new_unchecked(0.0),
                width: OrderedFloat::new_unchecked(width),
                height: OrderedFloat::new_unchecked(height),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            };
            doc.document.nodes = doc.document.nodes.update(id, node);

            let issues = validate_document(&doc);
            prop_assert!(issues.iter().any(|i| i.code == "invalid-numeric"));
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
                x: OrderedFloat::new_unchecked(0.0),
                y: OrderedFloat::new_unchecked(0.0),
                width: OrderedFloat::new_unchecked(dim),
                height: OrderedFloat::new_unchecked(dim),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
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
                x: OrderedFloat::new_unchecked(x),
                y: OrderedFloat::new_unchecked(y),
                width: OrderedFloat::new_unchecked(64.0),
                height: OrderedFloat::new_unchecked(64.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
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
