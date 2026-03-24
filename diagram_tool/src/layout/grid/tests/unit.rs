//! Grid layout tests.

use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;

use crate::layout::grid::algorithm::calculate_grid_layout;

fn node(x: f64, y: f64, locked: bool, parent: Option<NodeId>) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(100.0),
        height: OrderedFloat(60.0),
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
        z_index: 0,
        style: Some(NodeStyle::default()),
        collapsed: None,
    }
}

#[cfg(kani)]
#[kani::proof]
fn given_nested_children_when_grid_layout_moves_root_then_descendants_follow() {
    let root = NodeId::new(String::from("root"));
    let child = NodeId::new(String::from("child"));
    let grandchild = NodeId::new(String::from("grandchild"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(root.clone(), node(40.0, 40.0, false, None))
        .update(child.clone(), node(50.0, 50.0, true, Some(root.clone())))
        .update(
            grandchild.clone(),
            node(60.0, 60.0, true, Some(child.clone())),
        );

    let next = calculate_grid_layout(&doc, 100.0);

    let root_before = doc
        .document
        .nodes
        .get(&root)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
    let root_after = next
        .document
        .nodes
        .get(&root)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
    let delta = (root_after.0 - root_before.0, root_after.1 - root_before.1);

    let child_before = doc
        .document
        .nodes
        .get(&child)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
    let child_after = next
        .document
        .nodes
        .get(&child)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
    let grand_before = doc
        .document
        .nodes
        .get(&grandchild)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
    let grand_after = next
        .document
        .nodes
        .get(&grandchild)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

    assert!((child_after.0 - (child_before.0 + delta.0)).abs() < f64::EPSILON);
    assert!((child_after.1 - (child_before.1 + delta.1)).abs() < f64::EPSILON);
    assert!((grand_after.0 - (grand_before.0 + delta.0)).abs() < f64::EPSILON);
    assert!((grand_after.1 - (grand_before.1 + delta.1)).abs() < f64::EPSILON);
}

#[cfg(kani)]
#[kani::proof]
fn given_only_locked_roots_when_grid_layout_calculated_then_document_is_unchanged() {
    let n1 = NodeId::new(String::from("n1"));
    let n2 = NodeId::new(String::from("n2"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(n1, node(0.0, 0.0, true, None))
        .update(n2, node(100.0, 100.0, true, None));

    let next = calculate_grid_layout(&doc, 100.0);
    assert_eq!(next.document.nodes, doc.document.nodes);
}

#[cfg(kani)]
#[kani::proof]
fn given_locked_cells_when_layout_runs_then_unlocked_nodes_avoid_occupied_cells() {
    let locked = NodeId::new(String::from("locked"));
    let free1 = NodeId::new(String::from("free1"));
    let free2 = NodeId::new(String::from("free2"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(locked.clone(), node(10.0, 10.0, true, None))
        .update(free1.clone(), node(10.0, 10.0, false, None))
        .update(free2.clone(), node(20.0, 20.0, false, None));

    let next = calculate_grid_layout(&doc, 100.0);
    let p1 = next
        .document
        .nodes
        .get(&free1)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
    let p2 = next
        .document
        .nodes
        .get(&free2)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

    assert_ne!(p1, (0.0, 0.0));
    assert_ne!(p2, (0.0, 0.0));
    assert_ne!(p1, p2);
    assert!((p1.0 % 100.0).abs() < f64::EPSILON);
    assert!((p1.1 % 100.0).abs() < f64::EPSILON);
    assert!((p2.0 % 100.0).abs() < f64::EPSILON);
    assert!((p2.1 % 100.0).abs() < f64::EPSILON);
}

#[cfg(kani)]
#[kani::proof]
fn given_first_row_occupied_when_layout_runs_then_next_free_node_moves_to_next_row() {
    let l0 = NodeId::new(String::from("l0"));
    let l1 = NodeId::new(String::from("l1"));
    let free = NodeId::new(String::from("free"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(l0, node(0.0, 0.0, true, None))
        .update(l1, node(100.0, 0.0, true, None))
        .update(free.clone(), node(4.0, 4.0, false, None));

    let next = calculate_grid_layout(&doc, 100.0);
    let free_pos = next
        .document
        .nodes
        .get(&free)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

    assert_eq!(free_pos.0, 0.0);
    assert_eq!(free_pos.1, 100.0);
}

#[cfg(kani)]
#[kani::proof]
fn given_four_unlocked_nodes_when_layout_runs_then_positions_are_deterministic_and_unique() {
    let ids = [
        NodeId::new(String::from("n1")),
        NodeId::new(String::from("n2")),
        NodeId::new(String::from("n3")),
        NodeId::new(String::from("n4")),
    ];

    let mut doc = DiagramDocument::default();
    doc.document.nodes = ids.iter().fold(HashMap::new(), |nodes, id| {
        nodes.update(id.clone(), node(5.0, 5.0, false, None))
    });

    let next = calculate_grid_layout(&doc, 100.0);
    let mut positions = ids
        .iter()
        .filter_map(|id| next.document.nodes.get(&id).map(|n| (n.x.0, n.y.0)))
        .collect::<Vec<_>>();
    positions.sort_by(|(ax, ay), (bx, by)| ay.total_cmp(by).then_with(|| ax.total_cmp(bx)));

    assert_eq!(positions.len(), 4);
    assert!(positions.contains(&(0.0, 0.0)));
    assert!(positions.contains(&(100.0, 0.0)));
    assert!(positions.contains(&(0.0, 100.0)));
    assert!(positions.contains(&(100.0, 100.0)));
}

#[cfg(kani)]
#[kani::proof]
fn given_sparse_locked_cells_when_layout_runs_then_scan_order_is_stable() {
    let lock00 = NodeId::new(String::from("lock00"));
    let lock11 = NodeId::new(String::from("lock11"));
    let free_a = NodeId::new(String::from("free_a"));
    let free_b = NodeId::new(String::from("free_b"));
    let free_c = NodeId::new(String::from("free_c"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(lock00, node(0.0, 0.0, true, None))
        .update(lock11, node(100.0, 100.0, true, None))
        .update(free_a.clone(), node(5.0, 5.0, false, None))
        .update(free_b.clone(), node(5.0, 5.0, false, None))
        .update(free_c.clone(), node(5.0, 5.0, false, None));

    let next = calculate_grid_layout(&doc, 100.0);
    let pa = next
        .document
        .nodes
        .get(&free_a)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
    let pb = next
        .document
        .nodes
        .get(&free_b)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
    let pc = next
        .document
        .nodes
        .get(&free_c)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

    assert_eq!(pa, (100.0, 0.0));
    assert_eq!(pb, (0.0, 100.0));
    assert_eq!(pc, (0.0, 200.0));
}

#[cfg(kani)]
#[kani::proof]
fn given_locked_prefix_cells_when_layout_runs_then_it_advances_to_next_open_row() {
    let lock00 = NodeId::new(String::from("lock00"));
    let lock10 = NodeId::new(String::from("lock10"));
    let lock01 = NodeId::new(String::from("lock01"));
    let lock11 = NodeId::new(String::from("lock11"));
    let lock02 = NodeId::new(String::from("lock02"));
    let lock12 = NodeId::new(String::from("lock12"));
    let free = NodeId::new(String::from("free"));

    let mut doc = DiagramDocument::default();
    doc.document.nodes = HashMap::new()
        .update(lock00, node(0.0, 0.0, true, None))
        .update(lock10, node(100.0, 0.0, true, None))
        .update(lock01, node(0.0, 100.0, true, None))
        .update(lock11, node(100.0, 100.0, true, None))
        .update(lock02, node(0.0, 200.0, true, None))
        .update(lock12, node(100.0, 200.0, true, None))
        .update(free.clone(), node(5.0, 5.0, false, None));

    let next = calculate_grid_layout(&doc, 100.0);
    let free_pos = next
        .document
        .nodes
        .get(&free)
        .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

    assert_eq!(free_pos, (0.0, 300.0));
}
