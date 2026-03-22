#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::ui::commands::tests_dsl::dsl::TestDsl;
    use crate::ui::commands::zorder::*;
    use diagram_models::document::NodeId;
    use dioxus::prelude::*;

    #[test]
    fn given_selected_node_when_brought_forward_then_z_index_increases() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 100.0, 100.0, false)
                .with_z_index("n1", 0)
                .with_selection(&["n1"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_bring_forward(doc_signal, history_signal);

            assert!(result);
            let node = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .clone();
            assert_eq!(node.z_index, 1);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_selected_node_when_sent_backward_then_z_index_decreases() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 100.0, 100.0, false)
                .with_z_index("n1", 1)
                .with_selection(&["n1"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_send_backward(doc_signal, history_signal);

            assert!(result);
            let node = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .clone();
            assert_eq!(node.z_index, 0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_multiple_nodes_when_brought_to_front_then_z_index_is_max_plus_one() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 100.0, 100.0, false)
                .with_z_index("n1", 0)
                .with_node("n2", 0.0, 0.0, 100.0, 100.0, false)
                .with_z_index("n2", 10)
                .with_selection(&["n1"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_bring_to_front(doc_signal, history_signal);

            assert!(result);
            let node = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .clone();
            assert_eq!(node.z_index, 11);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_multiple_nodes_when_sent_to_back_then_z_index_is_min_minus_one() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 100.0, 100.0, false)
                .with_z_index("n1", 10)
                .with_node("n2", 0.0, 0.0, 100.0, 100.0, false)
                .with_z_index("n2", 0)
                .with_selection(&["n1"])
                .build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_send_to_back(doc_signal, history_signal);

            assert!(result);
            let node = doc_signal
                .read()
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .unwrap()
                .clone();
            assert_eq!(node.z_index, -1);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }
}
