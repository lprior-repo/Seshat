#[cfg(test)]
mod proptests {
    use crate::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use crate::validation::{validate_document, ValidationCode};
    use im::HashMap;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_finite_f64()(x in -1e6_f64..1e6_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_positive_f64()(x in 1.0_f64..1000.0_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_node_id()(s in "[a-z]{1,3}") -> NodeId { NodeId::new(s) }
    }

    prop_compose! {
        fn arb_node()(
            id in arb_node_id(),
            x in arb_finite_f64(),
            y in arb_finite_f64(),
            width in arb_positive_f64(),
            height in arb_positive_f64(),
        ) -> (NodeId, Node) {
            (
                id,
                Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: String::new(),
                    x: OrderedFloat::new_unchecked(x),
                    y: OrderedFloat::new_unchecked(y),
                    width: OrderedFloat::new_unchecked(width),
                    height: OrderedFloat::new_unchecked(height),
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
    }

    proptest! {
        fn prop_validate_never_panics_on_finite_geometry(nodes in proptest::collection::vec(arb_node(), 0..10)) {
            let mut doc = DiagramDocument::default();
            for (id, node) in nodes {
                doc.document.nodes = doc.document.nodes.update(id, node);
            }
            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != ValidationCode::INTERNAL_ERROR));
        }

        fn prop_validate_negative_dimensions_returns_error(
            id in arb_node_id(),
            width in -1000.0_f64..0.0_f64,
            height in -1000.0_f64..0.0_f64,
        ) {
            let mut doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat::new_unchecked(0.0),
                y: OrderedFloat::new_unchecked(0.0),
                width: OrderedFloat::new_unchecked(width),
                height: OrderedFloat::new_unchecked(height),
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
            };
            doc.document.nodes = doc.document.nodes.update(id, node);

            let issues = validate_document(&doc);
            prop_assert!(issues.iter().any(|i| i.code == ValidationCode::INVALID_NUMERIC));
        }

        fn prop_validate_tiny_dimensions_no_panic(
            id in arb_node_id(),
            dim in 0.0_f64..1.0_f64,
        ) {
            let mut doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat::new_unchecked(0.0),
                y: OrderedFloat::new_unchecked(0.0),
                width: OrderedFloat::new_unchecked(dim),
                height: OrderedFloat::new_unchecked(dim),
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
            };
            doc.document.nodes = doc.document.nodes.update(id, node);

            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != ValidationCode::INTERNAL_ERROR));
        }

        fn prop_validate_extreme_coords_no_panic(
            id in arb_node_id(),
            x in -1e15_f64..1e15_f64,
            y in -1e15_f64..1e15_f64,
        ) {
            let mut doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: String::new(),
                x: OrderedFloat::new_unchecked(x),
                y: OrderedFloat::new_unchecked(y),
                width: OrderedFloat::new_unchecked(64.0),
                height: OrderedFloat::new_unchecked(64.0),
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
            };
            doc.document.nodes = doc.document.nodes.update(id, node);

            let issues = validate_document(&doc);
            prop_assert!(issues.iter().all(|i| i.code != ValidationCode::INTERNAL_ERROR));
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn prop_validate_empty_doc_has_no_issues() {
        let doc = DiagramDocument::default();
        let issues = validate_document(&doc);
        assert!(issues.is_empty());
    }
}
