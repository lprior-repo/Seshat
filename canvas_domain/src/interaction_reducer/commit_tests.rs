#[cfg(test)]
mod tests {
    use crate::interaction_reducer::commit::{commit_inline_edit, CommitError};
    use diagram_models::document::{
        DiagramDocument, Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use diagram_models::history::History;
    use dioxus::prelude::*;
    use im::Vector;

    fn create_test_node(id: &str, label: &str) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: label.to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn create_test_edge(source: NodeId, target: NodeId, label: &str) -> Edge {
        Edge {
            source,
            target,
            label: label.to_string(),
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat(0.5),
            directed: true,
            bend_points: Vector::new(),
            tags: Vector::new(),
            metadata: im::HashMap::new(),
            color: None,
            thickness: OrderedFloat(1.0),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    #[test]
    fn given_missing_node_target_when_committing_inline_edit_then_returns_target_not_found_error() {
        let mut vdom = VirtualDom::new(|| {
            let doc = Signal::new(DiagramDocument::default());
            let history = Signal::new(History::new());
            let edit_value = Signal::new("New Label".to_string());

            let result = commit_inline_edit(
                doc,
                history,
                Some(NodeId::new("missing".to_string())),
                None,
                edit_value,
                None,
            );

            assert!(matches!(result, Err(CommitError::TargetNotFound)));
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_missing_edge_target_when_committing_inline_edit_then_returns_target_not_found_error() {
        let mut vdom = VirtualDom::new(|| {
            let doc = Signal::new(DiagramDocument::default());
            let history = Signal::new(History::new());
            let edit_value = Signal::new("New Label".to_string());

            let result = commit_inline_edit(
                doc,
                history,
                None,
                Some(EdgeId::new("missing".to_string())),
                edit_value,
                None,
            );

            assert!(matches!(result, Err(CommitError::TargetNotFound)));
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_no_target_when_committing_inline_edit_then_returns_ok_false() {
        let mut vdom = VirtualDom::new(|| {
            let doc = Signal::new(DiagramDocument::default());
            let history = Signal::new(History::new());
            let edit_value = Signal::new("New Label".to_string());

            let result = commit_inline_edit(doc, history, None, None, edit_value, None);

            assert!(matches!(result, Ok(false)));
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_existing_node_and_different_label_when_committing_then_updates_label_and_returns_true()
    {
        let mut vdom = VirtualDom::new(|| {
            let mut initial_doc = DiagramDocument::default();
            let n1 = NodeId::new("n1".to_string());
            initial_doc
                .document
                .nodes
                .insert(n1.clone(), create_test_node("n1", "Old Label"));

            let doc = Signal::new(initial_doc);
            let history = Signal::new(History::new());
            let edit_value = Signal::new("New Label".to_string());

            let result = commit_inline_edit(doc, history, Some(n1.clone()), None, edit_value, None);

            assert!(matches!(result, Ok(true)));
            assert_eq!(
                doc.read().document.nodes.get(&n1).unwrap().label,
                "New Label"
            );
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_existing_edge_and_different_label_when_committing_then_updates_label_and_returns_true()
    {
        let mut vdom = VirtualDom::new(|| {
            let mut initial_doc = DiagramDocument::default();
            let e1 = EdgeId::new("e1".to_string());
            initial_doc.document.edges.insert(
                e1.clone(),
                create_test_edge(
                    NodeId::new("n1".to_string()),
                    NodeId::new("n2".to_string()),
                    "Old Label",
                ),
            );

            let doc = Signal::new(initial_doc);
            let history = Signal::new(History::new());
            let edit_value = Signal::new("New Label".to_string());

            let result = commit_inline_edit(doc, history, None, Some(e1.clone()), edit_value, None);

            assert!(matches!(result, Ok(true)));
            assert_eq!(
                doc.read().document.edges.get(&e1).unwrap().label,
                "New Label"
            );
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_existing_node_and_same_label_when_committing_then_returns_ok_false_with_no_changes() {
        let mut vdom = VirtualDom::new(|| {
            let mut initial_doc = DiagramDocument::default();
            let n1 = NodeId::new("n1".to_string());
            initial_doc
                .document
                .nodes
                .insert(n1.clone(), create_test_node("n1", "Same Label"));

            let doc = Signal::new(initial_doc);
            let history = Signal::new(History::new());
            let edit_value = Signal::new("Same Label".to_string());

            let result = commit_inline_edit(doc, history, Some(n1.clone()), None, edit_value, None);

            assert!(matches!(result, Ok(false)));
            assert_eq!(
                doc.read().document.nodes.get(&n1).unwrap().label,
                "Same Label"
            );
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }
}
