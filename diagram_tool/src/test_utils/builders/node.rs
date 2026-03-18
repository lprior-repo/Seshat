use diagram_models::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};
use im::HashMap;

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
    metadata: im::HashMap<String, serde_json::Value>,
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
