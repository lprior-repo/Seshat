#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
#[cfg(test)]
mod tests {
    use crate::document::{
        Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat, SerializedPoint,
    };
    use crate::projection::policy_hash::projection_hash;
    use crate::projection::types::{DiagramProjection, ReplayError};
    use im::{HashMap, Vector};
    use serde_json::Value;

    fn create_test_node(id: &str, x: f64, y: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn create_test_edge(source: &str, target: &str) -> Edge {
        Edge {
            source: NodeId::new(source.to_string()),
            target: NodeId::new(target.to_string()),
            label: String::new(),
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat(0.5),
            directed: true,
            bend_points: Vector::new(),
            tags: Vector::new(),
            metadata: HashMap::new(),
            color: None,
            thickness: OrderedFloat(1.0),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    #[test]
    fn given_identical_projections_when_hashed_then_hashes_match() {
        let mut proj1 = DiagramProjection::default();
        let mut proj2 = DiagramProjection::default();

        let node = create_test_node("n1", 0.0, 0.0);
        proj1
            .nodes
            .insert(NodeId::new("n1".to_string()), node.clone());
        proj2.nodes.insert(NodeId::new("n1".to_string()), node);

        let hash1 = projection_hash(&proj1).unwrap();
        let hash2 = projection_hash(&proj2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn given_projections_with_different_node_order_when_hashed_then_hashes_match_due_to_sorting() {
        let mut proj1 = DiagramProjection::default();
        let mut proj2 = DiagramProjection::default();

        let n1 = create_test_node("n1", 0.0, 0.0);
        let n2 = create_test_node("n2", 10.0, 10.0);

        proj1
            .nodes
            .insert(NodeId::new("n1".to_string()), n1.clone());
        proj1
            .nodes
            .insert(NodeId::new("n2".to_string()), n2.clone());

        proj2.nodes.insert(NodeId::new("n2".to_string()), n2);
        proj2.nodes.insert(NodeId::new("n1".to_string()), n1);

        let hash1 = projection_hash(&proj1).unwrap();
        let hash2 = projection_hash(&proj2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn given_projection_with_nan_node_coordinates_when_hashed_then_returns_invariant_violation() {
        let mut proj = DiagramProjection::default();
        let n1 = create_test_node("n1", f64::NAN, 0.0);
        proj.nodes.insert(NodeId::new("n1".to_string()), n1);

        let result = projection_hash(&proj);
        assert!(matches!(result, Err(ReplayError::InvariantViolation(_))));
    }

    #[test]
    fn given_projection_with_nan_edge_thickness_when_hashed_then_returns_invariant_violation() {
        let mut proj = DiagramProjection::default();
        let mut e1 = create_test_edge("n1", "n2");
        e1.thickness = OrderedFloat(f64::NAN);
        proj.edges.insert(EdgeId::new("e1".to_string()), e1);

        let result = projection_hash(&proj);
        assert!(matches!(result, Err(ReplayError::InvariantViolation(_))));
    }

    #[test]
    fn given_different_node_attributes_when_hashed_then_hashes_differ() {
        let mut proj1 = DiagramProjection::default();
        let mut proj2 = DiagramProjection::default();

        let mut n1 = create_test_node("n1", 0.0, 0.0);
        proj1
            .nodes
            .insert(NodeId::new("n1".to_string()), n1.clone());

        n1.label = "different".to_string();
        proj2.nodes.insert(NodeId::new("n1".to_string()), n1);

        let hash1 = projection_hash(&proj1).unwrap();
        let hash2 = projection_hash(&proj2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn given_different_edge_bend_points_when_hashed_then_hashes_differ() {
        let mut proj1 = DiagramProjection::default();
        let mut proj2 = DiagramProjection::default();

        let mut e1 = create_test_edge("n1", "n2");
        proj1
            .edges
            .insert(EdgeId::new("e1".to_string()), e1.clone());

        let mut bp = Vector::new();
        bp.push_back(SerializedPoint {
            x: OrderedFloat(10.0),
            y: OrderedFloat(10.0),
        });
        e1.bend_points = bp;
        proj2.edges.insert(EdgeId::new("e1".to_string()), e1);

        let hash1 = projection_hash(&proj1).unwrap();
        let hash2 = projection_hash(&proj2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn given_different_metadata_when_hashed_then_hashes_differ() {
        let mut proj1 = DiagramProjection::default();
        let mut proj2 = DiagramProjection::default();

        let mut n1 = create_test_node("n1", 0.0, 0.0);
        n1.metadata.insert("key".to_string(), Value::Bool(true));
        proj1
            .nodes
            .insert(NodeId::new("n1".to_string()), n1.clone());

        let mut n2 = create_test_node("n1", 0.0, 0.0);
        n2.metadata.insert("key".to_string(), Value::Bool(false));
        proj2.nodes.insert(NodeId::new("n1".to_string()), n2);

        let hash1 = projection_hash(&proj1).unwrap();
        let hash2 = projection_hash(&proj2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn given_projection_with_tags_when_hashed_then_tags_are_sorted_and_hashes_match() {
        let mut proj1 = DiagramProjection::default();
        let mut proj2 = DiagramProjection::default();

        let mut n1 = create_test_node("n1", 0.0, 0.0);
        let mut tags1 = Vector::new();
        tags1.push_back("a".to_string());
        tags1.push_back("b".to_string());
        n1.tags = tags1;
        proj1.nodes.insert(NodeId::new("n1".to_string()), n1);

        let mut n2 = create_test_node("n1", 0.0, 0.0);
        let mut tags2 = Vector::new();
        tags2.push_back("b".to_string());
        tags2.push_back("a".to_string());
        n2.tags = tags2;
        proj2.nodes.insert(NodeId::new("n1".to_string()), n2);

        let hash1 = projection_hash(&proj1).unwrap();
        let hash2 = projection_hash(&proj2).unwrap();

        assert_eq!(hash1, hash2);
    }
}
