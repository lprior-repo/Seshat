#![allow(clippy::unwrap_used, clippy::panic, clippy::module_inception, clippy::let_unit_value, clippy::redundant_pattern_matching, unused_variables, unused_imports)]
use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;

pub fn make_subgraph_node(
    id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    locked: bool,
    collapsed: Option<bool>,
    parent: Option<NodeId>,
) -> (NodeId, Node) {
    let node_id = NodeId::new(id.to_string());
    let node = Node {
        kind: NodeKind::Subgraph,
        icon: String::new(),
        label: String::from("Container"),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(width),
        height: OrderedFloat(height),
        font_size: None,
        font_weight: None,
        lock_state: if locked {
            LockState::Locked
        } else {
            LockState::Unlocked
        },
        parent,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: -1,
        style: Some(NodeStyle::Box),
        collapsed,
    };
    (node_id, node)
}

pub fn make_child_node(
    id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    locked: bool,
    parent: Option<NodeId>,
) -> (NodeId, Node) {
    let node_id = NodeId::new(id.to_string());
    let node = Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::from("Child"),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(width),
        height: OrderedFloat(height),
        font_size: None,
        font_weight: None,
        lock_state: if locked {
            LockState::Locked
        } else {
            LockState::Unlocked
        },
        parent,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 1000,
        style: Some(NodeStyle::default()),
        collapsed: None,
    };
    (node_id, node)
}
