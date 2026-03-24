use crate::layout::grid::algorithm::calculate_grid_layout;
use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;

#[cfg(test)]
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
    #[should_panic(expected = "cell_size must be positive and finite")]
    fn prop_grid_layout_zero_cell_size(_ in Just(0.0_f64)) {
        let doc = make_doc(vec![("a".into(), 100.0, 100.0, false)]);
        let _result = calculate_grid_layout(&doc, 0.0);
    }

    #[cfg(kani)]
#[kani::proof]
    #[should_panic(expected = "cell_size must be positive and finite")]
    fn prop_grid_layout_negative_cell_size(cell_size in -1e10_f64..-1e-10_f64) {
        let doc = make_doc(vec![("a".into(), 100.0, 100.0, false)]);
        let _result = calculate_grid_layout(&doc, cell_size);
    }

    #[cfg(kani)]
#[kani::proof]
    #[should_panic(expected = "cell_size must be positive and finite")]
    fn prop_grid_layout_nan_cell_size(_ in Just(())) {
        let doc = make_doc(vec![("a".into(), 100.0, 100.0, false)]);
        let _result = calculate_grid_layout(&doc, f64::NAN);
    }

    #[cfg(kani)]
#[kani::proof]
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
    fn prop_grid_layout_empty_document(_ in Just(())) {
        let doc = DiagramDocument::default();
        let result = calculate_grid_layout(&doc, 100.0);
        prop_assert!(result.document.nodes.is_empty());
    }

    #[cfg(kani)]
#[kani::proof]
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
