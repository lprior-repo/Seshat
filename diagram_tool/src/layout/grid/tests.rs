//! Grid layout tests.

use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;

use super::{accumulated_parent_delta, calculate_grid_layout};

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
#[test]
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
#[test]
fn given_ancestor_chain_deltas_when_accumulated_then_sum_is_exact() {
    let root = NodeId::new(String::from("root"));
    let child = NodeId::new(String::from("child"));
    let grandchild = NodeId::new(String::from("grandchild"));

    let nodes = HashMap::new()
        .update(root.clone(), node(0.0, 0.0, true, None))
        .update(child.clone(), node(0.0, 0.0, true, Some(root.clone())))
        .update(grandchild, node(0.0, 0.0, true, Some(child.clone())));
    let deltas = HashMap::new()
        .update(root, (2.0, 3.0))
        .update(child.clone(), (5.0, 7.0));

    let result = accumulated_parent_delta(&child, &deltas, &nodes);
    assert_eq!(result, Some((7.0, 10.0)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
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
#[test]
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
#[test]
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
#[test]
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
#[test]
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
#[test]
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

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn make_node(x: f64, y: f64, locked: bool, parent: Option<NodeId>) -> Node {
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

    fn make_doc(nodes: Vec<(String, f64, f64, bool)>) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        for (id, x, y, locked) in nodes {
            doc.document.nodes = doc
                .document
                .nodes
                .update(NodeId::new(id), make_node(x, y, locked, None));
        }
        doc
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_never_panics_with_valid_cell_size(cell_size in 1e-10_f64..1e10_f64) {
            let doc = make_doc(vec![
                ("a".into(), 50.0, 50.0, false),
                ("b".into(), 150.0, 75.0, false),
            ]);
            let result = calculate_grid_layout(&doc, cell_size);
            prop_assert!(result.document.nodes.len() == 2);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[should_panic(expected = "cell_size must be positive and finite")]
        fn prop_grid_layout_zero_cell_size(_ in Just(0.0_f64)) {
            let doc = make_doc(vec![("a".into(), 100.0, 100.0, false)]);
            let _result = calculate_grid_layout(&doc, 0.0);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[should_panic(expected = "cell_size must be positive and finite")]
        fn prop_grid_layout_negative_cell_size(cell_size in -1e10_f64..-1e-10_f64) {
            let doc = make_doc(vec![("a".into(), 100.0, 100.0, false)]);
            let _result = calculate_grid_layout(&doc, cell_size);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[should_panic(expected = "cell_size must be positive and finite")]
        fn prop_grid_layout_nan_cell_size(_ in Just(())) {
            let doc = make_doc(vec![("a".into(), 100.0, 100.0, false)]);
            let _result = calculate_grid_layout(&doc, f64::NAN);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        #[should_panic(expected = "cell_size must be positive and finite")]
        fn prop_grid_layout_inf_cell_size(sign in -1_i32..=1) {
            let cell_size = match sign {
                -1 => f64::NEG_INFINITY,
                1 => f64::INFINITY,
                _ => f64::INFINITY,
            };
            let doc = make_doc(vec![("a".into(), 100.0, 100.0, false)]);
            let _result = calculate_grid_layout(&doc, cell_size);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_extreme_node_coordinates(coord in -1e15_f64..1e15_f64) {
            let doc = make_doc(vec![
                ("a".into(), coord, coord, false),
                ("b".into(), -coord, -coord, false),
            ]);
            let result = calculate_grid_layout(&doc, 100.0);
            prop_assert!(result.document.nodes.len() == 2);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_nan_node_positions(_ in Just(())) {
            let mut doc = DiagramDocument::default();
            let mut node = make_node(f64::NAN, f64::NAN, false, None);
            node.x = OrderedFloat(f64::NAN);
            node.y = OrderedFloat(f64::NAN);
            doc.document.nodes = doc.document.nodes.update(NodeId::new("nan".into()), node);
            let result = calculate_grid_layout(&doc, 100.0);
            prop_assert!(result.document.nodes.len() == 1);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_inf_node_positions(sign in -1_i32..=1) {
            let mut doc = DiagramDocument::default();
            let x = if sign < 0 { f64::NEG_INFINITY } else { f64::INFINITY };
            let mut node = make_node(x, x, false, None);
            node.x = OrderedFloat(x);
            node.y = OrderedFloat(x);
            doc.document.nodes = doc.document.nodes.update(NodeId::new("inf".into()), node);
            let result = calculate_grid_layout(&doc, 100.0);
            prop_assert!(result.document.nodes.len() == 1);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_empty_document(_ in Just(())) {
            let doc = DiagramDocument::default();
            let result = calculate_grid_layout(&doc, 100.0);
            prop_assert!(result.document.nodes.is_empty());
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_very_tiny_cell_size(_ in Just(())) {
            let doc = make_doc(vec![
                ("a".into(), 1e-5, 1e-5, false),
                ("b".into(), 2e-5, 2e-5, false),
            ]);
            let result = calculate_grid_layout(&doc, f64::MIN_POSITIVE);
            prop_assert!(result.document.nodes.len() == 2);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_mixed_locked_unlocked(
            unlocked_count in 1_usize..10,
            locked_count in 0_usize..5,
        ) {
            let mut nodes = Vec::new();
            for i in 0..unlocked_count {
                nodes.push((format!("free_{i}"), 10.0 * i as f64, 10.0, false));
            }
            for i in 0..locked_count {
                nodes.push((format!("locked_{i}"), 100.0 * i as f64, 100.0, true));
            }
            let doc = make_doc(nodes);
            let result = calculate_grid_layout(&doc, 50.0);
            prop_assert!(result.document.nodes.len() == unlocked_count + locked_count);

            for (id, _, _, was_locked) in &[
                (format!("free_0"), 0.0, 0.0, false),
            ] {
                if let Some(node) = result.document.nodes.get(&NodeId::new(id.clone())) {
                    if !was_locked {
                        prop_assert!(!node.x.0.is_nan());
                        prop_assert!(!node.y.0.is_nan());
                    }
                }
            }
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_positions_are_finite(
            cell_size in 0.1_f64..1e6_f64,
            x1 in -1e6_f64..1e6_f64,
            y1 in -1e6_f64..1e6_f64,
            x2 in -1e6_f64..1e6_f64,
            y2 in -1e6_f64..1e6_f64,
        ) {
            let doc = make_doc(vec![
                ("a".into(), x1, y1, false),
                ("b".into(), x2, y2, false),
            ]);
            let result = calculate_grid_layout(&doc, cell_size);
            for node in result.document.nodes.values() {
                prop_assert!(node.x.0.is_finite());
                prop_assert!(node.y.0.is_finite());
            }
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_grid_layout_locked_nodes_unchanged(
            x in -1000.0_f64..1000.0_f64,
            y in -1000.0_f64..1000.0_f64,
            cell_size in 10.0_f64..1000.0_f64,
        ) {
            let doc = make_doc(vec![("locked".into(), x, y, true)]);
            let result = calculate_grid_layout(&doc, cell_size);
            let orig = doc.document.nodes.get(&NodeId::new("locked".into())).unwrap();
            let new = result.document.nodes.get(&NodeId::new("locked".into())).unwrap();
            prop_assert!((orig.x.0 - new.x.0).abs() < f64::EPSILON);
            prop_assert!((orig.y.0 - new.y.0).abs() < f64::EPSILON);
        }
    }
}
