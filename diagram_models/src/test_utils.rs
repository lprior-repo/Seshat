//! Test Builder Helpers
//!
//! Consolidated test helpers for creating nodes, edges, and documents in tests.
//! This module provides a fluent builder pattern for test data construction.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![allow(dead_code)]
#![allow(clippy::pedantic)]

use crate::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, Node, NodeId, NodeKind,
    OrderedFloat, Revision,
};
use im::{HashMap, HashSet};

/// Default node for testing with basic Text kind.
#[must_use]
pub fn test_node(x: f64, y: f64, width: f64, height: f64) -> Node {
    test_node_builder(x, y, width, height).build()
}

/// Create a default test node at (0,0) with size (100,100).
#[must_use]
pub fn test_node_default() -> Node {
    test_node(0.0, 0.0, 100.0, 100.0)
}

/// Builder for nodes with flexible configuration.
pub struct NodeBuilder {
    kind: NodeKind,
    label: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    lock_state: LockState,
    parent: Option<NodeId>,
    z_index: i64,
    metadata: HashMap<String, serde_json::Value>,
}

impl NodeBuilder {
    /// Create a new node builder with default values.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            kind: NodeKind::Node,
            label: "Test".to_string(),
            x,
            y,
            width,
            height,
            lock_state: LockState::Unlocked,
            parent: None,
            z_index: 0,
            metadata: HashMap::new(),
        }
    }

    /// Set the node kind.
    pub fn with_kind(mut self, kind: NodeKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set the node label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the lock state.
    pub fn with_lock_state(mut self, lock_state: LockState) -> Self {
        self.lock_state = lock_state;
        self
    }

    /// Set locked state (convenience method).
    pub fn locked(mut self) -> Self {
        self.lock_state = LockState::Locked;
        self
    }

    /// Set the parent node.
    pub fn with_parent(mut self, parent: NodeId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Set z-index.
    pub fn with_z_index(mut self, z_index: i64) -> Self {
        self.z_index = z_index;
        self
    }

    /// Add metadata entry.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Build the node.
    pub fn build(self) -> Node {
        Node {
            kind: self.kind,
            icon: String::new(),
            label: self.label,
            x: OrderedFloat::new_unchecked(self.x),
            y: OrderedFloat::new_unchecked(self.y),
            width: OrderedFloat::new_unchecked(self.width),
            height: OrderedFloat::new_unchecked(self.height),
            font_size: None,
            font_weight: None,
            lock_state: self.lock_state,
            parent: self.parent,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: self.metadata,
            z_index: self.z_index,
            style: None,
            collapsed: None,
        }
    }
}

/// Create a node builder for fluent configuration.
pub fn test_node_builder(x: f64, y: f64, width: f64, height: f64) -> NodeBuilder {
    NodeBuilder::new(x, y, width, height)
}

/// Create a subgraph node for testing.
#[must_use]
pub fn test_subgraph() -> Node {
    NodeBuilder::new(0.0, 0.0, 100.0, 100.0)
        .with_kind(NodeKind::Subgraph)
        .with_label("Group")
        .build()
}

/// Create a subgraph builder for fluent configuration.
#[must_use]
pub fn test_subgraph_builder() -> NodeBuilder {
    NodeBuilder::new(0.0, 0.0, 100.0, 100.0)
        .with_kind(NodeKind::Subgraph)
        .with_label("Group")
}

/// Create a text node for testing.
#[must_use]
pub fn test_text_node(x: f64, y: f64, width: f64, height: f64) -> Node {
    NodeBuilder::new(x, y, width, height)
        .with_kind(NodeKind::Text)
        .build()
}

/// Create a basic edge for testing.
#[must_use]
pub fn test_edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: Default::default(),
        arrow_type: Default::default(),
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.0),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

/// Create an edge builder for fluent configuration.
pub fn test_edge_builder(source: NodeId, target: NodeId) -> EdgeBuilder {
    EdgeBuilder::new(source, target)
}

/// Builder for edges with flexible configuration.
pub struct EdgeBuilder {
    source: NodeId,
    target: NodeId,
    label: String,
    thickness: f64,
    directed: bool,
}

impl EdgeBuilder {
    /// Create a new edge builder.
    pub fn new(source: NodeId, target: NodeId) -> Self {
        Self {
            source,
            target,
            label: String::new(),
            thickness: 1.0,
            directed: true,
        }
    }

    /// Set the edge label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the edge thickness.
    pub fn with_thickness(mut self, thickness: f64) -> Self {
        self.thickness = thickness;
        self
    }

