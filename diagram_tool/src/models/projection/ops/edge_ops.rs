//! Edge operations for diagram projection
//!
//! This module provides functions for applying edge-related operations
//! to a diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use im::HashMap;

use crate::models::document::{ArrowType, Edge, EdgeId, EdgeStyle, NodeId, OrderedFloat};
use crate::models::envelope::DomainOp;

use crate::models::projection::types::{DiagramProjection, ReplayError};

/// Build result projection with updated edges
fn build_edge_result(
    state: DiagramProjection,
    new_edges: im::HashMap<EdgeId, Edge>,
) -> DiagramProjection {
    DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: state.nodes,
        edges: new_edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    }
}

/// Parse edge IDs from connection parameters
fn parse_edge_ids(id: &str, source: &str, target: &str) -> (EdgeId, NodeId, NodeId) {
    let edge_id = EdgeId::new(id.to_string());
    let source_id = NodeId::new(source.to_string());
    let target_id = NodeId::new(target.to_string());
    (edge_id, source_id, target_id)
}

/// Apply `EdgeConnect` operation
pub fn apply_edge_connect(
    state: DiagramProjection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<DiagramProjection, ReplayError> {
    let (edge_id, source_id, target_id) = parse_edge_ids(id, source, target);

    // Check for duplicate edge ID
    if state.has_edge(&edge_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "duplicate edge ID: {id}"
        )));
    }

    // Validate source and target nodes exist
    validate_edge_endpoints(&state, &source_id, &target_id, source, target)?;

    let new_edges = state
        .edges
        .update(edge_id, create_default_edge(source_id, target_id));

    Ok(build_edge_result(state, new_edges))
}

/// Validate that edge source and target nodes exist
fn validate_edge_endpoints(
    state: &DiagramProjection,
    source_id: &NodeId,
    target_id: &NodeId,
    source_str: &str,
    target_str: &str,
) -> Result<(), ReplayError> {
    if !state.has_node(source_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "source node not found: {source_str}"
        )));
    }
    if !state.has_node(target_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "target node not found: {target_str}"
        )));
    }
    Ok(())
}

/// Validate edge doesn't already exist
fn validate_no_duplicate_edge(
    state: &DiagramProjection,
    edge_id: &EdgeId,
    id: &str,
) -> Result<(), ReplayError> {
    if state.has_edge(edge_id) {
        return Err(ReplayError::DuplicateEdge(id.to_string()));
    }
    Ok(())
}

/// Validate source and target nodes exist for connect
fn validate_connect_nodes(
    state: &DiagramProjection,
    source_id: &NodeId,
    target_id: &NodeId,
    source: &str,
    target: &str,
) -> Result<(), ReplayError> {
    if !state.has_node(source_id) {
        return Err(ReplayError::PolicyViolation(format!(
            "source node not found: {source}"
        )));
    }
    if !state.has_node(target_id) {
        return Err(ReplayError::PolicyViolation(format!(
            "target node not found: {target}"
        )));
    }
    Ok(())
}

/// Create a default edge with standard settings
pub fn create_default_edge(source_id: NodeId, target_id: NodeId) -> Edge {
    Edge {
        source: source_id,
        target: target_id,
        label: String::new(),
        style: EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
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
    }
}

/// Apply `EdgeDisconnect` operation
pub fn apply_edge_disconnect(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());

    // Check edge exists
    if !state.has_edge(&edge_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "edge not found: {id}"
        )));
    }

    let new_edges = state.edges.without(&edge_id);
    Ok(build_edge_result(state, new_edges))
}

/// Apply an edge operation to the projection
///
/// This is the contract-specified entry point for applying edge operations.
/// It dispatches to the appropriate handler based on the operation type.
///
/// # Errors
/// - Returns `ReplayError::EdgeNotFound` if the edge does not exist for disconnect operations
/// - Returns `ReplayError::DuplicateEdge` if the edge already exists for connect operations
/// - Returns `ReplayError::PolicyViolation` if the operation violates policy constraints
/// - Returns `ReplayError::InvalidEvent` if the operation is not an edge operation
pub fn apply_edge_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    match op {
        DomainOp::EdgeConnect { id, source, target } => {
            apply_edge_connect_checked(state, id, source, target)
        }
        DomainOp::EdgeDisconnect { id } => apply_edge_disconnect_checked(state, id),
        _ => Err(ReplayError::InvalidEvent(format!(
            "not an edge operation: {:?}",
            op.kind()
        ))),
    }
}

