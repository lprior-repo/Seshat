#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::dag::validate_dag;
use crate::models::document::{DiagramDocument, NodeKind};

/// Severity of a validation issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
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
    let nodes = &doc.document.nodes;
    let edges = &doc.document.edges;

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
        DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
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
                locked: false,
                parent: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                style: NodeStyle::default(),
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
                directed: true,
                bend_points: Vec::new(),
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
    fn given_valid_document_when_validated_then_no_issues() {
        let doc = DiagramDocument::default();
        let issues = validate_document(&doc);
        assert!(issues.is_empty());
    }
}
