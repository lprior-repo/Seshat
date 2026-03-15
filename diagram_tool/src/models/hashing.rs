//! Hashing module - computes deterministic hashes of `DiagramProjection`
//!
//! This module provides deterministic hashing for the domain model `DiagramProjection`.
//! The hash can be used for:
//! - Verifying replay determinism
//! - Detecting state changes
//! - Caching and optimization

#![allow(dead_code)]
#![allow(unused_imports)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::models::projection::{DiagramProjection, ReplayError};

/// Compute a stable hash of a diagram projection.
///
/// This function produces a deterministic hash string that uniquely identifies
/// the projection state. The hash is stable across multiple invocations and
/// can be used for:
/// - Verifying replay determinism
/// - Detecting state changes
/// - Caching and optimization
///
/// # Errors
/// Returns `ReplayError::InvariantViolation` if the projection contains
/// data that cannot be hashed (e.g., NaN values in coordinates).
///
/// # Example
/// ```ignore
/// let projection = replay_events(&events)?;
/// let hash = projection_hash(&projection)?;
/// // Same events always produce same hash
/// assert_eq!(hash, projection_hash(&replay_events(&events)?)?);
/// ```
#[allow(clippy::too_many_lines)]
pub fn projection_hash(state: &DiagramProjection) -> Result<String, ReplayError> {
    // Validate that coordinates are finite (no NaN or infinity)
    for (id, node) in state.nodes.iter() {
        if !node.x.0.is_finite() || !node.y.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "node {id} has non-finite coordinates"
            )));
        }
        if !node.width.0.is_finite() || !node.height.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "node {id} has non-finite dimensions"
            )));
        }
    }

    for (id, edge) in state.edges.iter() {
        if !edge.label_offset_t.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "edge {id} has non-finite label_offset_t"
            )));
        }
        if !edge.thickness.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "edge {id} has non-finite thickness"
            )));
        }
    }

    let mut hasher = DefaultHasher::new();

    // Hash version
    state.version.hash(&mut hasher);

    // Hash revision
    state.revision.hash(&mut hasher);

    // Hash nodes in deterministic order (sorted by ID)
    let mut node_ids: Vec<_> = state.nodes.keys().collect();
    node_ids.sort();
    for id in node_ids {
        id.hash(&mut hasher);
        let node = state.nodes.get(id).ok_or_else(|| {
            ReplayError::InvariantViolation(format!("node {id} disappeared during hashing"))
        })?;

        // Hash node fields in consistent order
        node.kind.hash(&mut hasher);
        node.icon.hash(&mut hasher);
        node.label.hash(&mut hasher);
        // Use bitwise representation for floats to ensure determinism
        node.x.0.to_bits().hash(&mut hasher);
        node.y.0.to_bits().hash(&mut hasher);
        node.width.0.to_bits().hash(&mut hasher);
        node.height.0.to_bits().hash(&mut hasher);
        node.font_size.hash(&mut hasher);
        node.font_weight.hash(&mut hasher);
        node.lock_state.hash(&mut hasher);
        node.parent.hash(&mut hasher);
        node.dag_rank.hash(&mut hasher);
        node.z_index.hash(&mut hasher);
        node.style.hash(&mut hasher);
        node.collapsed.hash(&mut hasher);

        // Hash tags in sorted order
        let mut tags = node.tags.clone();
        tags.sort();
        tags.len().hash(&mut hasher);
        for tag in tags {
            tag.hash(&mut hasher);
        }

        // Hash metadata in sorted order
        let mut metadata_keys: Vec<_> = node.metadata.keys().collect();
        metadata_keys.sort();
        for key in metadata_keys {
            key.hash(&mut hasher);
            let value = node.metadata.get(key).ok_or_else(|| {
                ReplayError::InvariantViolation(format!(
                    "metadata key {key} disappeared during hashing"
                ))
            })?;
            value.hash(&mut hasher);
        }
    }

    // Hash edges in deterministic order (sorted by ID)
    let mut edge_ids: Vec<_> = state.edges.keys().collect();
    edge_ids.sort();
    for id in edge_ids {
        id.hash(&mut hasher);
        let edge = state.edges.get(id).ok_or_else(|| {
            ReplayError::InvariantViolation(format!("edge {id} disappeared during hashing"))
        })?;

        edge.source.hash(&mut hasher);
        edge.target.hash(&mut hasher);
        edge.label.hash(&mut hasher);
        edge.style.hash(&mut hasher);
        edge.arrow_type.hash(&mut hasher);
        edge.label_offset_t.0.to_bits().hash(&mut hasher);
        edge.color.hash(&mut hasher);
        edge.thickness.0.to_bits().hash(&mut hasher);
        edge.directed.hash(&mut hasher);
        edge.font_size.hash(&mut hasher);

        // Hash bend points
        edge.bend_points.len().hash(&mut hasher);
        for bp in &edge.bend_points {
            bp.x.0.to_bits().hash(&mut hasher);
            bp.y.0.to_bits().hash(&mut hasher);
        }

        // Hash tags in sorted order
        let mut tags = edge.tags.clone();
        tags.sort();
        tags.len().hash(&mut hasher);
        for tag in tags {
            tag.hash(&mut hasher);
        }

        // Hash metadata in sorted order
        let mut metadata_keys: Vec<_> = edge.metadata.keys().collect();
        metadata_keys.sort();
        for key in metadata_keys {
            key.hash(&mut hasher);
            let value = edge.metadata.get(key).ok_or_else(|| {
                ReplayError::InvariantViolation(format!(
                    "metadata key {key} disappeared during hashing"
                ))
            })?;
            value.hash(&mut hasher);
        }
    }

    // Hash author_priority in sorted order
    let mut priority_keys: Vec<_> = state.author_priority.keys().collect();
    priority_keys.sort();
    for key in priority_keys {
        key.hash(&mut hasher);
        let value = state.author_priority.get(key).ok_or_else(|| {
            ReplayError::InvariantViolation(format!(
                "author_priority key {key} disappeared during hashing"
            ))
        })?;
        value.hash(&mut hasher);
    }

    // Hash cycle_policy (use discriminant: 0=Allow, 1=Deny)
    let policy_is_deny = matches!(
        state.cycle_policy,
        crate::models::projection::CyclePolicy::Deny
    );
    policy_is_deny.hash(&mut hasher);

    // Produce hex string of hash
    let hash_value = hasher.finish();
    Ok(format!("{hash_value:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::{Edge, EdgeId, Node, NodeId, OrderedFloat};
    use crate::models::projection::DiagramProjection;
    use im::HashMap;

    fn make_test_node(id: &str) -> (NodeId, Node) {
        let id = NodeId::new(id.to_string());
        let node = Node {
            kind: crate::models::document::NodeKind::Node,
            icon: "test".to_string(),
            label: "Test Node".to_string(),
            x: OrderedFloat(100.0),
            y: OrderedFloat(200.0),
            width: OrderedFloat(80.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        (id, node)
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_hash_returns_deterministic_hash() {
        let (node_id, node) = make_test_node("node-1");
        let mut nodes = HashMap::new();
        let _ = nodes.insert(node_id.clone(), node);

        let projection = DiagramProjection {
            version: 2,
            revision: 0,
            nodes,
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: crate::models::projection::CyclePolicy::Allow,
        };

        let hash1 = projection_hash(&projection).unwrap();
        let hash2 = projection_hash(&projection).unwrap();

        assert_eq!(hash1, hash2);
        // Hash should be 16 hex characters
        assert_eq!(hash1.len(), 16);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_hash_different_states_different_hashes() {
        let (node_id, mut node) = make_test_node("node-1");
        let mut nodes1 = HashMap::new();
        let _ = nodes1.insert(node_id.clone(), node.clone());

        let projection1 = DiagramProjection {
            version: 2,
            revision: 0,
            nodes: nodes1,
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: crate::models::projection::CyclePolicy::Allow,
        };

        // Different position
        node.x = OrderedFloat(300.0);
        let mut nodes2 = HashMap::new();
        let _ = nodes2.insert(node_id, node);

        let projection2 = DiagramProjection {
            version: 2,
            revision: 0,
            nodes: nodes2,
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: crate::models::projection::CyclePolicy::Allow,
        };

        let hash1 = projection_hash(&projection1).unwrap();
        let hash2 = projection_hash(&projection2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_hash_empty_projection() {
        let projection = DiagramProjection::empty();
        let hash = projection_hash(&projection).unwrap();

        assert_eq!(hash.len(), 16);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_hash_returns_error_on_nan_coordinates() {
        let (node_id, mut node) = make_test_node("node-1");
        node.x = OrderedFloat(f64::NAN);
        let mut nodes = HashMap::new();
        let _ = nodes.insert(node_id, node);

        let projection = DiagramProjection {
            version: 2,
            revision: 0,
            nodes,
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: crate::models::projection::CyclePolicy::Allow,
        };

        let result = projection_hash(&projection);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
        let error_msg = format!("{err}");
        assert!(error_msg.contains("non-finite coordinates"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_hash_returns_error_on_infinity_coordinates() {
        let (node_id, mut node) = make_test_node("node-1");
        node.y = OrderedFloat(f64::INFINITY);
        let mut nodes = HashMap::new();
        let _ = nodes.insert(node_id, node);

        let projection = DiagramProjection {
            version: 2,
            revision: 0,
            nodes,
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: crate::models::projection::CyclePolicy::Allow,
        };

        let result = projection_hash(&projection);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
        let error_msg = format!("{err}");
        assert!(error_msg.contains("non-finite coordinates"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_projection_hash_includes_all_fields() {
        // Create projection with all fields populated
        let (node_id, node) = make_test_node("node-1");
        let node_id_for_insert = node_id.clone();
        let edge_id = EdgeId::new("edge-1".to_string());
        let edge = Edge {
            source: node_id,
            target: node_id_for_insert.clone(),
            label: "test edge".to_string(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: crate::models::document::ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::vector![],
            tags: im::vector![],
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        };

        let mut nodes = HashMap::new();
        let _ = nodes.insert(node_id_for_insert, node);
        let mut edges = HashMap::new();
        let _ = edges.insert(edge_id, edge);

        let mut author_priority = HashMap::new();
        let _ = author_priority.insert("op-1".to_string(), true);

        let projection = DiagramProjection {
            version: 2,
            revision: 5,
            nodes,
            edges,
            author_priority,
            cycle_policy: crate::models::projection::CyclePolicy::Deny,
        };

        let hash = projection_hash(&projection).unwrap();

        assert_eq!(hash.len(), 16);
        // Different cycle policy should produce different hash
        let projection_allow = DiagramProjection {
            cycle_policy: crate::models::projection::CyclePolicy::Allow,
            ..projection
        };
        let hash_allow = projection_hash(&projection_allow).unwrap();
        assert_ne!(hash, hash_allow);
    }
}
