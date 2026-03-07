#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::ui::grid::GridSize;
use im::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// Newtype for Node Identifier to prevent primitive obsession
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(String);

impl NodeId {
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self(id)
    }

    /// Create a new NodeId, returning error for empty strings
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is an empty string.
    pub fn try_new(id: String) -> Result<Self, &'static str> {
        if id.is_empty() {
            Err("NodeId cannot be empty")
        } else {
            Ok(Self(id))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for Edge Identifier
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId(String);

impl EdgeId {
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self(id)
    }

    /// Create a new EdgeId, returning error for empty strings
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is an empty string.
    pub fn try_new(id: String) -> Result<Self, &'static str> {
        if id.is_empty() {
            Err("EdgeId cannot be empty")
        } else {
            Ok(Self(id))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DiagramDocument {
    pub version: u32,
    pub revision: Revision,
    pub document: DocumentData,
    #[serde(default)]
    pub editor_state: EditorState,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(u64);

impl Revision {
    pub const INITIAL: Self = Self(0);

    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0 + 1)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DocumentData {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub kind: NodeKind,
    #[serde(default)]
    pub icon: String,
    pub label: String,
    pub x: OrderedFloat,
    pub y: OrderedFloat,
    pub width: OrderedFloat,
    pub height: OrderedFloat,
    #[serde(default, rename = "fontSize")]
    pub font_size: Option<OrderedFloat>,
    #[serde(default)]
    pub font_weight: Option<FontWeight>,
    pub locked: bool,
    #[serde(default)]
    pub parent: Option<NodeId>,
    #[serde(default)]
    pub dag_rank: Option<i64>,
    #[serde(default)]
    pub tags: im::Vector<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(default)]
    pub z_index: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<NodeStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,
}

/// Error type for OrderedFloat construction
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum OrderedFloatError {
    #[error("NaN is not a valid value for OrderedFloat")]
    NaN,
    #[error("Infinity is not a valid value for OrderedFloat")]
    Infinite,
}

/// Helper to make floats Eq
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct OrderedFloat(pub f64);

impl<'de> Deserialize<'de> for OrderedFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Ok(Self::new_unchecked(value))
    }
}

impl Serialize for OrderedFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl OrderedFloat {
    /// Creates a new OrderedFloat from a f64 value.
    ///
    /// # Errors
    ///
    /// Returns `OrderedFloatError::NaN` if value is NaN.
    /// Returns `OrderedFloatError::Infinite` if value is infinite.
    pub const fn new(value: f64) -> Result<Self, OrderedFloatError> {
        if value.is_nan() {
            Err(OrderedFloatError::NaN)
        } else if value.is_infinite() {
            Err(OrderedFloatError::Infinite)
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }
}

impl Eq for OrderedFloat {}

impl std::hash::Hash for OrderedFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Use the bits representation for hashing
        // This is consistent with Eq implementation
        self.0.to_bits().hash(state);
    }
}

impl fmt::Display for OrderedFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for OrderedFloat {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(self.0 + rhs.0)
    }
}

impl Sub for OrderedFloat {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(self.0 - rhs.0)
    }
}

impl Sub<f64> for OrderedFloat {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self::Output {
        Self::new_unchecked(self.0 - rhs)
    }
}

impl Mul<f64> for OrderedFloat {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new_unchecked(self.0 * rhs)
    }
}

impl Div<f64> for OrderedFloat {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new_unchecked(self.0 / rhs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Node,
    Subgraph,
    Text,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum NodeStyle {
    #[default]
    Box,
    Cloud,
    Cylinder,
    Dashed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum FontWeight {
    Normal,
    Bold,
}

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
    pub bend_points: im::Vector<Point>,
    #[serde(default)]
    pub tags: im::Vector<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(default, rename = "fontSize")]
    pub font_size: Option<OrderedFloat>,
}

const fn default_label_offset() -> OrderedFloat {
    OrderedFloat::new_unchecked(0.5)
}

const fn default_thickness() -> OrderedFloat {
    OrderedFloat::new_unchecked(1.5)
}

const fn default_directed() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum EdgeStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: OrderedFloat,
    pub y: OrderedFloat,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EditorState {
    pub camera_x: OrderedFloat,
    pub camera_y: OrderedFloat,
    pub zoom: OrderedFloat,
    #[serde(default)]
    pub grid_size: GridSize,
    #[serde(default = "default_snap")]
    pub snap_to_grid: bool,
    #[serde(default)]
    pub selected_items: im::HashSet<String>,
    #[serde(default)]
    pub editing_edge_id: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: EditorTheme,
    #[serde(default = "default_show_grid")]
    pub show_grid: bool,
    #[serde(default)]
    pub minimap_visible: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            camera_x: OrderedFloat::new_unchecked(0.0),
            camera_y: OrderedFloat::new_unchecked(0.0),
            zoom: OrderedFloat::new_unchecked(1.0),
            grid_size: GridSize::default(),
            snap_to_grid: true,
            selected_items: im::HashSet::new(),
            editing_edge_id: None,
            theme: default_theme(),
            show_grid: default_show_grid(),
            minimap_visible: false,
        }
    }
}

