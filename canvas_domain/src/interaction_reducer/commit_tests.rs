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
    use crate::interaction_reducer::commit::{commit_inline_edit, CommitError, LabelEditError};
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

            assert!(matches!(
                result,
                Err(CommitError::LabelEdit(LabelEditError::TargetNotFound))
            ));
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

            assert!(matches!(
                result,
                Err(CommitError::LabelEdit(LabelEditError::TargetNotFound))
            ));
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
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_too_long_label_when_committing_edge_edit_then_returns_validation_error() {
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
            let edit_value = Signal::new("a".repeat(1001));

            let result = commit_inline_edit(doc, history, None, Some(e1.clone()), edit_value, None);

            assert!(matches!(
                result,
                Err(CommitError::LabelEdit(LabelEditError::ValidationError))
            ));
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    use crate::interaction_reducer::commit::{
        calculate_edge_label_edit, calculate_node_label_edit,
    };

    #[test]
    fn given_valid_new_label_when_calculating_node_edit_then_returns_updated_document() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        doc.document
            .nodes
            .insert(n1.clone(), create_test_node("n1", "Old Label"));

        let new_doc = calculate_node_label_edit(&doc, &n1, "New Label").unwrap();

        assert_eq!(new_doc.document.nodes.get(&n1).unwrap().label, "New Label");
        assert_eq!(new_doc.revision, doc.revision.increment());
    }

    #[test]
    fn given_valid_new_label_when_calculating_edge_edit_then_returns_updated_document() {
        let mut doc = DiagramDocument::default();
        let e1 = EdgeId::new("e1".to_string());
        doc.document.edges.insert(
            e1.clone(),
            create_test_edge(
                NodeId::new("n1".to_string()),
                NodeId::new("n2".to_string()),
                "Old Label",
            ),
        );

        let new_doc = calculate_edge_label_edit(&doc, &e1, "New Label").unwrap();

        assert_eq!(new_doc.document.edges.get(&e1).unwrap().label, "New Label");
        assert_eq!(new_doc.revision, doc.revision.increment());
    }
}
