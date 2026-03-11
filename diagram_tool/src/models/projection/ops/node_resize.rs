//! Node resize operations for diagram projection
//!
//! This module provides the apply_node_resize function for handling
//! node resize operations in the diagram projection.

use crate::models::document::{NodeId, OrderedFloat};
use crate::models::projection::ops::node_bounds::propagate_bounds_to_ancestors;
use crate::models::projection::types::{DiagramProjection, ProjectionError};

/// Apply node resize operation
pub fn apply_node_resize(
    state: DiagramProjection,
    id: &NodeId,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<DiagramProjection, ProjectionError> {
    validate_dimensions(width, height)?;
    let node = state
        .nodes
        .get(id)
        .ok_or_else(|| ProjectionError::NodeNotFound(id.to_string()))?;
    let mut updated = node.clone();
    updated.x = OrderedFloat(x);
    updated.y = OrderedFloat(y);
    updated.width = OrderedFloat(width);
    updated.height = OrderedFloat(height);
    let new_nodes = state.nodes.update(id.clone(), updated);
    let new_nodes = propagate_bounds_to_ancestors(new_nodes, id);
    Ok(DiagramProjection {
        nodes: new_nodes,
        ..state
    })
}

fn validate_dimensions(width: f64, height: f64) -> Result<(), ProjectionError> {
    if !width.is_finite() || width <= 0.0 {
        return Err(ProjectionError::InvalidDimensions(format!(
            "invalid width: {width}"
        )));
    }
    if !height.is_finite() || height <= 0.0 {
        return Err(ProjectionError::InvalidDimensions(format!(
            "invalid height: {height}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::projection::types::DiagramProjection;
    use im::hashmap;

    fn create_test_projection() -> DiagramProjection {
        let node = crate::models::document::Node::new(
            NodeId::new("test-node".to_string()),
            0.0,
            0.0,
            100.0,
            50.0,
            "Test",
        );
        DiagramProjection {
            version: 1,
            nodes: hashmap! { node.id.clone() => node },
            edges: hashmap! {},
            groups: hashmap! {},
            revision: 1,
        }
    }

    #[test]
    fn given_valid_resize_when_applying_then_updates_dimensions() {
        let projection = create_test_projection();
        let node_id = NodeId::new("test-node".to_string());
        let result = apply_node_resize(projection, &node_id, 10.0, 20.0, 200.0, 100.0);
        assert!(result.is_ok());
        let updated = result.unwrap();
        let node = updated.nodes.get(&node_id).unwrap();
        assert_eq!(node.width.0, 200.0);
        assert_eq!(node.height.0, 100.0);
    }

    #[test]
    fn given_invalid_width_then_returns_error() {
        let projection = create_test_projection();
        let node_id = NodeId::new("test-node".to_string());
        let result = apply_node_resize(projection, &node_id, 0.0, 0.0, 0.0, 50.0);
        assert!(result.is_err());
    }

    #[test]
    fn given_nan_width_then_returns_error() {
        let projection = create_test_projection();
        let node_id = NodeId::new("test-node".to_string());
        let result = apply_node_resize(projection, &node_id, 0.0, 0.0, f64::NAN, 50.0);
        assert!(result.is_err());
    }
}
