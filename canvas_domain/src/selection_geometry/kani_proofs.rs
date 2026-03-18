use super::core::{selected_node_ids, selection_bounds};
use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};

fn make_any_node(locked: bool) -> Node {
    let x: f64 = kani::any();
    let y: f64 = kani::any();
    let w: f64 = kani::any();
    let h: f64 = kani::any();

    kani::assume(x.is_finite());
    kani::assume(y.is_finite());
    kani::assume(w.is_finite());
    kani::assume(h.is_finite());
    kani::assume(w >= 0.0);
    kani::assume(h >= 0.0);
    kani::assume(x > -1e10 && x < 1e10);
    kani::assume(y > -1e10 && y < 1e10);
    kani::assume(w < 1e10);
    kani::assume(h < 1e10);

    Node {
        kind: NodeKind::Node,
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

#[kani::proof]
fn proof_selection_bounds_envelop_all_selected_unlocked_nodes() {
    let mut doc = DiagramDocument::default();

    let id_a = NodeId::new(String::from("a"));
    let id_b = NodeId::new(String::from("b"));

    let locked_a: bool = kani::any();
    let locked_b: bool = kani::any();

    let node_a = make_any_node(locked_a);
    let node_b = make_any_node(locked_b);

    doc.document.nodes = doc.document.nodes.update(id_a.clone(), node_a.clone());
    doc.document.nodes = doc.document.nodes.update(id_b.clone(), node_b.clone());

    let _ = doc.editor_state.selected_items.insert(id_a.to_string());
    let _ = doc.editor_state.selected_items.insert(id_b.to_string());

    if let Some((min_x, min_y, width, height)) = selection_bounds(&doc) {
        let max_x = min_x + width;
        let max_y = min_y + height;

        if !locked_a {
            assert!(node_a.x.0 >= min_x);
            assert!(node_a.y.0 >= min_y);
            assert!(node_a.x.0 + node_a.width.0 <= max_x);
            assert!(node_a.y.0 + node_a.height.0 <= max_y);
        }
        if !locked_b {
            assert!(node_b.x.0 >= min_x);
            assert!(node_b.y.0 >= min_y);
            assert!(node_b.x.0 + node_b.width.0 <= max_x);
            assert!(node_b.y.0 + node_b.height.0 <= max_y);
        }
    } else {
        assert!(locked_a && locked_b);
    }
}

#[kani::proof]
fn proof_selected_node_ids_filters_locked_nodes() {
    let mut doc = DiagramDocument::default();

    let id_a = NodeId::new(String::from("a"));
    let locked_a: bool = kani::any();
    let node_a = make_any_node(locked_a);

    doc.document.nodes = doc.document.nodes.update(id_a.clone(), node_a.clone());
    let _ = doc.editor_state.selected_items.insert(id_a.to_string());

    let selected_ids = selected_node_ids(&doc);
    if locked_a {
        assert!(selected_ids.is_empty());
    } else {
        assert_eq!(selected_ids.len(), 1);
        assert_eq!(selected_ids[0], id_a);
    }
}
