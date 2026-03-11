use crate::models::document::{Node, OrderedFloat, Point};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PortError {
    #[error("Invalid port offset: coordinates must be finite and within [0.0, 1.0]")]
    InvalidPortOffset,
    #[error("Node not found")]
    NodeNotFound,
    #[error("Edge not found")]
    EdgeNotFound,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
pub struct NormalizedOffset {
    pub x: OrderedFloat,
    pub y: OrderedFloat,
}

impl NormalizedOffset {
    pub fn new(x: OrderedFloat, y: OrderedFloat) -> Result<Self, PortError> {
        let x_val = x.0;
        let y_val = y.0;

        if !x_val.is_finite() || !y_val.is_finite() {
            return Err(PortError::InvalidPortOffset);
        }
        if !(0.0..=1.0).contains(&x_val) || !(0.0..=1.0).contains(&y_val) {
            return Err(PortError::InvalidPortOffset);
        }

        Ok(Self { x, y })
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum PortAnchor {
    Top,
    Bottom,
    Left,
    Right,
    Center,
    Custom(NormalizedOffset),
}

#[must_use]
pub fn compute_port_absolute_position(node: &Node, port: &PortAnchor) -> Point {
    match port {
        PortAnchor::Top => Point {
            x: node.x + (node.width / 2.0),
            y: node.y,
        },
        PortAnchor::Bottom => Point {
            x: node.x + (node.width / 2.0),
            y: node.y + node.height,
        },
        PortAnchor::Left => Point {
            x: node.x,
            y: node.y + (node.height / 2.0),
        },
        PortAnchor::Right => Point {
            x: node.x + node.width,
            y: node.y + (node.height / 2.0),
        },
        PortAnchor::Center => Point {
            x: node.x + (node.width / 2.0),
            y: node.y + (node.height / 2.0),
        },
        PortAnchor::Custom(offset) => Point {
            x: node.x + (node.width * offset.x.0),
            y: node.y + (node.height * offset.y.0),
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::document::{DiagramDocument, Edge, EdgeId, Node, NodeId, NodeKind};

    fn create_test_node(x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Test".into(),
            x: OrderedFloat::new_unchecked(x),
            y: OrderedFloat::new_unchecked(y),
            width: OrderedFloat::new_unchecked(w),
            height: OrderedFloat::new_unchecked(h),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn create_test_edge(source: &str, target: &str) -> Edge {
        Edge {
            source: NodeId::new(source.into()),
            target: NodeId::new(target.into()),
            label: String::new(),
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat::new_unchecked(0.5),
            color: None,
            thickness: OrderedFloat::new_unchecked(1.5),
            directed: true,
            bend_points: im::vector![],
            tags: im::vector![],
            metadata: im::HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_edge_connects_to_top_port_anchor_successfully() {
        let node = create_test_node(0.0, 0.0, 100.0, 100.0);
        let pt = compute_port_absolute_position(&node, &PortAnchor::Top);
        assert_eq!(pt.x.0, 50.0);
        assert_eq!(pt.y.0, 0.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_edge_connects_to_custom_port_anchor_successfully() {
        let node = create_test_node(10.0, 20.0, 100.0, 100.0);
        let offset = NormalizedOffset::new(
            OrderedFloat::new_unchecked(0.25),
            OrderedFloat::new_unchecked(0.75),
        )
        .unwrap();
        let pt = compute_port_absolute_position(&node, &PortAnchor::Custom(offset));
        assert_eq!(pt.x.0, 35.0);
        assert_eq!(pt.y.0, 95.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_custom_port_offset_is_out_of_bounds() {
        let err = NormalizedOffset::new(
            OrderedFloat::new_unchecked(-0.1),
            OrderedFloat::new_unchecked(0.5),
        )
        .unwrap_err();
        assert_eq!(err, PortError::InvalidPortOffset);

        let err2 = NormalizedOffset::new(
            OrderedFloat::new_unchecked(1.5),
            OrderedFloat::new_unchecked(0.5),
        )
        .unwrap_err();
        assert_eq!(err2, PortError::InvalidPortOffset);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_setting_port_for_nonexistent_edge() {
        let mut doc = DiagramDocument::default();
        let err = doc
            .set_edge_source_port(&EdgeId::new("missing".into()), Some(PortAnchor::Top))
            .unwrap_err();
        assert_eq!(err, PortError::EdgeNotFound);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_node_not_found_for_port_computation() {
        let mut doc = DiagramDocument::default();
        doc.document
            .edges
            .insert(EdgeId::new("e1".into()), create_test_edge("n1", "n2"));
        let err = doc
            .set_edge_source_port(&EdgeId::new("e1".into()), Some(PortAnchor::Top))
            .unwrap_err();
        assert_eq!(err, PortError::NodeNotFound);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_edge_port_anchor_computes_correctly_for_zero_width_node() {
        let node = create_test_node(10.0, 10.0, 0.0, 0.0);
        let pt = compute_port_absolute_position(&node, &PortAnchor::Center);
        assert_eq!(pt.x.0, 10.0);
        assert_eq!(pt.y.0, 10.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_custom_port_anchor_at_exact_boundaries_zero_and_one() {
        let offset = NormalizedOffset::new(
            OrderedFloat::new_unchecked(0.0),
            OrderedFloat::new_unchecked(1.0),
        )
        .unwrap();
        assert_eq!(offset.x.0, 0.0);
        assert_eq!(offset.y.0, 1.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_p2_violation_returns_invalid_port_offset() {
        let res = NormalizedOffset::new(
            OrderedFloat::new_unchecked(1.5),
            OrderedFloat::new_unchecked(0.5),
        );
        assert_eq!(res.unwrap_err(), PortError::InvalidPortOffset);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_p3_violation_returns_node_not_found() {
        let mut doc = DiagramDocument::default();
        doc.document
            .edges
            .insert(EdgeId::new("e1".into()), create_test_edge("n1", "n2"));
        let res = doc.set_edge_source_port(&EdgeId::new("e1".into()), Some(PortAnchor::Top));
        assert_eq!(res.unwrap_err(), PortError::NodeNotFound);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_setting_port_updates_edge_state() {
        let mut doc = DiagramDocument::default();
        doc.document.nodes.insert(
            NodeId::new("n1".into()),
            create_test_node(0.0, 0.0, 10.0, 10.0),
        );
        doc.document.nodes.insert(
            NodeId::new("n2".into()),
            create_test_node(20.0, 20.0, 10.0, 10.0),
        );
        doc.document
            .edges
            .insert(EdgeId::new("e1".into()), create_test_edge("n1", "n2"));

        doc.set_edge_source_port(&EdgeId::new("e1".into()), Some(PortAnchor::Bottom))
            .unwrap();

        let edge = doc.document.edges.get(&EdgeId::new("e1".into())).unwrap();
        assert_eq!(edge.source_port, Some(PortAnchor::Bottom));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_edge_port_anchors_serialize_and_deserialize() {
        let offset = NormalizedOffset::new(
            OrderedFloat::new_unchecked(0.25),
            OrderedFloat::new_unchecked(0.75),
        )
        .unwrap();
        let port = PortAnchor::Custom(offset);

        let mut edge = create_test_edge("n1", "n2");
        edge.source_port = Some(port);
        edge.target_port = Some(PortAnchor::Right);

        let json = serde_json::to_string(&edge).unwrap();
        let deserialized: Edge = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.source_port, Some(port));
        assert_eq!(deserialized.target_port, Some(PortAnchor::Right));
    }
}
