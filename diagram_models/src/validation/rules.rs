use super::types::{ValidationCode, ValidationIssue};
use crate::dag::{validate_dag, CycleError};
use crate::document::{DiagramDocument, DocumentData, NodeKind};

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
        let src_issue = (!nodes.contains_key(&edge.source)).then(|| {
            ValidationIssue::error(
                ValidationCode::EDGE_DANGLING,
                format!("Edge {id} source '{}' does not exist", edge.source),
                Some(id.to_string()),
            )
        });

        let tgt_issue = (!nodes.contains_key(&edge.target)).then(|| {
            ValidationIssue::error(
                ValidationCode::EDGE_DANGLING,
                format!("Edge {id} target '{}' does not exist", edge.target),
                Some(id.to_string()),
            )
        });

        src_issue.into_iter().chain(tgt_issue)
    });

    let node_issues = nodes.iter().flat_map(|(id, node)| {
        let parent_issue = node.parent.as_ref().and_then(|parent_id| {
            if !nodes.contains_key(parent_id) {
                Some(ValidationIssue::error(
                    ValidationCode::INVALID_PARENT,
                    format!("Node {id} references non-existent parent {parent_id}"),
                    Some(id.to_string()),
                ))
            } else if nodes
                .get(parent_id)
                .is_some_and(|p| p.kind != NodeKind::Subgraph)
            {
                Some(ValidationIssue::error(
                    ValidationCode::INVALID_PARENT,
                    format!("Node {id} parent {parent_id} is not a Subgraph"),
                    Some(id.to_string()),
                ))
            } else {
                None
            }
        });

        let nan_issue = if !node.x.0.is_finite() || !node.y.0.is_finite() {
            Some(ValidationIssue::error(
                ValidationCode::INVALID_NUMERIC,
                format!(
                    "Node {id} has non-finite coordinates: x={}, y={}",
                    node.x.0, node.y.0
                ),
                Some(id.to_string()),
            ))
        } else {
            None
        };

        let dimension_issue = if node.width.0 < 0.0
            || node.height.0 < 0.0
            || !node.width.0.is_finite()
            || !node.height.0.is_finite()
        {
            Some(ValidationIssue::error(
                ValidationCode::INVALID_NUMERIC,
                format!(
                    "Node {id} has invalid dimensions: width={}, height={}",
                    node.width.0, node.height.0
                ),
                Some(id.to_string()),
            ))
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
                ValidationCode::DAG_CYCLE,
                "Document contains a cycle — DAGs must be acyclic".to_string(),
            ),
            CycleError::DisconnectedGraph(n) => (
                ValidationCode::DAG_DISCONNECTED,
                format!("Graph has {n} disconnected components — all nodes must be connected"),
            ),
        };
        ValidationIssue::error(code, message, None)
    });

    edge_issues.chain(node_issues).chain(dag_issues).collect()
}
