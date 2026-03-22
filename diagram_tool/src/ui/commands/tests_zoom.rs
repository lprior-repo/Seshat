#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::ui::commands::tests_dsl::dsl::TestDsl;
    use crate::ui::commands::zoom::*;
    use dioxus::prelude::*;

    #[test]
    fn given_default_zoom_when_zoomed_in_then_zoom_increases() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new().with_zoom(1.0, 0.0, 0.0).build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_zoom_in(doc_signal, history_signal, (1000.0, 1000.0));

            assert!(result);
            assert_eq!(doc_signal.read().editor_state.zoom.0, 1.25);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_default_zoom_when_zoomed_out_then_zoom_decreases() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new().with_zoom(1.0, 0.0, 0.0).build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_zoom_out(doc_signal, history_signal, (1000.0, 1000.0));

            assert!(result);
            assert_eq!(doc_signal.read().editor_state.zoom.0, 0.8);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_modified_zoom_when_reset_then_zoom_is_one() {
        let mut vdom = VirtualDom::new(|| {
            let doc = TestDsl::new().with_zoom(2.0, 0.0, 0.0).build();
            let doc_signal = Signal::new(doc);
            let history_signal = Signal::new(History::new());

            let result = apply_zoom_reset(doc_signal, history_signal, (1000.0, 1000.0));

            assert!(result);
            assert_eq!(doc_signal.read().editor_state.zoom.0, 1.0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_history_when_undo_then_reverts_to_previous_state() {
        let mut vdom = VirtualDom::new(|| {
            let doc1 = TestDsl::new().with_zoom(1.0, 0.0, 0.0).build();
            let doc2 = TestDsl::new().with_zoom(1.25, 0.0, 0.0).build();

            let doc_signal = Signal::new(doc2);
            let mut history = History::new();
            history = history.push(doc1); // the state before doc2
            let history_signal = Signal::new(history);

            apply_undo(doc_signal, history_signal);

            assert_eq!(doc_signal.read().editor_state.zoom.0, 1.0);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_undone_state_when_redo_then_restores_state() {
        let mut vdom = VirtualDom::new(|| {
            let doc1 = TestDsl::new().with_zoom(1.0, 0.0, 0.0).build();
            let doc2 = TestDsl::new().with_zoom(1.25, 0.0, 0.0).build();

            let doc_signal = Signal::new(doc2);
            let mut history = History::new();
            history = history.push(doc1.clone());
            let history_signal = Signal::new(history);

            apply_undo(doc_signal, history_signal);
            assert_eq!(doc_signal.read().editor_state.zoom.0, 1.0);

            apply_redo(doc_signal, history_signal);
            assert_eq!(doc_signal.read().editor_state.zoom.0, 1.25);
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }
}
