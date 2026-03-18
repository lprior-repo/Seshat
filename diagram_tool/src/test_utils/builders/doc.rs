use crate::test_utils::builders::edge::test_edge;
use crate::test_utils::builders::node::{test_node, test_node_builder};
use diagram_models::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, EditorState, Node, NodeId, Revision,
};
use im::HashMap;

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
    selected_items: im::HashSet<String>,
}

impl DocBuilder {
    /// Create a new document builder with empty state.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            selected_items: im::HashSet::new(),
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
        &self,
        id: impl Into<String>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        let mut this = self.clone();
        let id = NodeId::new(id.into());
        this.nodes.insert(id, test_node(x, y, width, height));
        this
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
