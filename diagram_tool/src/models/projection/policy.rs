//! Cycle policy enforcement for diagram projection
//!
//! This module provides cycle policy enforcement for the diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::models::envelope::DomainOp;

use crate::models::projection::replay::apply_event;
use crate::models::projection::types::{CyclePolicy, DiagramProjection, EventRecord, ReplayError};

/// Enforce cycle policy on a diagram projection
///
/// This function checks whether the current projection violates its configured
/// cycle policy. If the policy is `CyclePolicy::Deny` and the graph contains
/// cycles, an error is returned.
///
/// # Errors
/// - Returns `ReplayError::CycleViolation` if:
///   - The cycle policy is `Deny` and the projection contains a cycle
/// - Returns `ReplayError::PolicyMissing` if:
///   - The cycle policy field is not properly initialized (should not happen with default)
pub fn enforce_cycle_policy(state: &DiagramProjection) -> Result<(), ReplayError> {
    match state.cycle_policy {
        CyclePolicy::Allow => Ok(()),
        CyclePolicy::Deny => {
            // Use the DAG validation from the dag module
            crate::models::dag::validate_dag(&state.nodes, &state.edges)
                .map_err(|e| ReplayError::CycleViolation(e.to_string()))
        }
    }
}

/// Apply a domain operation with cycle policy enforcement
///
/// This function applies an operation to the projection while respecting
/// the configured cycle policy. If the operation would create a cycle and
/// the policy is `Deny`, the operation is rejected.
///
/// # Errors
/// - Returns `ReplayError::CycleViolation` if:
///   - The operation would create a cycle and policy is `Deny`
/// - Returns `ReplayError::InvariantViolation` if:
///   - The operation itself violates an invariant (e.g., duplicate node ID)
/// - Returns `ReplayError::InvalidEvent` if:
///   - The event is malformed
pub fn apply_policy_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    // First, apply the operation to get a tentative new state
    let event = EventRecord {
        op_id: format!("policy-op-{}", state.revision),
        revision: state.revision,
        operation: op.clone(),
        author: crate::models::envelope::Author {
            id: "system".to_string(),
            name: "Policy Enforcer".to_string(),
            email: None,
        },
        timestamp: 0,
    };

    let new_state = apply_event(state, &event)?;

    // Then, enforce the cycle policy on the new state
    enforce_cycle_policy(&new_state)?;

    // If we get here, the operation is valid
    Ok(new_state)
}

/// Compute a stable hash of a diagram projection
///
/// This function produces a deterministic hash string that uniquely identifies
/// the projection state.
pub fn projection_hash(state: &DiagramProjection) -> Result<String, ReplayError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Validate that coordinates are finite (no NaN or infinity)
    validate_node_coordinates(state)?;
    validate_edge_geometry(state)?;

    // Hash version, revision, nodes, edges, and author_priority
    let hash_value = compute_hash(state)?;

    Ok(format!("{hash_value:016x}"))
}

/// Compute the hash value from all projection components
fn compute_hash(state: &DiagramProjection) -> Result<u64, ReplayError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash version and revision
    state.version.hash(&mut hasher);
    state.revision.hash(&mut hasher);

    // Hash nodes in deterministic order
    hash_sorted_items(state.nodes.keys(), state, &mut hasher, hash_node)?;

    // Hash edges in deterministic order
    hash_sorted_items(state.edges.keys(), state, &mut hasher, hash_edge)?;

    // Hash author_priority in sorted order
    hash_author_priority(state, &mut hasher)?;

    Ok(hasher.finish())
}

/// Hash a collection of items sorted by their keys
fn hash_sorted_items<K, H>(
    keys: impl Iterator<Item = K>,
    state: &DiagramProjection,
    hasher: &mut H,
    mut hash_fn: impl FnMut(K, &DiagramProjection, &mut H) -> Result<(), ReplayError>,
) -> Result<(), ReplayError>
where
    K: std::cmp::Ord,
    H: std::hash::Hasher,
{
    let mut sorted: Vec<_> = keys.collect();
    sorted.sort();
    for key in sorted {
        hash_fn(key, state, hasher)?;
    }
    Ok(())
}

/// Hash author priority map
fn hash_author_priority(
    state: &DiagramProjection,
    hasher: &mut impl std::hash::Hasher,
) -> Result<(), ReplayError> {
    use std::hash::Hash;

    let mut priority_keys: Vec<_> = state.author_priority.keys().collect();
    priority_keys.sort();
    for key in priority_keys {
        key.hash(hasher);
        let value = state.author_priority.get(key).ok_or_else(|| {
            ReplayError::InvariantViolation(format!(
                "author_priority key {key} disappeared during hashing"
            ))
        })?;
        value.hash(hasher);
    }
    Ok(())
}

/// Validate node coordinates are finite
fn validate_node_coordinates(state: &DiagramProjection) -> Result<(), ReplayError> {
    for (id, node) in state.nodes.iter() {
        validate_finite(
            id,
            "node",
            "coordinates",
            node.x.0.is_finite() && node.y.0.is_finite(),
        )?;
        validate_finite(
            id,
            "node",
            "dimensions",
            node.width.0.is_finite() && node.height.0.is_finite(),
        )?;
    }
    Ok(())
}