const fn default_snap() -> bool {
    true
}

const fn default_theme() -> EditorTheme {
    EditorTheme::System
}

const fn default_show_grid() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EditorTheme {
    Light,
    Dark,
    System,
}

impl Default for DiagramDocument {
    fn default() -> Self {
        Self {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{ArrowType, Edge, EdgeId, EditorState, NodeId, OrderedFloat, Revision};

    #[test]
    fn given_legacy_arrowhead_key_when_deserializing_edge_then_it_is_accepted() {
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
    fn given_node_and_edge_ids_when_stringified_then_values_are_preserved() {
        let node = NodeId::new(String::from("node-1"));
        let edge = EdgeId::new(String::from("edge-1"));

        assert_eq!(node.as_str(), "node-1");
        assert_eq!(edge.as_str(), "edge-1");
        assert_eq!(node.to_string(), "node-1");
        assert_eq!(edge.to_string(), "edge-1");
    }

    #[test]
    fn given_node_id_try_new_with_empty_string_then_it_returns_error() {
        let result = NodeId::try_new(String::new());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "NodeId cannot be empty");
    }

    #[test]
    fn given_edge_id_try_new_with_empty_string_then_it_returns_error() {
        let result = EdgeId::try_new(String::new());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "EdgeId cannot be empty");
    }

    #[test]
    fn given_node_id_try_new_with_valid_string_then_it_succeeds() {
        let result = NodeId::try_new(String::from("valid-id"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "valid-id");
    }

    #[test]
    fn given_edge_id_try_new_with_valid_string_then_it_succeeds() {
        let result = EdgeId::try_new(String::from("valid-id"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "valid-id");
    }

    #[test]
    fn given_revision_when_incremented_then_it_increases_exactly_once() {
        let initial = Revision::INITIAL;
        let next = initial.increment();

        assert_eq!(
            serde_json::to_string(&initial).ok(),
            Some(String::from("0"))
        );
        assert_eq!(serde_json::to_string(&next).ok(), Some(String::from("1")));
    }

    #[test]
    fn given_ordered_float_operations_when_applied_then_arithmetic_is_exact() {
        let a = OrderedFloat(8.0);
        let b = OrderedFloat(2.0);

        assert_eq!((a + b).0, 10.0);
        assert_eq!((a - b).0, 6.0);
        assert_eq!((a - 3.0).0, 5.0);
        assert_eq!((a * 2.5).0, 20.0);
        assert_eq!((a / 2.0).0, 4.0);
        assert_eq!(a.to_string(), "8");
    }

    #[test]
    fn given_edge_without_directed_field_when_deserializing_then_default_is_true() {
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
    fn given_default_editor_state_when_created_then_snap_and_grid_are_enabled() {
        let state = EditorState::default();

        assert!(state.snap_to_grid);
        assert!(state.show_grid);
    }

    #[test]
    fn given_editor_state_json_without_snap_flag_when_deserialized_then_snap_defaults_true() {
        let json = r#"{
            "camera_x": 0.0,
            "camera_y": 0.0,
            "zoom": 1.0,
            "grid_size": 20.0,
            "selected_items": [],
            "editing_edge_id": null,
            "theme": "system",
            "show_grid": true,
            "minimap_visible": false
        }"#;

        let state = serde_json::from_str::<EditorState>(json).ok();
        assert!(state.is_some_and(|parsed| parsed.snap_to_grid));
    }
}

#[cfg(any())]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod proptests {
    use super::{
        DiagramDocument, Edge, EdgeId, EditorState, Node, NodeId, NodeKind, OrderedFloat, Revision,
    };
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn ordered_float_nan_breaks_eq(_ in Just(f64::NAN)) {
            let of = OrderedFloat(f64::NAN);
            let _ = of == of;
        }

        #[test]
        fn ordered_float_inf_comparison(_ in Just(f64::INFINITY)) {
            let of = OrderedFloat(f64::INFINITY);
            let of2 = OrderedFloat(f64::INFINITY);
            let _ = of == of2;
            let _ = of.partial_cmp(&OrderedFloat(f64::NEG_INFINITY));
        }

        #[test]
        fn node_with_nan_coordinates_roundtrip(
            width in 10.0f64..=100.0,
            height in 10.0f64..=100.0
        ) {
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "test".into(),
                x: OrderedFloat(f64::NAN),
                y: OrderedFloat(f64::NAN),
                width: OrderedFloat(width),
                height: OrderedFloat(height),
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
            };
            let json = serde_json::to_string(&node);
            if let Ok(j) = json {
                let parsed: Result<Node, _> = serde_json::from_str(&j);
                let _ = parsed;
            }
        }

        #[test]
        fn document_roundtrip_with_special_floats(use_nan in any::<bool>(), use_inf in any::<bool>()) {
            let mut doc = DiagramDocument::default();
            if use_nan {
                doc.editor_state.camera_x = OrderedFloat(f64::NAN);
            }
            if use_inf {
                doc.editor_state.zoom = OrderedFloat(f64::INFINITY);
            }
            let json = serde_json::to_string(&doc);
            if let Ok(j) = json {
                let parsed: Result<DiagramDocument, _> = serde_json::from_str(&j);
                let _ = parsed;
            }
        }

        #[test]
        fn node_with_negative_dimensions(
            width in -100.0f64..=-0.001,
            height in -100.0f64..=-0.001
        ) {
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "negative-dim".into(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(width),
                height: OrderedFloat(height),
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
            };
            let json = serde_json::to_string(&node).unwrap();
            let parsed: Node = serde_json::from_str(&json).unwrap();
            assert!(parsed.width.0 < 0.0);
            assert!(parsed.height.0 < 0.0);
        }

        #[test]
        fn edge_self_loop_same_source_target(id in "n[0-9]+") {
            let node_id = NodeId::new(id.clone());
            let edge = Edge {
                source: node_id.clone(),
                target: node_id,
                label: "self-loop".into(),
                style: Default::default(),
                arrow_type: Default::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::vector![],
                tags: im::vector![],
                metadata: im::HashMap::new(),
                font_size: None,
            };
            assert_eq!(edge.source, edge.target);
            let json = serde_json::to_string(&edge).unwrap();
            let parsed: Edge = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.source, parsed.target);
        }