    /// Set directedness.
    pub fn directed(mut self, directed: bool) -> Self {
        self.directed = directed;
        self
    }

    /// Build the edge.
    pub fn build(self) -> Edge {
        Edge {
            source: self.source,
            target: self.target,
            label: self.label,
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat::new_unchecked(self.thickness),
            directed: self.directed,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }
}

/// Create a basic DiagramDocument with some nodes for testing.
#[must_use]
pub fn setup_doc() -> DiagramDocument {
    let mut nodes = HashMap::new();
    nodes.insert(
        NodeId::new("A".to_string()),
        test_node(10.0, 10.0, 50.0, 50.0),
    );
    nodes.insert(
        NodeId::new("B".to_string()),
        test_node(20.0, 20.0, 30.0, 30.0),
    );

    let doc_data = DocumentData {
        nodes,
        edges: HashMap::new(),
    };

    let mut editor_state = EditorState::default();
    editor_state.selected_items.insert("A".to_string());
    editor_state.selected_items.insert("B".to_string());

    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: doc_data,
        editor_state,
    }
}

/// Builder for documents with flexible configuration.
#[derive(Clone)]
pub struct DocBuilder {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
    selected_items: HashSet<String>,
}

impl DocBuilder {
    /// Create a new document builder with empty state.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            selected_items: HashSet::new(),
        }
    }

    /// Add a node to the document.
    pub fn add_node(mut self, id: impl Into<String>, node: Node) -> Self {
        let id = NodeId::new(id.into());
        self.nodes.insert(id, node);
        self
    }

    /// Add a node builder result to the document.
    pub fn add_node_with(
        mut self,
        id: impl Into<String>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        let id = NodeId::new(id.into());
        self.nodes.insert(id, test_node(x, y, width, height));
        self
    }

    /// Add an edge to the document.
    pub fn add_edge(mut self, id: impl Into<String>, edge: Edge) -> Self {
        let id = EdgeId::new(id.into());
        self.edges.insert(id, edge);
        self
    }

    /// Add an edge using source and target strings.
    pub fn add_edge_str(
        mut self,
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        let id = EdgeId::new(id.into());
        let source = NodeId::new(source.into());
        let target = NodeId::new(target.into());
        self.edges.insert(id, test_edge(source, target));
        self
    }

    /// Add an item to the selection.
    pub fn with_selection(mut self, id: impl Into<String>) -> Self {
        self.selected_items.insert(id.into());
        self
    }

    /// Build the document.
    pub fn build(self) -> DiagramDocument {
        let doc_data = DocumentData {
            nodes: self.nodes,
            edges: self.edges,
        };

        let editor_state = EditorState {
            selected_items: self.selected_items,
            ..Default::default()
        };

        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: doc_data,
            editor_state,
        }
    }
}

impl Default for DocBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a document with nodes.
#[must_use]
pub fn doc_with_nodes<const N: usize>(items: [(&str, f64, f64, f64, f64); N]) -> DiagramDocument {
    let mut builder = DocBuilder::new();
    for (id, x, y, w, h) in items {
        builder = builder.add_node_with(id, x, y, w, h);
    }
    builder.build()
}

/// Create a document with specific nodes for marquee testing.
/// N1: (10, 10) 50x50 - enclosed by (0,0)->(100,100)
/// N2: (80, 80) 50x50 - intersects with (0,0)->(100,100)
/// N3: (150, 150) 50x50 - outside (0,0)->(100,100)
/// N4: (10, 10) 50x50 with rotation - slightly outside
#[must_use]
pub fn setup_doc_with_nodes() -> DiagramDocument {
    let mut builder = DocBuilder::new();

    // N1: (10, 10) 50x50. Enclosed by (0,0)->(100,100)
    // N2: (80, 80) 50x50. Intersects with (0,0)->(100,100)
    // N3: (150, 150) 50x50. Outside (0,0)->(100,100)
    // N4: (10, 10) 50x50, but rotated 45 degrees
    builder = builder
        .add_node_with("n1", 10.0, 10.0, 50.0, 50.0)
        .add_node_with("n2", 80.0, 80.0, 50.0, 50.0)
        .add_node_with("n3", 150.0, 150.0, 50.0, 50.0);

    // N4: rotated node - need to use builder for metadata
    let n4 = test_node_builder(10.0, 10.0, 50.0, 50.0)
        .with_label("n4")
        .with_metadata("rotation", serde_json::json!(std::f64::consts::FRAC_PI_4))
        .build();

    let mut doc = builder.build();
    doc.document.nodes.insert(NodeId::new("n4".to_string()), n4);
    doc
}
