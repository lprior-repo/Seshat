#![allow(dead_code)]

use crate::models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, NodeId, OrderedFloat,
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    #[error("Source node {0} not found")]
    SourceNotFound(NodeId),
    #[error("Target node {0} not found")]
    TargetNotFound(NodeId),
    #[error("Cannot create self-loop on node {0}")]
    SelfLoop(NodeId),
    #[error("Adding this edge creates a cycle")]
    CycleDetected,
}

/// Creates a new edge between two nodes.
///
/// # Errors
///
/// Returns `RoutingError::SelfLoop` if source and target are the same.
/// Returns `RoutingError::SourceNotFound` if source node doesn't exist.
/// Returns `RoutingError::TargetNotFound` if target node doesn't exist.
/// Returns `RoutingError::CycleDetected` if the edge would create a cycle.
pub fn create_edge(
    doc: &mut DiagramDocument,
    source: NodeId,
    target: NodeId,
    edge_id: EdgeId,
) -> Result<(), RoutingError> {
    if source == target {
        return Err(RoutingError::SelfLoop(source));
    }

    if !doc.document.nodes.contains_key(&source) {
        return Err(RoutingError::SourceNotFound(source));
    }

    if !doc.document.nodes.contains_key(&target) {
        return Err(RoutingError::TargetNotFound(target));
    }

    // Check for cycles
    if creates_cycle(doc, &source, &target) {
        return Err(RoutingError::CycleDetected);
    }

    let new_edge = Edge {
        source,
        target,
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
        label_offset_t: OrderedFloat::default(),
        color: None,
        thickness: OrderedFloat::default(),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        font_size: None,
    };

    doc.document.edges.insert(edge_id, new_edge);
    Ok(())
}

fn creates_cycle(doc: &DiagramDocument, from: &NodeId, to: &NodeId) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut queue = vec![to.clone()];

    while let Some(current) = queue.pop() {
        if current == *from {
            return true;
        }

        if visited.insert(current.clone()) {
            for (_, edge) in &doc.document.edges {
                if edge.source == current {
                    queue.push(edge.target.clone());
                }
            }
        }
    }

    false
}

#[cfg(test)]
#[path = "routing_tests.rs"]
mod tests;
