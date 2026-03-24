#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]

use crate::document::{
    Edge, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;

/// Creates a minimal valid `Node` with the given id.
pub(crate) fn make_node(id: &str) -> (NodeId, Node) {
    (
        NodeId::new(id.to_string()),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(64.0),
            height: OrderedFloat(64.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        },
    )
}

/// Creates a minimal valid `Edge` with the given id, source, and target.
pub(crate) fn make_edge(id: &str, src: &str, tgt: &str) -> (EdgeId, Edge) {
    (
        EdgeId::new(id.to_string()),
        Edge {
            source: NodeId::new(src.to_string()),
            target: NodeId::new(tgt.to_string()),
            label: String::new(),
            style: EdgeStyle::default(),
            arrow_type: Default::default(),
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
        },
    )
}
