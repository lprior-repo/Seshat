use diagram_models::document::{Edge, EdgeId, NodeId, OrderedFloat};
use im::HashMap;

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
