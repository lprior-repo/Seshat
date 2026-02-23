#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub, Mul, Div};
use im::HashMap;

/// Newtype for Node Identifier to prevent primitive obsession
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(String);

impl NodeId {
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self(id)
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
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagramDocument {
    pub revision: Revision,
    pub document: DocumentData,
    pub editor_state: EditorState,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(u64);

impl Revision {
    pub const INITIAL: Self = Self(1);
    
    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentData {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    #[serde(default)]
    pub icon: String,
    pub label: String,
    pub x: OrderedFloat,
    pub y: OrderedFloat,
    pub width: OrderedFloat,
    pub height: OrderedFloat,
    pub locked: bool,
    pub parent: Option<NodeId>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub style: NodeStyle,
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
    fn add(self, rhs: Self) -> Self::Output { Self(self.0 + rhs.0) }
}

impl Sub for OrderedFloat {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output { Self(self.0 - rhs.0) }
}

impl Sub<f64> for OrderedFloat {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self::Output { Self(self.0 - rhs) }
}

impl Mul<f64> for OrderedFloat {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output { Self(self.0 * rhs) }
}

impl Div<f64> for OrderedFloat {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output { Self(self.0 / rhs) }
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
    Default,
    Box, 
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub style: EdgeStyle,
    #[serde(default = "default_directed")]
    pub directed: bool,
    #[serde(default)]
    pub bend_points: Vec<Point>,
}

const fn default_directed() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EdgeStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Point {
    pub x: OrderedFloat,
    pub y: OrderedFloat,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorState {
    pub camera_x: OrderedFloat,
    pub camera_y: OrderedFloat,
    pub zoom: OrderedFloat,
    #[serde(default)]
    pub grid_size: OrderedFloat,
    #[serde(default = "default_snap")]
    pub snap_to_grid: bool,
    #[serde(skip)]
    pub selected_items: im::HashSet<String>, 
}

const fn default_snap() -> bool {
    true
}

impl Default for DiagramDocument {
    fn default() -> Self {
        Self {
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState {
                camera_x: OrderedFloat(0.0),
                camera_y: OrderedFloat(0.0),
                zoom: OrderedFloat(1.0),
                grid_size: OrderedFloat(20.0),
                snap_to_grid: true,
                selected_items: im::HashSet::new(),
            },
        }
    }
}
