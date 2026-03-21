use crate::layout::dag::DagLayoutSettings;
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, Node, NodeId,
    NodeKind, NodeStyle, OrderedFloat, Revision,
};
use proptest::prelude::*;

pub fn make_node_for_prop(x: f64, y: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(220.0),
        height: OrderedFloat(68.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

pub fn make_edge_for_prop(src: &NodeId, tgt: &NodeId) -> Edge {
    Edge {
        source: src.clone(),
        target: tgt.clone(),
        label: String::new(),
        style: diagram_models::document::EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: im::vector![],
        tags: im::vector![],
        metadata: im::HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

pub fn make_doc_for_prop(
    nodes: Vec<(NodeId, Node)>,
    edges: Vec<(EdgeId, Edge)>,
) -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: nodes.into_iter().collect(),
            edges: edges.into_iter().collect(),
        },
        editor_state: EditorState::default(),
    }
}

prop_compose! {
    pub fn arb_dag_layout_settings()(
        layer_spacing in 1.0..1000.0f64,
        node_spacing in 1.0..500.0f64,
    ) -> DagLayoutSettings {
        DagLayoutSettings { layer_spacing, node_spacing }
    }
}
