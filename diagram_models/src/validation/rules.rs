use super::types::{ValidationCode, ValidationIssue};
use crate::dag::{validate_dag, CycleError};
use crate::document::{DiagramDocument, DocumentData, EditorState, NodeKind};
use im::HashSet;

/// Pure function: validates a `DiagramDocument` and returns all issues found.
/// Deterministic, no side effects, collect-all pattern (never short-circuits).
#[must_use]
pub fn validate_document(doc: &DiagramDocument) -> Vec<ValidationIssue> {
    check_version(doc)
        .into_iter()
        .chain(validate_document_data(&doc.document))
        .chain(check_editor_state(&doc.editor_state))
        .collect()
}

/// Pure function: validates only the structural document graph.
/// Editor/camera state is intentionally ignored.
#[must_use]
pub fn validate_document_data(document: &DocumentData) -> Vec<ValidationIssue> {
    let nodes = &document.nodes;
    let edges = &document.edges;
    check_edge_properties(edges, nodes)
        .chain(check_nodes(nodes))
        .chain(check_parent_cycles(nodes))
        .chain(check_dag(nodes, edges))
        .collect()
}

/// Validates whether the document version is 2.
fn check_version(doc: &DiagramDocument) -> Option<ValidationIssue> {
    (doc.version != 2).then(|| {
        ValidationIssue::error(
            ValidationCode::INVALID_VERSION,
            format!("Document version must be 2, got {}", doc.version),
            None,
        )
    })
}
/// Validates that the editor state has finite camera/zoom values.
fn check_editor_state(es: &EditorState) -> Vec<ValidationIssue> {
    [
        ("camera_x", es.camera_x.0),
        ("camera_y", es.camera_y.0),
        ("zoom", es.zoom.0),
    ]
    .into_iter()
    .filter(|(_, v)| !v.is_finite())
    .map(|(name, v)| {
        ValidationIssue::error(
            ValidationCode::EDITOR_INVALID_STATE,
            format!("Editor state has non-finite {name}: {v}"),
            None,
        )
    })
    .collect()
}

/// Checks parent chain cycles via DFS with visited set (persistent, no mut).
fn check_parent_cycles(
    nodes: &im::HashMap<crate::document::NodeId, crate::document::Node>,
) -> Vec<ValidationIssue> {
    nodes
        .keys()
        .filter(|&id| detect_cycle(nodes, id, &HashSet::new()))
        .map(|id| {
            ValidationIssue::error(
                ValidationCode::PARENT_CYCLE,
                format!("Circular parent chain detected involving node {id}"),
                Some(id.to_string()),
            )
        })
        .collect()
}
/// DFS cycle detection for parent chains. Returns true if a cycle exists.
fn detect_cycle(
    nodes: &im::HashMap<crate::document::NodeId, crate::document::Node>,
    current: &crate::document::NodeId,
    visited: &HashSet<crate::document::NodeId>,
) -> bool {
    if visited.contains(current) {
        return true;
    }
    let next = visited.update(current.clone());
    nodes
        .get(current)
        .and_then(|n| n.parent.as_ref())
        .is_some_and(|p| detect_cycle(nodes, p, &next))
}
/// Validates edge properties: dangling, `label_offset_t`, thickness, color, `font_size`.
fn check_edge_properties<'a>(
    edges: &'a im::HashMap<crate::document::EdgeId, crate::document::Edge>,
    nodes: &'a im::HashMap<crate::document::NodeId, crate::document::Node>,
) -> impl Iterator<Item = ValidationIssue> + 'a {
    edges.iter().flat_map(|(id, edge)| {
        check_edge_dangling(id, edge, nodes)
            .chain(check_label_offset(id, edge))
            .chain(check_thickness(id, edge))
            .chain(check_color(id, edge))
            .chain(check_font_size(id, edge))
    })
}

/// Checks that edge source and target reference existing nodes.
fn check_edge_dangling(
    id: &crate::document::EdgeId,
    edge: &crate::document::Edge,
    nodes: &im::HashMap<crate::document::NodeId, crate::document::Node>,
) -> impl Iterator<Item = ValidationIssue> {
    let src = (!nodes.contains_key(&edge.source)).then(|| {
        ValidationIssue::error(
            ValidationCode::EDGE_DANGLING,
            format!("Edge {id} source '{}' does not exist", edge.source),
            Some(id.to_string()),
        )
    });
    let tgt = (!nodes.contains_key(&edge.target)).then(|| {
        ValidationIssue::error(
            ValidationCode::EDGE_DANGLING,
            format!("Edge {id} target '{}' does not exist", edge.target),
            Some(id.to_string()),
        )
    });
    src.into_iter().chain(tgt)
}
/// Validates `label_offset_t` is finite and in [0.0, 1.0].
fn check_label_offset(
    id: &crate::document::EdgeId,
    edge: &crate::document::Edge,
) -> Option<ValidationIssue> {
    let v = edge.label_offset_t.0;
    (!v.is_finite() || !(0.0..=1.0).contains(&v)).then(|| {
        ValidationIssue::error(
            ValidationCode::EDGE_INVALID_OFFSET,
            format!("Edge {id:?} has label_offset_t {v} outside valid range [0, 1]"),
            Some(id.to_string()),
        )
    })
}