/// Apply `EdgeConnect` operation with contract-specified error types
pub fn apply_edge_connect_checked(
    state: DiagramProjection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<DiagramProjection, ReplayError> {
    let (edge_id, source_id, target_id) = parse_edge_ids(id, source, target);

    // Validate edge doesn't exist
    validate_no_duplicate_edge(&state, &edge_id, id)?;

    // Validate source and target nodes exist
    validate_connect_nodes(&state, &source_id, &target_id, source, target)?;

    let new_edges = state
        .edges
        .update(edge_id, create_default_edge(source_id, target_id));

    Ok(build_edge_result(state, new_edges))
}

/// Validate edge exists for disconnect
fn validate_edge_exists(
    state: &DiagramProjection,
    edge_id: &EdgeId,
    id: &str,
) -> Result<(), ReplayError> {
    if !state.has_edge(edge_id) {
        return Err(ReplayError::EdgeNotFound(id.to_string()));
    }
    Ok(())
}

/// Apply `EdgeDisconnect` operation with contract-specified error types
pub fn apply_edge_disconnect_checked(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());

    // Validate edge exists
    validate_edge_exists(&state, &edge_id, id)?;

    let new_edges = state.edges.without(&edge_id);
    Ok(build_edge_result(state, new_edges))
}

/// Verify edge tolerance constraints in the projection
///
/// This function validates that all edges in the projection satisfy
/// the defined tolerance boundaries:
/// - All edges reference existing source and target nodes
/// - No duplicate edge IDs exist
/// - All edges have valid geometry (finite coordinates)
///
/// # Errors
/// - Returns `ReplayError::PolicyViolation` if any edge references a non-existent node
/// - Returns `ReplayError::DuplicateEdge` if duplicate edge IDs are detected
/// - Returns `ReplayError::InvariantViolation` if edge geometry is invalid
pub fn verify_edge_tolerance(state: &DiagramProjection) -> Result<(), ReplayError> {
    // Track seen edge IDs to detect duplicates
    let mut seen_ids = std::collections::HashSet::new();

    for (edge_id, edge) in state.edges.iter() {
        check_edge_id_unique(&mut seen_ids, edge_id)?;
        verify_edge_endpoints(state, edge_id, edge)?;
        verify_edge_geometry(edge_id, edge)?;
    }

    Ok(())
}

/// Check that edge ID is unique (no duplicates)
fn check_edge_id_unique(
    seen_ids: &mut std::collections::HashSet<String>,
    edge_id: &EdgeId,
) -> Result<(), ReplayError> {
    let id_str = edge_id.to_string();
    if !seen_ids.insert(id_str.clone()) {
        return Err(ReplayError::DuplicateEdge(id_str));
    }
    Ok(())
}

/// Verify that edge source and target nodes exist
fn verify_edge_endpoints(
    state: &DiagramProjection,
    edge_id: &EdgeId,
    edge: &Edge,
) -> Result<(), ReplayError> {
    if !state.has_node(&edge.source) {
        return Err(ReplayError::PolicyViolation(format!(
            "edge {} references non-existent source node: {}",
            edge_id, edge.source
        )));
    }
    if !state.has_node(&edge.target) {
        return Err(ReplayError::PolicyViolation(format!(
            "edge {} references non-existent target node: {}",
            edge_id, edge.target
        )));
    }
    Ok(())
}

/// Verify edge geometry values are finite
fn verify_edge_geometry(edge_id: &EdgeId, edge: &Edge) -> Result<(), ReplayError> {
    if !edge.label_offset_t.0.is_finite() {
        return Err(ReplayError::InvariantViolation(format!(
            "edge {} has invalid label_offset_t",
            edge_id
        )));
    }
    if !edge.thickness.0.is_finite() {
        return Err(ReplayError::InvariantViolation(format!(
            "edge {} has invalid thickness",
            edge_id
        )));
    }
    Ok(())
}

/// Apply update edge label operation
pub fn apply_update_edge_label(
    state: DiagramProjection,
    id: &str,
    label: &String,
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());

    if let Some(edge) = state.edges.get(&edge_id) {
        let mut updated_edge = edge.clone();
        updated_edge.label = label.clone();

        let new_edges = state.edges.update(edge_id, updated_edge);

        Ok(DiagramProjection {
            edges: new_edges,
            ..state
        })
    } else {
        Err(ReplayError::InvariantViolation(format!(
            "edge not found: {id}"
        )))
    }
}
