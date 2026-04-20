//! Edge-related domain types for diagram documents.
//!
//! Contains Edge, `EdgeStyle`, `ArrowType`, `SerializedPoint`, and related types.

use super::types::{NodeId, OrderedFloat};
use crate::geometry::Point as CanonicalPoint;
use im::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Edge styling options
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum EdgeStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// Arrowhead type for edges
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum ArrowType {
    #[default]
    Default,
    Sharp,
    Curved,
    Step,
    Straight,
}

/// A point in 2D space for serialization boundary only.
/// Uses `OrderedFloat` for serialization compatibility.
/// Internal code should use canonical Point from geometry module.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SerializedPoint {
    pub x: OrderedFloat,
    pub y: OrderedFloat,
}

impl From<CanonicalPoint> for SerializedPoint {
    fn from(p: CanonicalPoint) -> Self {
        Self {
            x: OrderedFloat(p.x),
            y: OrderedFloat(p.y),
        }
    }
}

impl From<SerializedPoint> for CanonicalPoint {
    fn from(sp: SerializedPoint) -> Self {
        Self::new(sp.x.0, sp.y.0)
    }
}

/// Deprecated: Use `SerializedPoint` for serialization boundary.
/// For internal geometric operations, use `crate::geometry::Point`.
#[deprecated(
    since = "1.0.0",
    note = "Use SerializedPoint for serialization or Point from geometry module"
)]
pub type Point = SerializedPoint;

/// Default label offset (0.5 = middle of edge)
const fn default_label_offset() -> OrderedFloat {
    OrderedFloat::new_unchecked(0.5)
}

/// Default edge thickness
const fn default_thickness() -> OrderedFloat {
    OrderedFloat::new_unchecked(1.5)
}

/// Default directed value
const fn default_directed() -> bool {
    true
}

/// An edge connecting two nodes in the diagram
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub style: EdgeStyle,
    #[serde(
        default,
        rename = "arrowType",
        alias = "arrowhead",
        alias = "arrow_type"
    )]
    pub arrow_type: ArrowType,
    #[serde(default = "default_label_offset")]
    pub label_offset_t: OrderedFloat,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default = "default_thickness")]
    pub thickness: OrderedFloat,
    #[serde(default = "default_directed")]
    pub directed: bool,
    #[serde(default)]
    pub bend_points: im::Vector<SerializedPoint>,
    #[serde(default)]
    pub tags: im::Vector<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(default, rename = "fontSize", alias = "font_size")]
    pub font_size: Option<OrderedFloat>,
    #[serde(default)]
    pub source_port: Option<crate::port::PortAnchor>,
    #[serde(default)]
    pub target_port: Option<crate::port::PortAnchor>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::super::types::{NodeId, OrderedFloat};
    use super::{ArrowType, Edge, EdgeStyle, SerializedPoint};

    fn create_test_edge(source: &str, target: &str) -> Edge {
        Edge {
            source: NodeId::new(source.to_string()),
            target: NodeId::new(target.to_string()),
            label: String::new(),
            style: EdgeStyle::default(),
            arrow_type: ArrowType::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    #[test]
    fn edge_serialization_roundtrip() {
        let edge = create_test_edge("n1", "n2");
        let json = serde_json::to_string(&edge).unwrap();
        let parsed: Edge = serde_json::from_str(&json).unwrap();
        assert_eq!(edge, parsed);
    }

    #[test]
    fn edge_with_bend_points_roundtrips() {
        let mut edge = create_test_edge("n1", "n2");
        edge.bend_points = im::vector![
            SerializedPoint {
                x: OrderedFloat(10.0),
                y: OrderedFloat(20.0)
            },
            SerializedPoint {
                x: OrderedFloat(30.0),
                y: OrderedFloat(40.0)
            }
        ];
        let json = serde_json::to_string(&edge).unwrap();
        let parsed: Edge = serde_json::from_str(&json).unwrap();
        assert_eq!(edge.bend_points.len(), parsed.bend_points.len());
    }

    #[test]
    fn default_directed_is_true() {
        let edge = create_test_edge("n1", "n2");
        assert!(edge.directed);
    }

    #[test]
    fn legacy_arrowhead_key_is_accepted() {
        let json = r#"{
            "source": "n1",
            "target": "n2",
            "label": "",
            "style": "solid",
            "arrowhead": "default",
            "label_offset_t": 0.5,
            "thickness": 1.5,
            "directed": true,
            "bend_points": [],
            "tags": [],
            "metadata": {}
        }"#;

        let parsed = serde_json::from_str::<Edge>(json);
        assert!(parsed.is_ok(), "{:?}", parsed.err());
        assert_eq!(
            parsed.ok().map(|edge| edge.arrow_type),
            Some(ArrowType::Default)
        );
    }

    #[test]
    fn edge_without_directed_field_defaults_to_true() {
        let json = r#"{
            "source": "n1",
            "target": "n2",
            "label": "",
            "style": "solid",
            "arrowType": "default",
            "label_offset_t": 0.5,
            "thickness": 1.5,
            "bend_points": [],
            "tags": [],
            "metadata": {}
        }"#;

        let parsed = serde_json::from_str::<Edge>(json).ok();
        assert!(parsed.is_some());
        assert!(parsed.is_some_and(|edge| edge.directed));
    }

    #[test]
    fn allows_self_loop_edge() {
        let edge = create_test_edge("N1", "N1");
        assert!(edge.source == edge.target);
    }

    #[test]
    fn allows_multiple_edges_between_same_nodes() {
        let edge1 = create_test_edge("N1", "N2");
        let edge2 = create_test_edge("N1", "N2");
        assert!(edge1.source == edge2.source);
        assert!(edge1.target == edge2.target);
    }
}
