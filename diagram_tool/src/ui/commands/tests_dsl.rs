#[cfg(test)]
#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
pub mod dsl {
    use diagram_models::document::{
        DiagramDocument, Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::{HashMap, Vector};

    pub struct TestDsl {
        doc: DiagramDocument,
    }

    impl TestDsl {
        pub fn new() -> Self {
            Self {
                doc: DiagramDocument::default(),
            }
        }

        pub fn with_node(mut self, id: &str, x: f64, y: f64, w: f64, h: f64, locked: bool) -> Self {
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: id.to_string(),
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                width: OrderedFloat(w),
                height: OrderedFloat(h),
                font_size: None,
                font_weight: None,
                lock_state: if locked {
                    LockState::Locked
                } else {
                    LockState::Unlocked
                },
                parent: None,
                dag_rank: None,
                tags: Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            self.doc
                .document
                .nodes
                .insert(NodeId::new(id.to_string()), node);
            self
        }

        pub fn with_z_index(mut self, id: &str, z: i64) -> Self {
            if let Some(node) = self
                .doc
                .document
                .nodes
                .get_mut(&NodeId::new(id.to_string()))
            {
                node.z_index = z;
            }
            self
        }

        pub fn with_zoom(mut self, zoom: f64, cam_x: f64, cam_y: f64) -> Self {
            self.doc.editor_state.zoom = OrderedFloat(zoom);
            self.doc.editor_state.camera_x = OrderedFloat(cam_x);
            self.doc.editor_state.camera_y = OrderedFloat(cam_y);
            self
        }

        pub fn with_subgraph(mut self, id: &str, parent: Option<&str>) -> Self {
            let node = Node {
                kind: NodeKind::Subgraph,
                icon: String::new(),
                label: id.to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(100.0),
                height: OrderedFloat(100.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: parent.map(|p| NodeId::new(p.to_string())),
                dag_rank: None,
                tags: Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            self.doc
                .document
                .nodes
                .insert(NodeId::new(id.to_string()), node);
            self
        }

        pub fn with_edge(mut self, id: &str, source: &str, target: &str) -> Self {
            let edge = Edge {
                source: NodeId::new(source.to_string()),
                target: NodeId::new(target.to_string()),
                label: id.to_string(),
                style: Default::default(),
                arrow_type: Default::default(),
                label_offset_t: OrderedFloat(0.5),
                directed: true,
                bend_points: Vector::new(),
                tags: Vector::new(),
                metadata: HashMap::new(),
                color: None,
                thickness: OrderedFloat(1.0),
                font_size: None,
                source_port: None,
                target_port: None,
            };
            self.doc
                .document
                .edges
                .insert(EdgeId::new(id.to_string()), edge);
            self
        }

        pub fn with_selection(mut self, ids: &[&str]) -> Self {
            for id in ids {
                self.doc.editor_state.selected_items.insert(id.to_string());
            }
            self
        }

        pub fn build(self) -> DiagramDocument {
            self.doc
        }
    }
}
