#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::ui::commands::clipboard::*;
    use crate::ui::commands::tests_dsl::dsl::TestDsl;
    use diagram_models::document::NodeId;
    use dioxus::prelude::*;

    #[test]
    fn given_selected_node_when_copied_then_clipboard_has_content() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 100.0, 100.0, false)
                .with_selection(&["n1"])
                .build();
            let doc_signal = Signal::new(doc);
            let clipboard_signal = Signal::new(None);

            let result = apply_copy_selection(doc_signal, clipboard_signal);

            assert!(result);
            assert!(clipboard_signal.read().is_some());
            assert_eq!(clipboard_signal.read().as_ref().unwrap().nodes.len(), 1);
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }

    #[test]
    fn given_clipboard_with_content_when_pasted_then_node_is_added_with_offset() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 100.0, 100.0, false)
                .with_selection(&["n1"])
                .build();
            let mut doc_signal = Signal::new(doc);
            let clipboard_signal = Signal::new(None);
            let history_signal = Signal::new(History::new());

            apply_copy_selection(doc_signal, clipboard_signal);

            // clear selection before paste to cleanly see the effect
            doc_signal.write().editor_state.selected_items.clear();

            let result = apply_paste_selection(doc_signal, clipboard_signal, history_signal);

            assert!(result);
            let doc_read = doc_signal.read();
            assert_eq!(doc_read.document.nodes.len(), 2);
            assert_eq!(doc_read.editor_state.selected_items.len(), 1);

            // new node should have offset +20
            let new_node_id = doc_read.editor_state.selected_items.iter().next().unwrap();
            let new_node = doc_read
                .document
                .nodes
                .get(&NodeId::new(new_node_id.clone()))
                .unwrap();
            assert_eq!(new_node.x.0, 20.0);
            assert_eq!(new_node.y.0, 20.0);
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }

    #[test]
    fn given_selected_node_when_duplicated_then_node_is_copied_and_pasted() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new()
                .with_node("n1", 0.0, 0.0, 100.0, 100.0, false)
                .with_selection(&["n1"])
                .build();
            let doc_signal = Signal::new(doc);
            let clipboard_signal = Signal::new(None);
            let history_signal = Signal::new(History::new());

            let result = apply_duplicate_selection(doc_signal, clipboard_signal, history_signal);

            assert!(result);
            let doc_read = doc_signal.read();
            assert_eq!(doc_read.document.nodes.len(), 2);
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }
}
