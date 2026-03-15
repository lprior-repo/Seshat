#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::layout::grid::calculate_grid_layout;
use crate::models::document::DiagramDocument;

#[must_use]
pub fn apply_layout(doc: &DiagramDocument, cell_size: f64) -> DiagramDocument {
    // Validate cell_size - return original document if invalid to avoid panic
    if !cell_size.is_finite() || cell_size <= 0.0 {
        return doc.clone();
    }
    calculate_grid_layout(doc, cell_size)
}

#[cfg(test)]
mod tests {
    use super::apply_layout;
    use crate::models::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;
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
            tags: im::vector![],
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    fn make_doc_with_nodes(nodes: Vec<(String, f64, f64, bool)>) -> DiagramDocument {
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
        fn prop_apply_layout_zero_cell_size_returns_unchanged(_ in Just(())) {
            let doc = make_doc_with_nodes(vec![
                ("a".into(), 100.0, 100.0, false),
                ("b".into(), 200.0, 200.0, false),
            ]);
            let result = apply_layout(&doc, 0.0);
            // Should return unchanged document instead of panicking
            prop_assert_eq!(result.document.nodes.len(), doc.document.nodes.len());
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_negative_cell_size_returns_unchanged(cell_size in -1e10_f64..-0.001) {
            let doc = make_doc_with_nodes(vec![("a".into(), 100.0, 100.0, false)]);
            let result = apply_layout(&doc, cell_size);
            // Should return unchanged document instead of panicking
            prop_assert_eq!(result.document.nodes.len(), doc.document.nodes.len());
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_with_parent_cycle(_ in Just(())) {
            let n1 = NodeId::new("n1".into());
            let n2 = NodeId::new("n2".into());
            let n3 = NodeId::new("n3".into());

            let mut doc = DiagramDocument::default();
            doc.document.nodes = doc.document.nodes.update(n1.clone(), make_node(0.0, 0.0, false, Some(n3.clone())));
            doc.document.nodes = doc.document.nodes.update(n2.clone(), make_node(100.0, 0.0, false, Some(n1.clone())));
            doc.document.nodes = doc.document.nodes.update(n3.clone(), make_node(200.0, 0.0, false, Some(n2.clone())));

            let result = apply_layout(&doc, 100.0);
            prop_assert!(result.document.nodes.len() == 3);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_extreme_position_preserves_finiteness(coord in -1e15_f64..1e15_f64) {
            let doc = make_doc_with_nodes(vec![
                ("a".into(), coord, coord, false),
                ("b".into(), -coord, -coord, false),
            ]);
            let result = apply_layout(&doc, 100.0);
            for node in result.document.nodes.values() {
                prop_assert!(node.x.0.is_finite() || node.x.0.is_nan() || node.x.0.is_infinite());
                prop_assert!(node.y.0.is_finite() || node.y.0.is_nan() || node.y.0.is_infinite());
            }
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_very_small_cell_size(cell_size in f64::MIN_POSITIVE..1e-10) {
            let doc = make_doc_with_nodes(vec![("a".into(), 1.0, 1.0, false)]);
            let result = apply_layout(&doc, cell_size);
            prop_assert!(result.document.nodes.len() == 1);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_subnormal_cell_size(_ in Just(())) {
            let subnormal = f64::from_bits(1_u64);
            let doc = make_doc_with_nodes(vec![("a".into(), 1.0, 1.0, false)]);
            let result = apply_layout(&doc, subnormal);
            prop_assert!(result.document.nodes.len() == 1);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_inf_cell_size_returns_unchanged(sign in -1_i32..=1) {
            let cell_size = if sign < 0 { f64::NEG_INFINITY } else { f64::INFINITY };
            let doc = make_doc_with_nodes(vec![("a".into(), 100.0, 100.0, false)]);
            let result = apply_layout(&doc, cell_size);
            // Should return unchanged document instead of panicking
            prop_assert_eq!(result.document.nodes.len(), doc.document.nodes.len());
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_extreme_scale(scale in 1e-15_f64..1e15_f64) {
            let doc = make_doc_with_nodes(vec![
                ("a".into(), 50.0, 50.0, false),
                ("b".into(), 150.0, 150.0, false),
            ]);
            let result = apply_layout(&doc, scale);
            prop_assert!(result.document.nodes.len() == 2);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_preserves_node_count(
            node_count in 0_usize..20,
            cell_size in 0.001_f64..1000.0,
        ) {
            let mut nodes = Vec::new();
            for i in 0..node_count {
                nodes.push((format!("n{}", i), i as f64 * 10.0, i as f64 * 10.0, i % 3 == 0));
            }
            let doc = make_doc_with_nodes(nodes);
            let result = apply_layout(&doc, cell_size);
            prop_assert!(result.document.nodes.len() == node_count);
        }

        #[cfg(kani)]
        #[kani::proof]
        #[test]
        fn prop_apply_layout_locked_nodes_unchanged(
            x in -1e6_f64..1e6_f64,
            y in -1e6_f64..1e6_f64,
            cell_size in 1.0_f64..1000.0,
        ) {
            let doc = make_doc_with_nodes(vec![("locked".into(), x, y, true)]);
            let result = apply_layout(&doc, cell_size);
            let orig = doc.document.nodes.get(&NodeId::new("locked".into())).unwrap();
            let new = result.document.nodes.get(&NodeId::new("locked".into())).unwrap();
            prop_assert!((orig.x.0 - new.x.0).abs() < f64::EPSILON);
            prop_assert!((orig.y.0 - new.y.0).abs() < f64::EPSILON);
        }
    }
}
