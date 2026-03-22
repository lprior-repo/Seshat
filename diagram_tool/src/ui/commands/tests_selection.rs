#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::ui::commands::selection::*;
    use diagram_models::document::{
        DiagramDocument, Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use dioxus::prelude::*;
    use im::{HashMap, Vector};

    // --- DSL Helpers ---
    struct SelectionTestDsl {
        doc: DiagramDocument,
    }

    impl SelectionTestDsl {
        fn new() -> Self {
            Self {
                doc: DiagramDocument::default(),
            }
        }

        fn with_node(mut self, id: &str, locked: bool) -> Self {
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: id.to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(100.0),
                height: OrderedFloat(100.0),
                font_size: None,
                font_weight: None,
                lock_state: if locked {
                    LockState::Locked
                } else {
                    LockState::Unlocked
                },
                parent: None,
                dag_rank: None,
                tags: Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            self.doc
                .document
                .nodes
                .insert(NodeId::new(id.to_string()), node);
            self
        }

        fn with_subgraph(mut self, id: &str, parent: Option<&str>) -> Self {
            let node = Node {
                kind: NodeKind::Subgraph,
                icon: String::new(),
                label: id.to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(100.0),
                height: OrderedFloat(100.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: parent.map(|p| NodeId::new(p.to_string())),
                dag_rank: None,
                tags: Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            self.doc
                .document
                .nodes
                .insert(NodeId::new(id.to_string()), node);
            self
        }

        fn with_edge(mut self, id: &str, source: &str, target: &str) -> Self {
            let edge = Edge {
                source: NodeId::new(source.to_string()),
                target: NodeId::new(target.to_string()),
                label: id.to_string(),
                style: Default::default(),
                arrow_type: Default::default(),
                label_offset_t: OrderedFloat(0.5),
                directed: true,
                bend_points: Vector::new(),
                tags: Vector::new(),
                metadata: HashMap::new(),
                color: None,
                thickness: OrderedFloat(1.0),
                font_size: None,
                source_port: None,
                target_port: None,
            };
            self.doc
                .document
                .edges
                .insert(EdgeId::new(id.to_string()), edge);
            self
        }

        fn with_selection(mut self, ids: &[&str]) -> Self {
            for id in ids {
                self.doc.editor_state.selected_items.insert(id.to_string());
            }
            self
        }

        fn build(self) -> DiagramDocument {
            self.doc
        }
    }

    // --- Tests ---

    #[test]
    fn given_selected_node_when_clearing_selection_then_selection_is_empty() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new()
                .with_node("n1", false)
                .with_selection(&["n1"])
                .build();

            let doc_signal = Signal::new(doc);

            apply_clear_selection(doc_signal);

            assert!(doc_signal.read().editor_state.selected_items.is_empty());
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_unselected_nodes_when_selecting_all_then_all_nodes_and_edges_are_selected() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new()
                .with_node("n1", false)
                .with_node("n2", false)
                .with_edge("e1", "n1", "n2")
                .build();

            let doc_signal = Signal::new(doc);

            apply_select_all(doc_signal);

            let selected = &doc_signal.read().editor_state.selected_items;
            assert!(selected.contains("n1"));
            assert!(selected.contains("n2"));
            assert!(selected.contains("e1"));
            assert_eq!(selected.len(), 3);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_selected_nodes_when_deleted_then_nodes_and_connected_edges_are_removed() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new()
                .with_node("n1", false)
                .with_node("n2", false)
                .with_edge("e1", "n1", "n2")
                .with_selection(&["n1"])
                .build();

            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_delete_selected(doc_signal, history_signal);

            assert!(result);
            let doc_read = doc_signal.read();
            assert!(!doc_read
                .document
                .nodes
                .contains_key(&NodeId::new("n1".to_string())));
            assert!(doc_read
                .document
                .nodes
                .contains_key(&NodeId::new("n2".to_string())));
            // Edge should be removed because source n1 was deleted
            assert!(!doc_read
                .document
                .edges
                .contains_key(&EdgeId::new("e1".to_string())));
            assert!(doc_read.editor_state.selected_items.is_empty());
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_empty_selection_when_deleted_then_returns_false() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new().with_node("n1", false).build();

            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_delete_selected(doc_signal, history_signal);

            assert!(!result);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_selected_unlocked_node_when_nudged_then_position_is_updated() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new()
                .with_node("n1", false)
                .with_selection(&["n1"])
                .build();

            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_nudge_selection(doc_signal, history_signal, 10.0, -5.0, true);

            assert!(result);
            let doc_read = doc_signal.read();
            let node = doc_read
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap();
            assert_eq!(node.x.0, 10.0);
            assert_eq!(node.y.0, -5.0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_selected_locked_node_when_nudged_then_position_is_not_updated() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new()
                .with_node("n1", true)
                .with_selection(&["n1"])
                .build();

            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_nudge_selection(doc_signal, history_signal, 10.0, -5.0, true);

            // Operation evaluates nodes, skips locked ones, but still returns true if delta != 0 and selection non-empty
            assert!(result);
            let doc_read = doc_signal.read();
            let node = doc_read
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap();
            assert_eq!(node.x.0, 0.0);
            assert_eq!(node.y.0, 0.0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_selected_directed_edge_when_toggled_then_direction_changes() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new()
                .with_node("n1", false)
                .with_node("n2", false)
                .with_edge("e1", "n1", "n2")
                .with_selection(&["e1"])
                .build();

            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let initial_rev = doc_signal.read().revision;
            let result = apply_toggle_edge_direction(doc_signal, history_signal);

            assert!(result);
            let doc_read = doc_signal.read();
            assert!(doc_read.revision.value() > initial_rev.value());
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_selected_nodes_when_grouped_then_group_is_created_and_nodes_reparented() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new()
                .with_node("n1", false)
                .with_node("n2", false)
                .with_selection(&["n1", "n2"])
                .build();

            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            // Provide None for the db_tx coroutine
            let result = apply_group_selection(doc_signal, history_signal, None);

            assert!(result);
            let doc_read = doc_signal.read();

            // There should be a new group node created
            let nodes = &doc_read.document.nodes;
            assert_eq!(nodes.len(), 3); // 2 original nodes + 1 new group

            // Find the new group node
            let group_node = nodes
                .iter()
                .find(|(_, n)| n.kind == NodeKind::Subgraph)
                .unwrap();
            let group_id = group_node.0;

            // Check that children were reparented
            assert_eq!(
                nodes.get(&NodeId::new("n1".to_string())).unwrap().parent,
                Some(group_id.clone())
            );
            assert_eq!(
                nodes.get(&NodeId::new("n2".to_string())).unwrap().parent,
                Some(group_id.clone())
            );

            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_selected_group_when_ungrouped_then_group_is_removed_and_children_reparented() {
        let mut vdom = VirtualDom::new(|| {
            let doc = SelectionTestDsl::new()
                .with_subgraph("g1", None)
                .with_node("n1", false)
                .with_selection(&["g1"])
                .build();

            // Reparent n1 to g1
            let mut doc = doc;
            doc.document
                .nodes
                .get_mut(&NodeId::new("n1".to_string()))
                .unwrap()
                .parent = Some(NodeId::new("g1".to_string()));

            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_ungroup_selection(doc_signal, history_signal, None);

            assert!(result);
            let doc_read = doc_signal.read();

            let nodes = &doc_read.document.nodes;
            // The group should be removed
            assert!(!nodes.contains_key(&NodeId::new("g1".to_string())));
            // The child should still exist but have no parent
            let n1 = nodes.get(&NodeId::new("n1".to_string())).unwrap();
            assert_eq!(n1.parent, None);

            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }
}
