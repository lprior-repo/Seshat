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
    use super::{ArrowType, Edge};

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
}
