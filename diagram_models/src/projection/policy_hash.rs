//! Stable hashing for projection state.

use crate::projection::types::{DiagramProjection, ReplayError};

pub fn projection_hash(state: &DiagramProjection) -> Result<String, ReplayError> {
    validate_node_coordinates(state)?;
    validate_edge_geometry(state)?;
    let hash_value = compute_hash(state)?;
    Ok(format!("{hash_value:016x}"))
}

fn compute_hash(state: &DiagramProjection) -> Result<u64, ReplayError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    state.version.hash(&mut hasher);
    state.revision.hash(&mut hasher);
    hash_sorted_items(state.nodes.keys(), state, &mut hasher, hash_node)?;
    hash_sorted_items(state.edges.keys(), state, &mut hasher, hash_edge)?;
    hash_author_priority(state, &mut hasher)?;
    Ok(hasher.finish())
}

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

fn hash_node(
    id: &crate::document::NodeId,
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

fn hash_node_fields(node: &crate::document::Node, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;

    node.kind.hash(hasher);
    node.icon.hash(hasher);
    node.label.hash(hasher);
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

fn hash_node_tags(node: &crate::document::Node, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;

    let mut tags = node.tags.clone();
    tags.sort();
    tags.len().hash(hasher);
    for tag in tags {
        tag.hash(hasher);
    }
}

fn hash_node_metadata(
    node: &crate::document::Node,
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

fn hash_edge(
    id: &crate::document::EdgeId,
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

fn hash_edge_fields(edge: &crate::document::Edge, hasher: &mut impl std::hash::Hasher) {
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

fn hash_edge_bend_points(edge: &crate::document::Edge, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;

    edge.bend_points.len().hash(hasher);
    for bp in &edge.bend_points {
        bp.x.0.to_bits().hash(hasher);
        bp.y.0.to_bits().hash(hasher);
    }
}

fn hash_edge_tags(edge: &crate::document::Edge, hasher: &mut impl std::hash::Hasher) {
    use std::hash::Hash;

    let mut tags = edge.tags.clone();
    tags.sort();
    tags.len().hash(hasher);
    for tag in tags {
        tag.hash(hasher);
    }
}

fn hash_edge_metadata(
    edge: &crate::document::Edge,
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
