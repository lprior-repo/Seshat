#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
#[cfg(test)]
mod tests {
    use crate::core::nudge::nudge_selection;
    use diagram_models::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };

    fn create_test_node(x: f64, y: f64, locked: bool, kind: NodeKind) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: "Test".to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
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
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn nudge_selection_moves_unlocked_skips_locked() {
        let mut doc = DiagramDocument::default();

        let node1_id = NodeId::new("node1".to_string());
        let node2_id = NodeId::new("node2".to_string());
        let node3_id = NodeId::new("node3".to_string());

        // node1 is unlocked, should move
        doc.document.nodes.insert(
            node1_id.clone(),
            create_test_node(10.0, 10.0, false, NodeKind::Node),
        );
        // node2 is locked, shouldn't move
        doc.document.nodes.insert(
            node2_id.clone(),
            create_test_node(20.0, 20.0, true, NodeKind::Node),
        );
        // node3 is locked but is a subgraph, should move
        doc.document.nodes.insert(
            node3_id.clone(),
            create_test_node(30.0, 30.0, true, NodeKind::Subgraph),
        );

        doc.editor_state.selected_items.insert(node1_id.to_string());
        doc.editor_state.selected_items.insert(node2_id.to_string());
        doc.editor_state.selected_items.insert(node3_id.to_string());

        let initial_revision = doc.revision;

        let moved = nudge_selection(&mut doc, 5.0, -5.0);
        assert!(moved);
        assert_eq!(doc.revision.value(), initial_revision.value() + 1);

        let n1 = doc.document.nodes.get(&node1_id).unwrap();
        assert_eq!(n1.x.0, 15.0);
        assert_eq!(n1.y.0, 5.0);

        let n2 = doc.document.nodes.get(&node2_id).unwrap();
        assert_eq!(n2.x.0, 20.0);
        assert_eq!(n2.y.0, 20.0);

        let n3 = doc.document.nodes.get(&node3_id).unwrap();
        assert_eq!(n3.x.0, 35.0);
        assert_eq!(n3.y.0, 25.0);
    }

    #[test]
    fn nudge_selection_empty_does_nothing() {
        let mut doc = DiagramDocument::default();
        let initial_revision = doc.revision;
        let moved = nudge_selection(&mut doc, 5.0, -5.0);
        assert!(!moved);
        assert_eq!(doc.revision.value(), initial_revision.value());
    }

    #[test]
    fn nudge_selection_zero_delta_does_nothing() {
        let mut doc = DiagramDocument::default();
        let node1_id = NodeId::new("node1".to_string());
        doc.document.nodes.insert(
            node1_id.clone(),
            create_test_node(10.0, 10.0, false, NodeKind::Node),
        );
        doc.editor_state.selected_items.insert(node1_id.to_string());

        let initial_revision = doc.revision;
        let moved = nudge_selection(&mut doc, 0.0, 0.0);
        assert!(!moved);
        assert_eq!(doc.revision.value(), initial_revision.value());
    }
}
