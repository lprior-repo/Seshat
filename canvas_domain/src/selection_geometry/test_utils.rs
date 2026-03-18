#![allow(dead_code)]
use diagram_models::document::{LockState, Node, NodeKind, NodeStyle, OrderedFloat};

pub fn make_node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
    make_node_with_lock(kind, x, y, w, h, false)
}

pub fn make_node_with_lock(kind: NodeKind, x: f64, y: f64, w: f64, h: f64, locked: bool) -> Node {
    Node {
        kind,
        icon: String::new(),
        label: String::from("n"),
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
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}
