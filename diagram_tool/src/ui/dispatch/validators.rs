//! Validation functions for dispatch operations

use diagram_models::dag::validate_dag;
use diagram_models::document::ArrowType;
use diagram_models::document::{Edge, EdgeId, NodeId, OrderedFloat};

/// Validate that coordinates are finite (not NaN or Infinity)
#[must_use]
pub const fn validate_coordinates(x: f64, y: f64) -> bool {
    x.is_finite() && y.is_finite()
}

/// Validate that dimensions are positive
#[must_use]
pub fn validate_dimensions(width: f64, height: f64) -> bool {
    width > 0.0 && height > 0.0 && width.is_finite() && height.is_finite()
}

/// Check if adding an edge would preserve the DAG (no cycles)
#[must_use]
pub fn edge_preserves_dag(
    nodes: &im::HashMap<NodeId, diagram_models::document::Node>,
    edges: &im::HashMap<EdgeId, Edge>,
    source: &NodeId,
    target: &NodeId,
) -> bool {
    // Self-loop check
    if source == target {
        return false;
    }

    // Create candidate edges with the new edge added
    let candidate_edge = diagram_models::document::Edge {
        source: source.clone(),
        target: target.clone(),
        label: String::new(),
        style: diagram_models::document::EdgeStyle::Solid,
        arrow_type: ArrowType::default(),
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    };

    let mut candidate_edges = edges.clone();
    candidate_edges.insert(
        EdgeId::new(uuid::Uuid::new_v4().to_string()),
        candidate_edge,
    );

    validate_dag(nodes, &candidate_edges).is_ok()
}