/// Validate edge geometry is finite
fn validate_edge_geometry(state: &DiagramProjection) -> Result<(), ReplayError> {
    for (id, edge) in state.edges.iter() {
        validate_finite(
            id,
            "edge",
            "label_offset_t",
            edge.label_offset_t.0.is_finite(),
        )?;
        validate_finite(id, "edge", "thickness", edge.thickness.0.is_finite())?;
    }
    Ok(())
}

/// Helper to validate a finite condition
fn validate_finite(
    id: &impl std::fmt::Display,
    kind: &str,
    field: &str,
    is_valid: bool,
) -> Result<(), ReplayError> {
    if !is_valid {
        return Err(ReplayError::InvariantViolation(format!(
            "{kind} {id} has non-finite {field}"
        )));
    }
    Ok(())
}

/// Hash a node into the hasher
fn hash_node(
    id: &crate::models::document::NodeId,
    state: &DiagramProjection,
    hasher: &mut std::collections::hash_map::DefaultHasher,
) -> Result<(), ReplayError> {
    use std::hash::Hash;

    id.hash(hasher);
    let node = state.nodes.get(id).ok_or_else(|| {
        ReplayError::InvariantViolation(format!("node {id} disappeared during hashing"))
    })?;

    hash_node_fields(node, hasher);
    hash_node_tags(node, hasher);
    hash_node_metadata(node, hasher)?;

    Ok(())
}

/// Hash node's basic fields
fn hash_node_fields(node: &crate::models::document::Node, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;

    node.kind.hash(hasher);
    node.icon.hash(hasher);
    node.label.hash(hasher);
    // Use bitwise representation for floats to ensure determinism
    node.x.0.to_bits().hash(hasher);
    node.y.0.to_bits().hash(hasher);
    node.width.0.to_bits().hash(hasher);
    node.height.0.to_bits().hash(hasher);
    node.font_size.hash(hasher);
    node.font_weight.hash(hasher);
    node.lock_state.hash(hasher);
    node.parent.hash(hasher);
    node.dag_rank.hash(hasher);
    node.z_index.hash(hasher);
    node.style.hash(hasher);
    node.collapsed.hash(hasher);
}

/// Hash node's tags
fn hash_node_tags(node: &crate::models::document::Node, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;

    let mut tags = node.tags.clone();
    tags.sort();
    tags.len().hash(hasher);
    for tag in tags {
        tag.hash(hasher);
    }
}

/// Hash node's metadata
fn hash_node_metadata(
    node: &crate::models::document::Node,
    hasher: &mut impl std::hash::Hasher,
) -> Result<(), ReplayError> {
    use std::hash::Hash;

    let mut metadata_keys: Vec<_> = node.metadata.keys().collect();
    metadata_keys.sort();
    for key in metadata_keys {
        key.hash(hasher);
        let value = node.metadata.get(key).ok_or_else(|| {
            ReplayError::InvariantViolation(format!(
                "metadata key {key} disappeared during hashing"
            ))
        })?;
        value.hash(hasher);
    }
    Ok(())
}

/// Hash an edge into the hasher
fn hash_edge(
    id: &crate::models::document::EdgeId,
    state: &DiagramProjection,
    hasher: &mut std::collections::hash_map::DefaultHasher,
) -> Result<(), ReplayError> {
    use std::hash::Hash;

    id.hash(hasher);
    let edge = state.edges.get(id).ok_or_else(|| {
        ReplayError::InvariantViolation(format!("edge {id} disappeared during hashing"))
    })?;

    hash_edge_fields(edge, hasher);
    hash_edge_bend_points(edge, hasher);
    hash_edge_tags(edge, hasher);
    hash_edge_metadata(edge, hasher)?;

    Ok(())
}

/// Hash edge's basic fields
fn hash_edge_fields(edge: &crate::models::document::Edge, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;

    edge.source.hash(hasher);
    edge.target.hash(hasher);
    edge.label.hash(hasher);
    edge.style.hash(hasher);
    edge.arrow_type.hash(hasher);
    edge.label_offset_t.0.to_bits().hash(hasher);
    edge.color.hash(hasher);
    edge.thickness.0.to_bits().hash(hasher);
    edge.directed.hash(hasher);
    edge.font_size.hash(hasher);
}

/// Hash edge's bend points
fn hash_edge_bend_points(
    edge: &crate::models::document::Edge,
    hasher: &mut impl std::hash::Hasher,
) {
    use std::hash::Hash;

    edge.bend_points.len().hash(hasher);
    for bp in &edge.bend_points {
        bp.x.0.to_bits().hash(hasher);
        bp.y.0.to_bits().hash(hasher);
    }
}

/// Hash edge's tags
fn hash_edge_tags(edge: &crate::models::document::Edge, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;

    let mut tags = edge.tags.clone();
    tags.sort();
    tags.len().hash(hasher);
    for tag in tags {
        tag.hash(hasher);
    }
}

/// Hash edge's metadata
fn hash_edge_metadata(
    edge: &crate::models::document::Edge,
    hasher: &mut impl std::hash::Hasher,
) -> Result<(), ReplayError> {
    use std::hash::Hash;

    let mut metadata_keys: Vec<_> = edge.metadata.keys().collect();
    metadata_keys.sort();
    for key in metadata_keys {
        key.hash(hasher);
        let value = edge.metadata.get(key).ok_or_else(|| {
            ReplayError::InvariantViolation(format!(
                "metadata key {key} disappeared during hashing"
            ))
        })?;
        value.hash(hasher);
    }
    Ok(())
}
