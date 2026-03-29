use diagram_models::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, Node, NodeId, NodeKind,
    OrderedFloat, Revision,
};
use im::HashMap;
use uuid::Uuid;

/// A simple Domain Specific Language (DSL) for building graphs to test layout algorithms.
pub struct GraphBuilder {
    doc: DiagramDocument,
    nodes: Vec<NodeId>,
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            doc: DiagramDocument {
                version: Default::default(),
                revision: Revision::new(0),
                document: DocumentData {
                    nodes: HashMap::new(),
                    edges: HashMap::new(),
                },
                editor_state: EditorState::default(),
            },
            nodes: Vec::new(),
        }
    }

    /// Add a generic node and return its ID
    pub fn add_node(&mut self) -> NodeId {
        let id = NodeId::new(Uuid::new_v4().to_string());
        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Node".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        self.doc.document.nodes.insert(id.clone(), node);
        self.nodes.push(id.clone());
        id
    }

    /// Add a generic node with a specific reference index for easier testing
    pub fn add_node_at(&mut self, idx: usize) -> NodeId {
        while self.nodes.len() <= idx {
            self.add_node();
        }
        self.nodes[idx].clone()
    }

    /// Connect two nodes by their internal indices
    pub fn connect(&mut self, from_idx: usize, to_idx: usize) -> &mut Self {
        let source = self.add_node_at(from_idx);
        let target = self.add_node_at(to_idx);

        let edge_id = EdgeId::new(format!("edge-{from_idx}-{to_idx}"));
        let edge = Edge {
            source,
            target,
            label: String::new(),
            style: diagram_models::document::EdgeStyle::default(),
            arrow_type: diagram_models::document::ArrowType::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            font_size: None,
            source_port: None,
            target_port: None,
        };

        self.doc.document.edges.insert(edge_id, edge);
        self
    }

    pub fn build(&self) -> DiagramDocument {
        self.doc.clone()
    }

    pub fn get_node(&self, idx: usize) -> &Node {
        let id = &self.nodes[idx];
        self.doc.document.nodes.get(id).unwrap()
    }

    pub fn get_node_id(&self, idx: usize) -> NodeId {
        self.nodes[idx].clone()
    }
}