/// Validates thickness is finite and non-negative.
fn check_thickness(
    id: &crate::document::EdgeId,
    edge: &crate::document::Edge,
) -> Option<ValidationIssue> {
    let v = edge.thickness.0;
    (!v.is_finite() || v < 0.0).then(|| {
        ValidationIssue::error(
            ValidationCode::EDGE_INVALID_THICKNESS,
            format!("Edge {id:?} has invalid thickness: {v}"),
            Some(id.to_string()),
        )
    })
}

/// Validates color hex format if present.
fn check_color(
    id: &crate::document::EdgeId,
    edge: &crate::document::Edge,
) -> Option<ValidationIssue> {
    edge.color
        .as_ref()
        .filter(|c| !is_valid_hex_color(c))
        .map(|c| {
            ValidationIssue::error(
                ValidationCode::EDGE_INVALID_COLOR,
                format!("Edge {id:?} has invalid color format: {c}"),
                Some(id.to_string()),
            )
        })
}

/// Validates `font_size` is finite if present.
fn check_font_size(
    id: &crate::document::EdgeId,
    edge: &crate::document::Edge,
) -> Option<ValidationIssue> {
    edge.font_size
        .as_ref()
        .filter(|fs| !fs.0.is_finite())
        .map(|fs| {
            ValidationIssue::error(
                ValidationCode::EDGE_INVALID_FONT_SIZE,
                format!("Edge {id:?} has non-finite font_size: {}", fs.0),
                Some(id.to_string()),
            )
        })
}

/// Validates nodes: coordinates, dimensions, parent references.
fn check_nodes(
    nodes: &im::HashMap<crate::document::NodeId, crate::document::Node>,
) -> impl Iterator<Item = ValidationIssue> + '_ {
    nodes.iter().flat_map(|(id, node)| {
        check_parent(id, node, nodes)
            .into_iter()
            .chain(check_coordinates(id, node))
            .chain(check_dimensions(id, node))
    })
}

/// Validates node parent exists and is a Subgraph.
fn check_parent(
    id: &crate::document::NodeId,
    node: &crate::document::Node,
    nodes: &im::HashMap<crate::document::NodeId, crate::document::Node>,
) -> Option<ValidationIssue> {
    node.parent.as_ref().and_then(|parent_id| {
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
    })
}

/// Validates node coordinates are finite.
fn check_coordinates(
    id: &crate::document::NodeId,
    node: &crate::document::Node,
) -> Option<ValidationIssue> {
    (!node.x.0.is_finite() || !node.y.0.is_finite()).then(|| {
        ValidationIssue::error(
            ValidationCode::INVALID_NUMERIC,
            format!(
                "Node {id} has non-finite coordinates: x={}, y={}",
                node.x.0, node.y.0
            ),
            Some(id.to_string()),
        )
    })
}

/// Validates node dimensions are finite and non-negative.
fn check_dimensions(
    id: &crate::document::NodeId,
    node: &crate::document::Node,
) -> Option<ValidationIssue> {
    (node.width.0 < 0.0
        || node.height.0 < 0.0
        || !node.width.0.is_finite()
        || !node.height.0.is_finite())
    .then(|| {
        ValidationIssue::error(
            ValidationCode::INVALID_NUMERIC,
            format!(
                "Node {id} has invalid dimensions: width={}, height={}",
                node.width.0, node.height.0
            ),
            Some(id.to_string()),
        )
    })
}

/// Validates the DAG property: acyclic and connected (via petgraph).
fn check_dag(
    nodes: &im::HashMap<crate::document::NodeId, crate::document::Node>,
    edges: &im::HashMap<crate::document::EdgeId, crate::document::Edge>,
) -> Vec<ValidationIssue> {
    validate_dag(nodes, edges)
        .err()
        .map(|e| {
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
        })
        .into_iter()
        .collect()
}

/// Checks if a string is a valid hex color: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`.
/// Case-insensitive via `is_ascii_hexdigit()`.
#[must_use]
pub fn is_valid_hex_color(color: &str) -> bool {
    color.starts_with('#')
        && matches!(color.len(), 4 | 5 | 7 | 9)
        && color[1..].chars().all(|c| c.is_ascii_hexdigit())
}
