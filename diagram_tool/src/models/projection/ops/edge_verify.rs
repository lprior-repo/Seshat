//! Edge verification for diagram projection
//!
//! This module provides verification functions for ensuring edge integrity
//! in the diagram projection.

use crate::models::document::{Edge, EdgeId, NodeId};
use crate::models::projection::types::{DiagramProjection, ReplayError};

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
    use crate::models::document::OrderedFloat;

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