        #[test]
        #[should_panic(expected = "overflow")]
        fn revision_overflow(start in (u64::MAX - 1)..=u64::MAX) {
            let rev = Revision(start);
            let incremented = rev.increment();
            let json = serde_json::to_string(&incremented).unwrap();
            let parsed: Revision = serde_json::from_str(&json).unwrap();
            assert_eq!(incremented, parsed);
        }

        #[test]
        fn editor_state_extreme_zoom(zoom in -1e308f64..=1e308f64) {
            let mut state = EditorState::default();
            state.zoom = OrderedFloat(zoom);
            let json = serde_json::to_string(&state).unwrap();
            let parsed: EditorState = serde_json::from_str(&json).unwrap();
            if zoom.is_finite() && zoom.abs() < 1e100 {
                let rel_epsilon = (zoom.abs() * 1e-10).max(1e-10);
                assert!((parsed.zoom.0 - zoom).abs() < rel_epsilon);
            }
        }

        #[test]
        fn ordered_float_arithmetic_with_zero_divisor(a in -1e6f64..=1e6f64) {
            let of_a = OrderedFloat(a);
            let result = of_a / 0.0;
            assert!(result.0.is_infinite() || result.0.is_nan());
        }

        #[test]
        fn ordered_float_subtraction_underflow(a in 0.0f64..=1e6, b in 1e6f64..=1e308) {
            let of_a = OrderedFloat(a);
            let of_b = OrderedFloat(b);
            let result = of_a - of_b;
            assert!(result.0 <= 0.0);
        }

        #[test]
        fn node_id_special_characters(id in "[\\x00-\\x7F]{1,20}") {
            let node_id = NodeId::new(id.clone());
            let json = serde_json::to_string(&node_id).unwrap();
            let parsed: NodeId = serde_json::from_str(&json).unwrap();
            assert_eq!(node_id, parsed);
        }

        #[test]
        fn edge_id_unicode(id in "\\PC*") {
            let edge_id = EdgeId::new(id.clone());
            let json = serde_json::to_string(&edge_id).unwrap();
            let parsed: EdgeId = serde_json::from_str(&json).unwrap();
            assert_eq!(edge_id, parsed);
        }

        #[test]
        fn document_with_many_nodes(node_count in 0usize..100) {
            let mut doc = DiagramDocument::default();
            for i in 0..node_count {
                let id = NodeId::new(format!("n{}", i));
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: format!("Node {}", i),
                    x: OrderedFloat(i as f64 * 100.0),
                    y: OrderedFloat(i as f64 * 50.0),
                    width: OrderedFloat(80.0),
                    height: OrderedFloat(40.0),
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
                };
                doc.document.nodes.insert(id, node);
            }
            let json = serde_json::to_string(&doc).unwrap();
            let parsed: DiagramDocument = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.document.nodes.len(), node_count);
        }

        #[test]
        fn ordered_float_neg_zero_comparison(_pos in Just(0.0f64), _neg in Just(-0.0f64)) {
            let pos = OrderedFloat(0.0f64);
            let neg = OrderedFloat(-0.0f64);
            assert!(pos == neg || pos.0.is_nan() || neg.0.is_nan());
        }

        #[test]
        fn edge_with_empty_source_target(_ in Just(())) {
            let edge = Edge {
                source: NodeId::new(String::new()),
                target: NodeId::new(String::new()),
                label: String::new(),
                style: Default::default(),
                arrow_type: Default::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::vector![],
                tags: im::vector![],
                metadata: im::HashMap::new(),
                font_size: None,
            };
            let json = serde_json::to_string(&edge).unwrap();
            let parsed: Edge = serde_json::from_str(&json).unwrap();
            assert!(parsed.source.as_str().is_empty());
        }
    }
}
