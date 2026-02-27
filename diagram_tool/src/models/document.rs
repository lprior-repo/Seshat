#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    pub const fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(default)]
    pub z_index: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<NodeStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,
}

/// Helper to make floats Eq
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Default)]
pub struct OrderedFloat(pub f64);

impl Eq for OrderedFloat {}

impl fmt::Display for OrderedFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for OrderedFloat {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for OrderedFloat {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<f64> for OrderedFloat {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Mul<f64> for OrderedFloat {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<f64> for OrderedFloat {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Node,
    Subgraph,
    Text,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeStyle {
    #[default]
    Box,
    Cloud,
    Cylinder,
    Dashed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    pub bend_points: Vec<Point>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(default, rename = "fontSize")]
    pub font_size: Option<OrderedFloat>,
}

const fn default_label_offset() -> OrderedFloat {
    OrderedFloat(0.5)
}

const fn default_thickness() -> OrderedFloat {
    OrderedFloat(1.5)
}

const fn default_directed() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EdgeStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    #[serde(default = "default_grid_size")]
    pub grid_size: OrderedFloat,
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
            camera_x: OrderedFloat(0.0),
            camera_y: OrderedFloat(0.0),
            zoom: OrderedFloat(1.0),
            grid_size: default_grid_size(),
            snap_to_grid: true,
            selected_items: im::HashSet::new(),
            editing_edge_id: None,
            theme: default_theme(),
            show_grid: default_show_grid(),
            minimap_visible: false,
        }
    }
}

const fn default_grid_size() -> OrderedFloat {
    OrderedFloat(20.0)
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
